# Marco 0 da reescrita em Rust: resultado do spike de UI

Data: 2026-10-15 · Aparelho: Adreno 616 (Vulkan), Android arm64 · Crate: `rust/crates/sk-ui-spike` (egui 0.36 + wgpu, `NativeActivity`)

## O que foi medido
| Teste | Resultado |
|---|---|
| Build para `aarch64-linux-android` (release) | `libsk_ui_spike.so` de 10,7 MB, APK de 10,7 MB |
| Renderização | Vulkan (Adreno), grade de 3 colunas, 40 linhas |
| Quadros | 16,5 a 16,9 ms em média (60 fps com vsync); pior quadro 29 a 47 ms (abertura e teclado) |
| Tilt 3D | Projeção em perspectiva própria (`tilt.rs`, 2 testes no host); a capa tocada inclina e volta |
| Teclado virtual | Abre ao tocar o campo; `input text` chegou ao campo (`sankofa`) |

## O que ainda NÃO foi medido
- Capas com imagem de verdade (decodificação, textura, cache); o spike usa retângulos coloridos.
- IME com composição: acentos (ç, ã), CJK e autocorreção. Só texto ASCII foi testado.
- Acessibilidade (TalkBack) e seleção de texto.
- Scroll com imagens; consumo de bateria e memória.
- Blur/vidro, sombras e elemento compartilhado (não existem prontos no egui).

## Leitura
- Há caminho: o egui/wgpu roda em Android sem Java, com teclado e 60 fps na grade simples.
- O custo do design Sankofa está nos efeitos (vidro, transição compartilhada, tilt com textura), que o egui não traz; vão exigir mesh e shaders próprios sobre wgpu.
- Gatilhos do plano B (núcleo Rust + UI Kotlin) continuam valendo: IME com composição e acessibilidade são os pontos a testar antes do marco 2.

## Ferramentas (Windows)
- Rust 1.99 via `rustup-init` com checksum conferido; toolchain `stable-x86_64-pc-windows-gnu` (sem MSVC). O `dlltool.exe` vem do NDK (`llvm-dlltool.exe`, copiado para `~/.cargo/bin`).
- `rust/tools/ndk-env.ps1` configura o linker do NDK, o alinhamento de 16 KB e o remapeamento de caminhos locais por variáveis de ambiente (nada específico da máquina fica no repositório).
- `rust/tools/package-spike.ps1` empacota o `.so` em APK (aapt2 + zipalign + apksigner com a debug keystore).
- `cargo-apk` não compilou neste host (falta `dlltool` para a árvore do `windows-sys` em tempo de instalação); por isso o script próprio.

## sk-db: leitura do banco real (2026-10-16)
`rust/crates/sk-db` (rusqlite com SQLite embutido, compilado pelo clang do NDK) abre o `tachiyomi.db` do app Kotlin e lê a biblioteca.

| Verificação | Resultado |
|---|---|
| Banco usado | Cópia do `tachiyomi.db` (+ `-wal`, `-shm`) do app debug, via `run-as`; SQLite puro, sem criptografia |
| `PRAGMA user_version` | 48 (migrações aplicadas pelo SQLDelight) |
| Biblioteca | 4 favoritos, os mesmos títulos do app |
| Capítulos | Wind Mage 0/31, Hwarim 0/16 (lidos/total), iguais ao que o app mostra |
| Testes | 2 passam no aparelho: um com banco em memória e um contra o banco real |

- Os testes de `sk-db` rodam **no aparelho**: o host só tem toolchain GNU sem compilador C. `rust/tools/test-on-device.ps1 -Crate sk-db -DbFile <tachiyomi.db>` compila, empurra, roda e apaga os arquivos de `/data/local/tmp/sk`.
- O banco do Kotlin é dono do schema; `sk-db` só lê por enquanto. Migrações em Rust vêm depois, com o app Kotlin ainda sendo a referência.
- Cópias de banco ficam em `rust/target/testdata/` (ignorado pelo git); contêm dados de leitura do usuário e não devem ser commitadas.

## sk-net: HTTPS no aparelho (2026-10-16)
`rust/crates/sk-net` (ureq 3 + rustls/ring, raízes Mozilla embutidas, gzip/brotli) baixa páginas pelo telefone.

| Teste (no aparelho, via Wi-Fi) | Resultado |
|---|---|
| `https://example.com/` | 577 bytes, contém "Example Domain" |
| Página inexistente | `Error::Status(404)` |
| `expired.badssl.com`, `self-signed.badssl.com`, `wrong.host.badssl.com` | os três rejeitados pela validação TLS (data vencida, emissor desconhecido, nome errado) |
| `https://www.mangakakalot.gg/` (a fonte da biblioteca) | 242 KB baixados e decodificados, sem desafio do Cloudflare nesta rede |

Decisões:
- **ureq em vez de reqwest.** O reqwest 0.13 sempre usa `rustls-platform-verifier`, que no Android precisa de JNI e de uma classe Kotlin no APK; isso quebra o app 100% nativo (`hasCode=false`) e não roda em binário de teste via adb. O ureq usa raízes embutidas, é bloqueante e traz bem menos dependências. Chamadas assíncronas ficam em threads de trabalho.
- **Limitação conhecida** (marcada com `ponytail:` no código): CAs instaladas pelo usuário e proxies corporativos não são confiados.
- **Ainda não tem:** cookies persistentes, DoH, cache em disco e limitação de taxa (o Kotlin tem os quatro em `NetworkHelper`/OkHttp). Entram no marco 1.
- **Cloudflare:** a página abriu sem desafio, mas fontes protegidas vão exigir o WebView via `sk-android`.

## Capas reais na grade (2026-10-16)
Spike com 56 capas WebP da fonte da biblioteca (mangakakalot.gg): listagem pelo `sk-net`, download em 4 threads, decodificação com o crate `image` (WebP/JPEG/PNG puros em Rust), miniatura de 360 px e textura wgpu por capa; tilt 3D com a textura mapeada em um quad em perspectiva.

| Medida (Adreno 616, 60 Hz) | Resultado |
|---|---|
| Capas exibidas | 56/56; a primeira em 0,1 s e todas em 0,5 s (arquivos locais; ver abaixo) |
| Decodificação + miniatura | média de 28 ms por capa, em threads de fundo |
| Quadros parado | p50 16,6 ms, p95 ~19 ms, máximo ~22 ms |
| Quadros rolando (20 arrastes seguidos) | p50 16,6 ms, p95 18,5–19,3 ms, **p99 31–37 ms, máximo 37–52 ms**; 3 a 5 quadros acima de 20 ms a cada 120 |
| Memória | PSS total 83 MB, 23 MB de gráficos para as 56 texturas |
| Tilt | funciona com textura (mapeamento afim de 2 triângulos; ver `ponytail:` no código) |

Leitura: a grade com imagens reais roda a 60 fps na maior parte do tempo, mas a rolagem rápida tem 2–4% de quadros de 30–50 ms. O custo vem do egui (layout e tesselação por quadro) e de uma textura por capa (uma chamada de desenho cada). Um atlas de texturas e layout em cache são o caminho para tirar esses engasgos; ainda não foi feito.

### Descobertas que mudam o plano
1. **CDN de capas exige `Referer`.** Sem ele a resposta é 403 (testado no aparelho). O `sk-net` ganhou `get_bytes_with(url, headers)`; o contrato de fonte em Rust precisa de cabeçalhos por pedido.
2. **O aparelho bloqueou a rede do spike.** `dumpsys netpolicy` mostra `UID=10213 policy=REJECT_ALL` e "Restricted networking mode: true" para o app novo; o app Kotlin tem rede. Dentro do spike o DNS falha ("No address associated with hostname"), embora o mesmo binário de rede funcione pelo `adb shell`. Eu não mudo configurações do sistema; para este teste as capas foram baixadas no PC (com Referer), empurradas para a pasta do próprio app e apagadas depois. **O app Rust definitivo precisa ter a rede liberada nas configurações do telefone** (Configurações > Apps > o app > dados/rede), senão não baixa nada.
3. **INTERNET** precisa estar no manifesto (o spike tinha esquecido).

### Ainda não medido
IME com acento (ç, ã) e acessibilidade (TalkBack); capas baixadas pelo app (bloqueado pelo item 2); bateria.

## IME e acessibilidade (2026-10-16): os dois gatilhos do plano B foram atingidos
Teste no mesmo spike (egui 0.36 + winit 0.30 + `NativeActivity`), aparelho com Gboard.

| Teste | Resultado |
|---|---|
| Letras ASCII | funciona |
| Caractere acentuado por toque longo (`ç`) | **chega ao campo** (como tecla com caractere, ex.: `ça`) |
| Tipo de entrada que o app declara ao teclado (`dumpsys input_method`) | `inputType=0x0` (TYPE_NULL): modo de eventos de tecla, **sem `InputConnection` de texto** |
| Sugestões, autocorreção, predição | **ausentes** (a barra de sugestões nem aparece) |
| IMEs com composição (japonês, coreano, chinês, e a digitação por deslize) | **não funcionam**: dependem de texto em composição (preedit), que o winit 0.30 no Android não entrega |
| Acessibilidade: árvore de nós vista por serviços como o TalkBack (`uiautomator dump`) | spike: **9 nós, nenhum do conteúdo do egui** (só a moldura da Activity). App Kotlin: **140 nós, 21 com texto, 3 com descrição** |

Consequências:
- Uma tela em Rust desenhada direto numa superfície GL é invisível para o TalkBack: não há árvore de acessibilidade. O egui tem integração com AccessKit, mas o adaptador de Android exige código Java/Kotlin dentro do APK.
- Campo de texto em Rust puro serve para ASCII e caracteres acentuados por tecla, mas quebra a busca em japonês/coreano/chinês, que é central num leitor de mangá, e perde autocorreção e predição.
- Consertar exige, em qualquer toolkit Rust sobre `NativeActivity`, uma camada Kotlin/Java de `InputConnection` e `AccessibilityNodeProvider`: o app deixa de ser "100% Rust" e vira um híbrido com a parte mais difícil (texto e acessibilidade) fora do Rust.
- Esses dois resultados mais os engasgos de rolagem são os gatilhos (1) e (2) do plano B definido em `HANDOFF`/plano da reescrita.
