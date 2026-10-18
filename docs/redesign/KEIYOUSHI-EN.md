# Fontes em inglês do Keiyoushi -> Rust

Conversão em lote das fontes em inglês do repositório `keiyoushi/extensions` (índice `index.pb`, 17/10/2026: 1434 extensões, 2422 fontes, **493 em inglês** em 492 extensões) para fontes em Rust, publicadas no repositório oficial.

## Resultado

- **88 fontes novas** no repositório (15 delas atrás de Cloudflare: o teste no PC não passa pelo desafio, o app passa).
- 23 geradas mas **deixadas de fora**: o site está fora do ar ou a página não é mais lida (lista abaixo).
- As outras **não foram convertidas**: cada uma precisa de porte manual (tabela abaixo).

## Por família de site

| Família (tema) | Extensões EN | Publicadas | Fora (site morto/quebrado) | Sem conversão automática |
|---|---:|---:|---:|---|
| madara | 104 | 64 | 14 | 26 com código próprio além do tema |
| mangathemesia | 24 | 8 | 8 | 8 com código próprio além do tema |
| keyoapp | 17 | 0 | 0 | tema ainda sem motor em Rust |
| mangacatalog | 13 | 8 | 1 | 4 com código próprio além do tema |
| mangahub | 11 | 0 | 0 | tema ainda sem motor em Rust |
| yiimanga | 10 | 8 | 0 | 2 com código próprio além do tema |
| galleryadults | 9 | 0 | 0 | tema ainda sem motor em Rust |
| iken | 8 | 0 | 0 | tema ainda sem motor em Rust |
| yiimadara | 7 | 0 | 0 | tema ainda sem motor em Rust |
| vinetheme | 6 | 0 | 0 | tema ainda sem motor em Rust |
| guya | 4 | 0 | 0 | tema ainda sem motor em Rust |
| manga18 | 3 | 0 | 0 | tema ainda sem motor em Rust |
| mangabox | 3 | 0 | 0 | tema ainda sem motor em Rust |
| hentaihand | 2 | 0 | 0 | tema ainda sem motor em Rust |
| mangadventure | 2 | 0 | 0 | tema ainda sem motor em Rust |
| bakkin | 2 | 0 | 0 | tema ainda sem motor em Rust |
| foolslide | 2 | 0 | 0 | tema ainda sem motor em Rust |
| eromuse | 2 | 0 | 0 | tema ainda sem motor em Rust |
| ezmanhwa | 2 | 0 | 0 | tema ainda sem motor em Rust |
| heancms | 2 | 0 | 0 | tema ainda sem motor em Rust |
| mangak | 2 | 0 | 0 | tema ainda sem motor em Rust |
| monochrome | 2 | 0 | 0 | tema ainda sem motor em Rust |
| goda | 1 | 0 | 0 | tema ainda sem motor em Rust |
| hiper | 1 | 0 | 0 | tema ainda sem motor em Rust |
| initmanga | 1 | 0 | 0 | tema ainda sem motor em Rust |
| mangareader | 1 | 0 | 0 | tema ainda sem motor em Rust |
| liliana | 1 | 0 | 0 | tema ainda sem motor em Rust |
| manhwaz | 1 | 0 | 0 | tema ainda sem motor em Rust |
| zeistmanga | 1 | 0 | 0 | tema ainda sem motor em Rust |
| mmrcms | 1 | 0 | 0 | tema ainda sem motor em Rust |
| mangataro | 1 | 0 | 0 | tema ainda sem motor em Rust |
| pam | 1 | 0 | 0 | tema ainda sem motor em Rust |
| wpcomics | 1 | 0 | 0 | tema ainda sem motor em Rust |
| (código próprio, sem tema) | 231 | 0 | 0 | uma a uma; as 12 já portadas à mão são as do aparelho |

As famílias sem motor (iken, vinetheme, mangahub, yiimadara, keyoapp antigo, galleryadults...) são o próximo passo natural: escrever o tema uma vez converte todas as fontes dele.

## Como funciona

- `rust/tools/port/port.py` lê o índice e o código Kotlin de cada extensão e só traduz as que são **configuração pura** de um tema (constantes, seletores, limite de taxa). Qualquer lógica própria (`override fun` com corpo, classes auxiliares, `when (lang)`) manda a fonte para a lista manual, com o motivo.
- Cada fonte vira um crate de ~20 linhas em `rust/sources/<nome>/` sobre o tema (`sk-theme-madara`, `-mangathemesia`, `-mangacatalog`, `-yiimanga`), com o ícone da extensão.
- `rust/crates/sk-check` abre o módulo contra o **site real** (popular, detalhes, capítulos, páginas e busca) e classifica: `ok`, `cloudflare`, `dead` (fora do ar) ou `broken`. `rust/tools/port/collect.py` publica `ok` e `cloudflare`.
- `rust/tools/publish-sources.ps1` empacota e assina tudo (módulos do perfil `wasm`: `cargo build --profile wasm --target wasm32-unknown-unknown -p sk-source-<nome>`).

```text
python rust/tools/port/port.py --index index.pb --kei <extensions-source> --sources rust/sources --theme madara --report madara.json
cargo build --profile wasm --target wasm32-unknown-unknown -p sk-source-<nome> ...
rust/target/release/sk-check <modulo.wasm> en            # uma linha JSON por fonte
python rust/tools/port/collect.py --reports *.json --check check.jsonl --out rust/tools/ported-sources.json
```

## Publicadas

**madara** (64): AP Comics, AllPornComic, AllPornComic.io, Anisa Scans, Borat Scans (CF), Bun Manga (CF), Cocomic, Cucumber Manga, DragonTea (CF), Ero18x (CF), GakaMangas, GalaxyDegenScans, Gourmet Scans (CF), HM2D, Hentai1.io, HentaiXComic, HentaiXYuri, Hunlight Comics, Jinmangas, KSGroupScans, KaliScan.org (unoriginal), KokoMangas, LHTranslation, LinkManga, Manga 18x (CF), Manga District, Manga Eclipse, Manga18Free, MangaManiacs, MangaOwl.io (unoriginal), MangaPark.tv (unoriginal), MangaYY, Mangaforfree.com, Mangafree, Mangasushi, Manhua Plus, Manhua Zonghe (CF), ManhuaHot, ManhuaTop, ManhuaUS (CF), Manhwa Comics, Manhwa Reads, Manhwa18.today (CF), Manhwa68 (CF), ManhwaDen, ManhwaHub.to, Manhwatop, OctopusManga, Orchisasia, Paritehaber, Petrotechsociety, Pornhwaz, S2Manga, SeraphicDeviltry, Sleepy Translations, ToonGod (CF), Toonizy, TopManhua.net (CF), TritiniaScans, WebtoonScan (CF), WuxiaWorld, YaoiScan (CF), Yaoihub, Zinmanga

**mangathemesia** (8): Akaza Scans, Aster Scans, Hentai Desi, King of Shojo, Lagoon Scans, Manga Trend, Raven Scans, Wildtoon (CF)

**mangacatalog** (8): Read Chainsaw Man Manga Online, Read Fairy Tail & Edens Zero Manga Online, Read Jujutsu Kaisen Manga Online, Read Kingdom Manga Online, Read Nanatsu no Taizai 7 Deadly Sins Manga Online, Read One Piece Manga Online, Read Solo Leveling Manga Manhwa Online, Read Tokyo Ghoul Re & Tokyo Ghoul Manga Online

**yiimanga** (8): 4PornComic, Hentai20 (Unoriginal), Manhwa18TX, ManhwaGalaxy, ManhwaOnline, Manhwaddict, MyReadingComic, YaoiScan (Unoriginal)

## Geradas e deixadas de fora

| Fonte | Tema | Motivo |
|---|---|---|
| Aqua Manga | madara | HTTP 404 for https://aquareader.org/manga/?m_orderby=views |
| Athrea Scans | mangathemesia | https://athreascans.com/manga/?page=1&order=popular: curl: curl: (28) Operation timed out  |
| Aunt Manhwa | madara | the page has no title (the site changed?) |
| Daily Manhwa | madara | HTTP 525 for https://www.dailymanhwa.com/wp-admin/admin-ajax.php |
| Decadence Scans | madara | https://reader.decadencescans.com/wp-admin/admin-ajax.php: curl: curl: (28) Operation time |
| Eva Scans | mangathemesia | popular is empty |
| Galaxy Manga | mangathemesia | popular is empty |
| GingeRTooN | madara | popular is empty |
| Hades Scans | mangathemesia | popular is empty |
| Madara Scans | mangathemesia | popular is empty |
| Mahouirexnohentaikarte | madara | HTTP 404 for https://mahouirexnohentaikarte.com/wp-admin/admin-ajax.php |
| Manga Kiss | madara | HTTP 522 for https://mangakiss.org/wp-admin/admin-ajax.php |
| MangaRead.org | madara | source trapped: all fuel consumed by WebAssembly |
| Manhuanext | madara | no chapters |
| ManhwaGet | madara | https://manhwaget.com/wp-admin/admin-ajax.php: curl: curl: (35) schannel: next InitializeS |
| ManhwaManhua | madara | https://manhwamanhua.com/wp-admin/admin-ajax.php: curl: curl: (35) schannel: failed to rec |
| Milftoon | madara | popular is empty |
| NexComic | mangathemesia | HTTP 404 for https://nexcomic.com/manga/?page=1&order=popular |
| Razure | mangathemesia | popular is empty |
| Read Black Clover Manga Online | mangacatalog | https://ww10.readblackclover.com/manga/black-clover/: curl: curl: (6) Could not resolve ho |
| Rizz Comic (unoriginal) | mangathemesia | HTTP 500 for https://rizzcomic.com/manga/?page=1&order=popular |
| Spmanhwa | madara | https://spmanhwa.online/wp-admin/admin-ajax.php: curl: curl: (56) Recv failure: Connection |
| Top Manhua | madara | HTTP 400 |

## Limites conhecidos

- Verificado no PC com `curl`: um site atrás de Cloudflare aparece como `cloudflare` e só foi conferido no aparelho para uma amostra.
- A busca e os filtros usam os do tema; filtros que o site da extensão sobrescrevia com código próprio não vieram.
- Preferências por fonte (domínio, imagens) existem só onde o tema as tem.
