# Fontes escritas em Rust

O Sankofa Rust aceita dois tipos de fonte:

| Tipo | Formato | Como roda |
|---|---|---|
| Extensão legada | APK Kotlin (Mihon/Komikku) | carregada pela JVM, como sempre |
| **Fonte em Rust** | módulo WebAssembly (`.wasm`) | executada pelo núcleo Rust (`sk-source`, interpretador wasmi) |

As duas aparecem juntas em Navegar > Fontes; a fonte em Rust herda o mesmo caminho de imagens do app (`HttpSource`), com cookies, Cloudflare e `Referer`.

## Como escrever uma fonte
1. Crie um crate `cdylib` que dependa de `sk-source-sdk` (em `rust/crates/sk-source-sdk`). Para um site de um tema conhecido (MangaThemesia, Madara, Zeist, Keyoapp, MangaBox), dependa do crate do tema (`sk-theme-*`) e só declare o que muda: ~10-50 linhas.
2. Implemente `Source` e exporte com `export_source!`:

```rust
use sk_source_sdk::*;

#[derive(Default)]
struct MySource;

impl Source for MySource {
    fn info(&self) -> Info { Info { name: "Minha fonte".into(), lang: "en".into(), base_url: "https://exemplo.org".into(), ..Default::default() } }
    fn popular(&self, page: u32) -> Result<MangaPage> { /* get(...).send()?.html() e seletores */ }
    fn details(&self, manga: &Manga) -> Result<Manga> { /* ... */ }
    fn chapters(&self, manga: &Manga) -> Result<Vec<Chapter>> { /* ... */ }
    fn pages(&self, chapter: &Chapter) -> Result<Vec<Page>> { /* ... */ }
}
export_source!(MySource);
```
3. Compile: `cargo build --release --target wasm32-unknown-unknown` (uma vez: `rustup target add wasm32-unknown-unknown`).
4. No PC, teste com HTML salvo do site (`set_mock_http`, `include_str!`); no aparelho de desenvolvimento, `rust/tools/push-sources.ps1` copia os `.wasm` para o app de depuração e o app roda um teste de fumaça (popular, últimos, busca, título, capítulo, primeira imagem) no log (`Rust smoke`).
5. Assine e empacote (próxima seção) e instale pelo app em **Configurações > Navegar > Fontes em Rust > Instalar de um arquivo**.

Referências: `rust/sources/mangadex` (API JSON), `rust/sources/arenascans` (tema), `rust/sources/mangafire` (assinatura de pedidos, idiomas, preferências), `rust/sources/luscious` (GraphQL, 12 idiomas), `docs/redesign/EXTENSIONS-TO-RUST.md` (análise e plano de conversão das extensões).

## Contrato da fonte (`Source`)
| Método | Obrigatório | Retorno |
|---|---|---|
| `info()` | sim | `Info`: nome, idioma, `base_url`, espelhos (`mirrors`, viram a configuração "Domínio"), `supports_latest`, `supports_filter_fetching`, `supports_related`, `custom_urls`, `version_id` (1 por padrão), `id` (0 = o app deriva como as extensões faziam), `rate_limit` (inclusive por host), `prefs`, `headers`, `cookies` |
| `popular(page)` | sim | `MangaPage { mangas, has_next }` |
| `latest(page)`, `search(page, query, filters)` | não | idem; o padrão devolve erro "não suportado" |
| `filters(data)`, `filter_data()` | não | definição dos filtros (texto, seleção, caixa, tri-estado, grupo, ordenação) e dados baixados do site (gêneros), guardados pelo app por um dia |
| `details(manga)`, `chapters(manga)` | sim | `Manga` (autor, artista, descrição, gêneros, `status` como no `SManga`) e `Vec<Chapter>` |
| `update(manga, detalhes, capítulos)` | não | os dois numa só ida à rede (padrão: chama os dois) |
| `pages(chapter)` | sim | `Vec<Page>` com `image_url`, `headers`, `ops` (edições de imagem) e `mirrors` |
| `by_url(url)`, `related(manga)` | não | abrir um link do site; títulos relacionados |
| `manga_url(manga)`, `chapter_url(chapter)` | não | endereço para abrir no navegador quando difere do guardado |

`Manga.url` e `Chapter.url` são texto que a fonte escolhe (caminho, id) e o app devolve sem alterar; `memo` guarda o que a fonte quiser de volta (ids achados na página, um slug). Os caminhos guardados seguem o formato da extensão original para a biblioteca e o histórico continuarem ligados (`path_of` decodifica como o `setUrlWithoutDomain`).

`ImageOp` (`Merge`, `Unscramble`, `Shuffle`, `Text`): a fonte descreve, o app executa no `Bitmap` (um interpretador sem JIT seria lento demais para pixels).

## ABI (para quem não usa o SDK) — versão 2
O módulo exporta `memory`, `sk_alloc(len) -> ptr`, `sk_free(ptr, len)` e `sk_call(ptr, len) -> i64`.
`sk_call` recebe um JSON de pedido e devolve `(ptr << 32) | len` de um JSON de resposta (o host o libera com `sk_free`). Todo pedido leva `ctx`: `{"lang":"en","prefs":{...},"filter_data":...}` (o idioma em que a cópia foi instalada, as preferências do usuário e os dados de filtro em cache).

Pedidos (`op`): `info`, `filters`, `filter_data`, `popular`, `latest`, `search` (com `query` e `filters`), `update`, `details`, `chapters`, `pages`, `by_url`, `related`, `manga_url`, `chapter_url`.
Respostas: `{"info":...,"abi":2}`, `{"page":{...}}`, `{"manga":...}`, `{"update":{...}}`, `{"chapters":[...]}`, `{"pages":[...]}`, ... ou `{"error":"mensagem"}`.

O host oferece ao módulo, no namespace `sk`: `http` e `http_batch` (um ou vários pedidos, método, cabeçalhos, corpo, binário, sem redirecionamento, limite de bytes), `http_get`, `cookie(url, nome)`, `now_ms()` e `log`. A rede é a do app (OkHttp: Cloudflare, cookies, limite de taxa, User-Agent).

## Segurança e limites
- Uma fonte **não abre sockets nem arquivos**: tudo passa por `http`, que **recusa endereços locais e de rede privada** (`localhost`, `127.*`, `10.*`, `172.16-31.*`, `192.168.*`, `169.254.*`, IPv6 local, nomes `.local`/`.lan`/`.internal`, só `http(s)`), para uma fonte não poder sondar o roteador ou outros aparelhos da sua rede. Limite conhecido: confere o endereço escrito, não o que um nome público resolve (DNS rebinding).
- Cada chamada tem um orçamento de 6 bilhões de instruções (alguns segundos num telefone; uma página com milhares de capítulos cabe); estourou, a chamada é interrompida com erro, o app continua. Seletores com `:has`/`:eq` sobre páginas enormes são o caminho mais comum para estourar.
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

## Repositório oficial
- **Endereço:** `https://uzeda4.github.io/sankofa-rust/index.skrepo` (GitHub Pages, ramo `gh-pages` deste repositório; conteúdo: `index.skrepo`, os pacotes `.sksource` e uma página de apresentação).
- **Já vem no app:** na primeira vez que a tela Fontes em Rust é aberta, o app cadastra este repositório e confia na impressão digital abaixo (fixa no código, `RustSourceRepos.OFFICIAL`). Uma vez só: se você o remover, não volta; quem já o adicionou à mão mantém a série vista.
- **Impressão digital que o app deve mostrar:** `aca83acae150b852b62231082950406f`. Se for outra, **não continue**.
- Fontes (série 4): MangaDex (Rust) v2 e as 12 convertidas das extensões (Arena Scans, Asmodeus Scans, Asura Scans, Kissmanga.in, Luscious ×12 idiomas, Manga Demon, MangaFire ×7, MangaFox, Mangakakalot, MurimScan, Read Comics Online, Webtoons.com ×7): 36 pacotes, cada um com o ícone da extensão original (`rust/sources/<fonte>/icon.png`).
- `rust/tools/publish-sources.ps1 -Key <chave> -Out <checkout do gh-pages> -Serial <n>` empacota e assina tudo, copia os ícones (`icons/`) e gera o `index.skrepo`; depois é só commit e push no `gh-pages`. O índice lista o ícone de cada fonte (`--icon "Nome=icons/x.png"` no `sk-pack index`); o app baixa os ícones ao ler o repositório ou instalar, mostra na tela de Fontes em Rust e em Navegar.
- Para publicar uma versão nova: gere o pacote e o índice com `sk-pack` (**série maior que a anterior**), copie os dois arquivos para o ramo `gh-pages` e envie. O app só aceita série igual ou maior.

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

### Verificado no aparelho (servidor local pelo `adb reverse`; depois com o repositório oficial)
Depois de publicado no GitHub Pages: o app leu o índice por HTTPS, mostrou a impressão digital `aca83aca…`, e instalou a MangaDex (Rust) v2 (série 3).

Teste com o servidor local:
Adicionar o repositório mostrou a impressão digital `aca83aca…`; a fonte apareceu em Disponíveis, instalou (v1) e, com o índice na série 2 e o pacote v2, o botão virou Atualizar e atualizou para v2. Ao servir de volta o índice da série 1, o app recusou: "o repositório serviu um índice mais antigo (série 1 < 2)", e manteve a v2 instalada. Testes no PC: 5 do formato do índice (ida e volta, byte alterado, assinatura de outra chave, lixo, URLs relativas).

## O que ainda não existe
- **Filtros** e **preferências por fonte** nas fontes em Rust.
- **Revogação remota** e rotação de chave (hoje a chave de um repositório não pode mudar: o app recusa).
- **Cookies persistentes e Cloudflare** no `sk-net` (o desafio precisa do WebView).
- Segunda ABI (WIT/Component Model) se um dia valer a pena; a JSON-sobre-memória funciona em qualquer linguagem com saída WebAssembly (Rust, Zig, AssemblyScript, C).

