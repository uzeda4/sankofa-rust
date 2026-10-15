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
