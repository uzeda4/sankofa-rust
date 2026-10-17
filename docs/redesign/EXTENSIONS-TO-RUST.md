# Extensões instaladas -> fontes em Rust

Análise das 12 extensões de mangá instaladas no aparelho (as de anime, `plutotv` e `s1flex`, ficam fora) e o plano para convertê-las em fontes Rust (WebAssembly) com o mesmo comportamento, mais o que dá para melhorar no caminho.

Código-fonte lido: `keiyoushi/extensions-source` (commit `bca16780`, extensões 1.6.x, API "KeiSource" com `memo`, `fetchMangaUpdate`, filtros com dados buscados na rede).

## 1. Como uma extensão funciona hoje

Um APK (DEX) carregado por `ChildFirstPathClassLoader`. Cada extensão herda de uma classe base que já traz o site inteiro; o APK só diz qual site.

| Peça | O que faz | Onde está |
|---|---|---|
| Tema (multisrc) | Raspa o HTML de uma família de sites com seletores CSS, regras de URL, filtros e datas | `lib-multisrc/*` |
| Extensão | Nome, idioma, `baseUrl` e sobrescritas de seletor | `src/<lang>/<nome>` |
| Cliente HTTP | OkHttp com interceptores: limite de taxa, Cloudflare (WebView), cookies, cabeçalhos | app (`NetworkHelper`) + `configureClient()` |
| Preferências | `androidx.preference` por fonte (domínio, mesclar imagens...) | `ConfigurableSource` |
| Imagens | `imageRequest()` e interceptores que reescrevem a resposta (remontar, mesclar, desenhar texto) | interceptores OkHttp |

### As 12 fontes

| Fonte | Base | Linhas | O que exige além de HTML + seletores |
|---|---|---|---|
| Arena Scans | MangaThemesia | 7 (+ tema 622) | filtros com gêneros buscados do site, datas em vários idiomas, `ts_reader.run` em base64, "ver" via `admin-ajax` |
| Asmotoon | KeyoappV2 | 7 (+ tema 224) | respostas JSON com HTML embutido (`_ajax=search`), filtros dinâmicos, preferência de capítulos pagos |
| Kissmanga.in | Madara | 15 (+ tema 740) | POST `admin-ajax` (`madara_load_more`), 5 modos de lista de capítulos, AES (capítulos protegidos), datas relativas multilíngues, `srcset` |
| Murim Scan | ZeistManga | 20 (+ tema 676) | feed JSON do Blogger, regex pesada, `memo` |
| Mangakakalot | MangaBox | 57 (+ tema 645) | API de capítulos em JSON, **mesclar imagens cortadas** (decodifica e cola bitmaps), **CDN alternativo** com troca automática, preferência, rate limit, tamanho de WebP por `Range` |
| ReadComicsOnline | próprio | 103 | só HTML |
| Mangademon | próprio | 253 | HTML, filtros |
| MangaFox | próprio | 338 | HTML, filtros |
| Webtoons | próprio (7+ idiomas) | 492 | HTML, **desenhar texto como imagem** (notas do autor), preferências, `getMangaByUrl` |
| Asura Scans | próprio | 795 | JSON, **imagem embaralhada** (reordena faixas num `Canvas`), token por cookie, preferências, filtros dinâmicos |
| Luscious | próprio | 1051 | GraphQL (POST JSON), filtros ricos, preferências (resolução, ordem, mesclar), reescreve `Content-Type` de imagens |
| MangaFire | próprio, 7 idiomas | 1007 | API JSON com **assinatura `vrf`** (3 estágios de tabela, portável), **captcha de formas** resolvido em WebView com OpenCV.js, 7 preferências, capítulos em paralelo |

Somando: 7 famílias de código para 12 fontes. Quem escreve o motor de cada família uma vez entrega a maior parte das fontes.

## 2. O que o contrato Rust atual não tem

O ABI v1 (`popular/latest/search(query)/details/chapters/pages` + `http_get`) cobre o MangaDex, mas não essas fontes:

| Falta | Quem precisa | Solução |
|---|---|---|
| Analisar HTML e seletores no estilo jsoup (`:contains`, `:containsOwn`, `:has`, `ownText`, `abs:`) | todas menos Luscious/MangaFire | crate `sk-html` no SDK, testada contra páginas reais |
| POST (form, JSON), cabeçalhos, status, URL final, cookies | Madara, Luscious, Keyoapp, Webtoons | `http_request` v2 |
| Cloudflare, cookies, User-Agent, taxa de requisições | quase todas | **a rede passa a ser a do app** (OkHttp), não a do `sk-net` |
| Filtros (select, texto, tri-estado, grupo, ordenação) e dados de filtro buscados | todas com busca | `filters` no `info` + `filters` em `search` |
| Preferências por fonte | Mangakakalot, Asura, Webtoons, Luscious, MangaFire, Keyoapp | esquema no `info`, valores em cada pedido |
| Pós-processamento de imagem | Mangakakalot (mesclar), Asura (desembaralhar), Webtoons (texto) | `ops` na página, executadas pelo app (Kotlin, `Bitmap`) |
| CDN de reserva | Mangakakalot | `alt_urls` na página; o app tenta em ordem |
| Várias linguagens num módulo | MangaFire, Webtoons | o idioma do pacote vai em todo pedido (`lang`) |
| Requisições em paralelo | MangaFire (páginas de capítulos), Madara | `http_batch` |
| Resolver desafio no WebView | MangaFire (captcha) | `solve_challenge`, por último (ver 5) |
| Criptografia (AES-CBC, MD5, SHA) | Madara (protetor), Asura | crates Rust puros dentro do módulo |
| `getMangaByUrl`, relacionados | todas | ops opcionais `by_url`, `related` |

`memo` já existe no app (`SManga.memo`, `SChapter.memo`), então o ABI só precisa devolvê-lo e recebê-lo.

## 3. Arquitetura

```
fonte (wasm)  = config + sobrescritas        ~10-50 linhas por site de tema
   |
tema (crate)  = sk-theme-{mangathemesia,madara,zeist,keyoapp,mangabox}
   |
SDK           = sk-source-sdk (ABI v2) + sk-html (seletores) + utilidades (datas, regex, url, json)
   |  ABI JSON-sobre-memória
host          = sk-source (wasmi) -> ponte de rede -> RustSource.kt -> OkHttp do app (Cloudflare, cookies, taxa)
                                                  -> interceptor de imagens (ops) -> Bitmap
```

Escolhas, em ordem da escada do ponytail:
- **Reusar a rede do app em vez de um segundo cliente.** Uma interface de retorno (UniFFI) que o Kotlin implementa com o `NetworkHelper`. Ganha Cloudflare, cookies persistentes e limite de taxa de graça, e o `sk-net` vira só reserva.
- **Seletores próprios em vez de `scraper`.** O `selectors` do Servo não tem `:contains`/`:containsOwn`, que os temas usam em dezenas de lugares. Escrever um motor pequeno em cima do DOM do `html5ever` (parser puro Rust, compila para wasm) é menos arriscado do que remendar o seletor por texto.
- **`fancy-regex`** para os `(?<!...)` do Madara; `regex` para o resto.
- **Datas** com `time` + tabela de nomes de mês/palavras relativas por idioma (os temas já listam as palavras).
- **Imagens processadas no app, não no wasm.** Interpretar `Bitmap` em wasmi (sem JIT) seria lento; o módulo só descreve a operação, o Kotlin executa.
- **Mesmo `id` da extensão.** Para a biblioteca e o histórico continuarem ligados, a fonte Rust informa o `id` que a extensão tinha (`MD5("nome/idioma/versionId")`, 8 primeiros bytes). Ao instalar, a fonte Rust passa na frente da extensão de mesmo id.

## 4. Melhorias além da paridade

1. **Um motor por família:** corrigir o tema conserta todos os sites; hoje cada extensão é um APK de ~2 MB que o usuário atualiza separado.
2. **Rede única:** limite de taxa e cookies compartilhados, sem 12 clientes OkHttp.
3. **Requisições em paralelo** (`http_batch`): lista de capítulos do MangaFire e outras paginações deixam de ser sequenciais.
4. **Teste de fumaça por fonte:** cada fonte leva fixtures (HTML real salvo) e roda `cargo test` no PC, e um comando `sk-pack check` bate nos sites de verdade. Quando um site muda, o teste quebra antes do usuário.
5. **Cache de dados de filtro** (gêneros) com validade, em vez de baixar a cada abertura.
6. **Sandbox:** orçamento de instruções e bloqueio de rede privada que as extensões APK não têm (um APK roda com todas as permissões do app).
7. **Atualização assinada e independente** pelo repositório oficial, sem lojas de APK.

## 5. Ordem de trabalho

| Marco | Entrega | Verificação |
|---|---|---|
| A | `sk-html` (seletores jsoup) + fixtures reais | testes no PC contra as páginas das 12 fontes |
| B | ABI v2: ponte de rede do app, POST, filtros, preferências, `ops` de imagem, `lang`, `http_batch` | MangaDex continua funcionando; teste de ida e volta |
| C | Tema MangaBox + **Mangakakalot** (a que está na biblioteca) | no aparelho: biblioteca, busca, leitura com imagens, mesclar |
| D | MangaThemesia (Arena), KeyoappV2 (Asmotoon), Madara (Kissmanga.in), Zeist (Murim) | no aparelho, uma por uma |
| E | ReadComics, Mangademon, MangaFox | idem |
| F | Webtoons, Asura (desembaralhar), Luscious | idem |
| G | MangaFire (VRF, 7 idiomas, preferências), por último o captcha | idem |
| H | Pacotes assinados, publicação no repositório, troca no aparelho pelo mesmo `id` | instalar pela tela de Fontes em Rust |

Risco conhecido: sites mudam e alguns bloqueiam IPs fora de uso de navegador; o teste ao vivo mostra isso fonte por fonte. O captcha do MangaFire depende de OpenCV.js no WebView e fica para o fim; sem ele a fonte funciona enquanto o site não pedir o desafio.
