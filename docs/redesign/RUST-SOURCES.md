# Fontes escritas em Rust

O Sankofa Rust aceita dois tipos de fonte:

| Tipo | Formato | Como roda |
|---|---|---|
| Extensão legada | APK Kotlin (Mihon/Komikku) | carregada pela JVM, como sempre |
| **Fonte em Rust** | módulo WebAssembly (`.wasm`) | executada pelo núcleo Rust (`sk-source`, interpretador wasmi) |

As duas aparecem juntas em Navegar > Fontes; a fonte em Rust herda o mesmo caminho de imagens do app (`HttpSource`), com cookies, Cloudflare e `Referer`.

## Como escrever uma fonte
1. Crie um crate `cdylib` que dependa de `sk-source-sdk` (em `rust/crates/sk-source-sdk`).
2. Implemente `Source` e exporte com `export_source!`:

```rust
use sk_source_sdk::*;

#[derive(Default)]
struct MySource;

impl Source for MySource {
    fn info(&self) -> Info { Info { name: "Minha fonte".into(), lang: "en".into(), base_url: "https://exemplo.org".into(), supports_latest: false, id: 0 } }
    fn popular(&self, page: u32) -> Result<MangaPage> { /* http_get(...) e parse */ }
    fn details(&self, url: &str) -> Result<Manga> { /* ... */ }
    fn chapters(&self, url: &str) -> Result<Vec<Chapter>> { /* ... */ }
    fn pages(&self, chapter_url: &str) -> Result<Vec<Page>> { /* ... */ }
}
export_source!(MySource);
```
3. Compile: `cargo build --release --target wasm32-unknown-unknown` (uma vez: `rustup target add wasm32-unknown-unknown`).
4. Copie o `.wasm` para `files/rust-sources/` do app e reabra o app.

Referência completa: `rust/sources/mangadex` (popular, últimos, busca, detalhes, capítulos e páginas, 235 KB).

## Contrato da fonte (`Source`)
| Método | Obrigatório | Retorno |
|---|---|---|
| `info()` | sim | nome, idioma, `base_url`, `supports_latest`, `id` (0 = o app deriva de nome+idioma) |
| `popular(page)` | sim | `MangaPage { mangas, has_next }` |
| `latest(page)`, `search(page, query)` | não | idem; o padrão devolve erro "não suportado" |
| `details(url)` | sim | `Manga` (autor, artista, descrição, gêneros, `status` como no `SManga`) |
| `chapters(url)` | sim | `Vec<Chapter>` (nome, número, scanlator, data em ms) |
| `pages(chapter_url)` | sim | `Vec<Page>` com `image_url` |

`url` é qualquer texto que identifique o item dentro da fonte (caminho ou id); o app o devolve sem alterar.

## ABI (para quem não usa o SDK)
O módulo exporta `memory`, `sk_alloc(len) -> ptr`, `sk_free(ptr, len)` e `sk_call(ptr, len) -> i64`.
`sk_call` recebe um JSON de pedido na memória do módulo e devolve `(ptr << 32) | len` de um JSON de resposta (o host o libera com `sk_free`).

Pedidos: `{"op":"info"}`, `{"op":"popular","page":1}`, `{"op":"latest","page":1}`, `{"op":"search","page":1,"query":"..."}`, `{"op":"details","url":"..."}`, `{"op":"chapters","url":"..."}`, `{"op":"pages","url":"..."}`.
Respostas: `{"info":...}`, `{"page":{"mangas":[...],"has_next":true}}`, `{"manga":...}`, `{"chapters":[...]}`, `{"pages":[...]}` ou `{"error":"mensagem"}`.

O host oferece ao módulo, no namespace `sk`:
- `http_get(url_ptr, url_len, headers_ptr, headers_len) -> i64`: GET; `headers` é um JSON `[["Nome","valor"],...]`; devolve `(ptr << 32) | len` de `{"status":200,"body":"..."}` ou `{"status":0,"error":"..."}`, alocado pelo `sk_alloc` do módulo.
- `log(ptr, len)`: escreve uma linha no log do host.

## Segurança e limites
- Uma fonte **não abre sockets nem arquivos**: tudo passa por `http_get`.
- Cada chamada tem um orçamento de 400 milhões de instruções (cerca de alguns segundos num telefone); estourou, a chamada é interrompida com erro, o app continua.
- Pedido e resposta limitados a 16 MB.
- Os módulos são carregados da pasta privada do app, que só o próprio app e o `adb` de um app depurável escrevem. Um instalador com confiança por assinatura (como o das extensões) ainda não existe.

## O que ainda não existe
- **Filtros** (`Filter`/`FilterList`): a busca recebe só o texto.
- **Preferências por fonte** (`ConfigurableSource`).
- **Instalação pela interface** e repositório de fontes Rust; hoje o `.wasm` é copiado à mão.
- **Cookies e sessão** persistentes no `sk-net`, e o desafio do Cloudflare, que precisa do WebView; hoje a fonte só vê o que o `sk-net` consegue baixar.
- Segunda ABI (WIT/Component Model) se um dia valer a pena; a JSON-sobre-memória foi escolhida por funcionar em qualquer linguagem com saída WebAssembly (Rust, Zig, AssemblyScript, C).
