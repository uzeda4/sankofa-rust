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
4. Assine e empacote (próxima seção) e instale pelo app em **Configurações > Navegar > Fontes em Rust > Instalar de um arquivo**.

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
- Uma fonte **não abre sockets nem arquivos**: tudo passa por `http_get`, que **recusa endereços locais e de rede privada** (`localhost`, `127.*`, `10.*`, `172.16-31.*`, `192.168.*`, `169.254.*`, IPv6 local, nomes `.local`/`.lan`/`.internal`, só `http(s)`), para uma fonte não poder sondar o roteador ou outros aparelhos da sua rede. Limite conhecido: confere o endereço escrito, não o que um nome público resolve (DNS rebinding).
- Cada chamada tem um orçamento de 400 milhões de instruções (cerca de alguns segundos num telefone); estourou, a chamada é interrompida com erro, o app continua.
- Pedido e resposta limitados a 16 MB.
- Os módulos são carregados da pasta privada do app, que só o próprio app e o `adb` de um app depurável escrevem. Um instalador com confiança por assinatura (como o das extensões) ainda não existe.

## Pacotes assinados (`.sksource`)
Uma fonte instalada é um pacote assinado com Ed25519: o módulo `.wasm` mais um manifesto (nome, idioma, versão, publicador, chave pública, SHA-256 do módulo e a assinatura). A assinatura cobre nome, idioma, versão e o hash do módulo, então trocar o módulo ou renomear o pacote a invalida.

```text
cargo build -p sk-pack --release
sk-pack keygen  chave.key                      # uma vez; gera a chave e mostra a impressão digital
sk-pack pack --key chave.key --wasm fonte.wasm --name "Minha fonte" --lang en --version 1 \
             --publisher "Meu nome" --out minha-fonte.sksource
sk-pack inspect minha-fonte.sksource           # confere assinatura e mostra a impressão digital
```
A chave privada (`chave.key`, a semente em hex) **não vai para o repositório**: quem a tem publica fontes em que seus usuários confiam. Guarde uma cópia; sem ela não há como assinar uma atualização com a mesma identidade.

### Confiança
- A **impressão digital** do publicador é o SHA-256 da chave pública (32 caracteres hex). Ao instalar, o app mostra nome, versão, publicador e impressão digital, e só instala depois que o usuário confirma ("Confiar no publicador e instalar").
- As impressões digitais confiáveis ficam guardadas no app. **A cada abertura** o app reverifica assinatura e confiança: um pacote cuja chave foi revogada, cujo arquivo foi alterado ou que veio de um publicador não confiável não carrega e aparece em "Não carregada", com o botão Confiar e a impressão digital para conferir.
- Atualizar: o mesmo nome e idioma, versão igual ou maior, **mesma chave**. Outra chave com o mesmo nome é recusada, e uma versão menor também.
- Um módulo `.wasm` solto na pasta só carrega em build de depuração.

### Verificado no aparelho
Pacote adulterado (1 byte do módulo alterado): recusado na instalação ("o módulo não confere com o manifesto"). Pacote íntegro: o diálogo mostra a impressão digital `aca83acae150b852b62231082950406f`, a mesma que o `sk-pack` imprime; depois de confiar, a fonte instala e aparece em Navegar. Apagar a confiança e reabrir: a fonte some e o pacote aparece como não carregado. 8 testes do formato (round trip, hash, assinatura, chave trocada, lixo) rodam no PC.

## Repositórios de fontes (`index.skrepo`)
Um repositório é um arquivo assinado que lista pacotes. O app o baixa, mostra o que há, instala e atualiza.

```text
sk-pack index --key chave-do-repo.key --name "Meu repositório" --serial 3 --out index.skrepo \
              fonte-a-1.sksource fonte-b-4.sksource      # os pacotes ficam ao lado do índice
```
Hospede `index.skrepo` e os `.sksource` na mesma pasta (qualquer servidor estático); as URLs dos pacotes no índice são relativas a ele. No app: **Fontes em Rust > Adicionar repositório**, cole o endereço do `index.skrepo`.

- **Formato:** `SKREPO1`, o corpo em JSON (nome, chave pública, `serial`, lista com nome, idioma, versão, URL, SHA-256 e tamanho) e a assinatura Ed25519 de `sankofa-repo-v1\n` + bytes do corpo.
- **Confiança:** ao adicionar, o app mostra a impressão digital do repositório e só segue com a sua confirmação; confiar no repositório também confia na chave dele como publicador de pacotes (a chave do repositório pode ser a mesma dos pacotes, como no teste).
- **Cada abertura da tela** relê os índices e recusa: assinatura inválida, **chave diferente** da confirmada, e **série menor** que a última vista (impede reapresentar uma lista antiga). O pacote baixado ainda precisa bater com o SHA-256 do índice **e** ter assinatura de publicador válida e confiável.
- **Atualizar:** o botão aparece quando o índice tem versão maior que a instalada.

### Verificado no aparelho (servidor local pelo `adb reverse`)
Adicionar o repositório mostrou a impressão digital `aca83aca…`; a fonte apareceu em Disponíveis, instalou (v1) e, com o índice na série 2 e o pacote v2, o botão virou Atualizar e atualizou para v2. Ao servir de volta o índice da série 1, o app recusou: "o repositório serviu um índice mais antigo (série 1 < 2)", e manteve a v2 instalada. Testes no PC: 5 do formato do índice (ida e volta, byte alterado, assinatura de outra chave, lixo, URLs relativas).

## O que ainda não existe
- **Filtros** e **preferências por fonte** nas fontes em Rust.
- **Revogação remota** e rotação de chave (hoje a chave de um repositório não pode mudar: o app recusa).
- **Cookies persistentes e Cloudflare** no `sk-net` (o desafio precisa do WebView).
- Segunda ABI (WIT/Component Model) se um dia valer a pena; a JSON-sobre-memória funciona em qualquer linguagem com saída WebAssembly (Rust, Zig, AssemblyScript, C).

