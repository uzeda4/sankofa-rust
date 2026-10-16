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
- `rust/.cargo/config.toml` aponta o linker para o clang do NDK 29.
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
