# Handoff: redesign do fork do Komikku

Protótipo de referência: https://claude.ai/artifact/68EqTYGvNAJydJJ1zQF8hd (23 telas, 4 ondas, design system).
Direção: **cinematográfico guiado pela capa**. Neutro por padrão, cada título tinge a interface, com profundidade 3D e fallback para aparelhos fracos.

## 1. Escopo e premissas

- Reescrita total da camada de apresentação. `domain`, `data`, `source-api`, `source-local` e `core` ficam como estão.
- Verificado no repositório `komikku-app/komikku` (listagem da API do GitHub): módulos `app`, `core`, `core-metadata`, `domain`, `data`, `presentation-core`, `presentation-widget`, `source-api`, `source-local`, `i18n*`; pastas de apresentação `eu/kanade/presentation/{browse,category,components,history,library,libraryUpdateError,manga,more,reader,theme,track,updates,util,webview,...}` e `eu/kanade/tachiyomi/ui/{browse,category,download,history,home,library,main,manga,more,reader,setting,stats,updates,webview,...}`; 21 esquemas de cor em `theme/colorscheme`.
- **Verificado no checkout** (`buildSrc/.../AndroidConfig.kt`): `minSdk 26`, `compileSdk 36`, `targetSdk 36`, JVM 17. `namespace = eu.kanade.tachiyomi`. Atualizador e telemetria só entram com `-Penable-updater` / `-Pinclude-telemetry` (desligados por padrão).
- **Não verificado** (conferir antes de codar): versão do Compose BOM e biblioteca de navegação em uso.
- O `AGENTS.md` do Komikku pede marcadores `// KMK --> ... // KMK <--` em toda alteração de código existente, strings novas só em `i18n-kmk` (`KMR`) e `spotlessApply` + `spotlessCheck` antes de concluir. Manter isso nas partes que ainda recebem cherry-pick.
- Dados do protótipo (títulos, fontes, números) são fictícios.

## 2. Design tokens

### Cor (base neutra)
| Token | Valor | Uso |
|---|---|---|
| `ink-0` | `#0F0E11` | Fundo (Z0) |
| `ink-amoled` | `#000000` | Fundo no modo AMOLED |
| `paper-0` | `#F4F0EA` | Texto principal |
| `text-muted` | `paper-0 @ 72%` | Texto secundário (contraste ≥ 4,5:1 sobre `ink-0`) |
| `surface-1` | `white @ 6%` + borda `white @ 10%` | Cartões (Z1) |
| `surface-glass` | `#18161C @ 62%` + blur 24 + saturação 1,5 + borda `white @ 14%` | Dock, barras do reader, sheets (Z2) |
| `hairline` | `white @ 10–16%` | Bordas |

### Cor guiada pela capa (4 tokens por título)
| Token | Como obter | Uso |
|---|---|---|
| `cover-base` | cor escura dominante da capa, luminância reduzida | Fundo da tela do título |
| `cover-surface` | `cover-base` +12% de luminância | Gradiente lateral |
| `cover-accent` | cor vibrante dominante, garantir contraste ≥ 4,5:1 do texto `ink-0` sobre ela | Botão primário, chips ativos, progresso, brilho da capa |
| `cover-veil` | `cover-accent @ 18%` | Fundo de chip, badge suave |

Acento padrão (sem capa em foco): `#F2A65A`. Paletas de exemplo do protótipo:

| Capa | base | surface | accent |
|---|---|---|---|
| Maré de Cinzas | `#0B1820` | `#1B3A4B` | `#F2A65A` |
| Estação Kitsune 9 | `#150B1C` | `#3B1F4A` | `#FF6B8B` |
| Ronin Neon | `#050F0F` | `#0F2B2A` | `#7CF0C9` |
| Clube do Último Sino | `#0A0E24` | `#1F2A55` | `#8FB4FF` |

Regra de contraste: texto sobre `cover-accent` sempre `ink-0`. Se a razão ficar abaixo de 4,5:1, clarear o accent até passar (o texto é escuro, então só um accent mais claro melhora o contraste). Nunca usar matiz sozinho para distinguir estados.

### Tipografia
| Token | Família | Peso | Tamanho / altura | Uso |
|---|---|---|---|---|
| `display-xl` | Bricolage Grotesque | 800 | 40 / 40, tracking −0,02em | Título de tela |
| `display-lg` | Bricolage Grotesque | 800 | 28–34 / 1,05 | Título de mangá, cabeçalho de sub-tela |
| `title` | Bricolage Grotesque | 700 | 17–22 / 1,1 | Título de cartão |
| `body` | DM Sans | 400/500 | 15,5 / 1,5 | Sinopse, listas |
| `label` | DM Sans | 700 | 13–15 | Botões, chips |
| `eyebrow` | DM Sans | 700 | 12,5, caixa alta, tracking 0,14em | Rótulo de seção |
| `kanji` | Shippori Mincho B1 | 700 | 14–34, `writing-mode: vertical` | Ornamento de capa |

Em Compose: carregar via `FontFamily` com fontes em `res/font` (variable font para Bricolage com eixo de peso). Kanji só como ornamento, não carrega significado.

### Espaço, forma e profundidade
| Token | Valor |
|---|---|
| `space` | 4, 8, 10, 12, 14, 16, 18, 20, 24 dp |
| `gutter` | 16 dp |
| `touch-min` | 44 dp (todos os alvos) |
| `radius-chip` | 22 dp (altura 44) |
| `radius-card` | 18–26 dp |
| `radius-cover` | 12 (mini) / 16 / 18 / 22 (hero) dp |
| `radius-sheet` | 32 dp (topo) |
| `radius-dock` | 26 dp, altura 68, item 20 dp |

| Nível | Superfície | Sombra |
|---|---|---|
| Z0 | fundo com atmosfera da capa | nenhuma |
| Z1 | cartão `surface-1` | `0 10 18 −12 black@70%` |
| Z2 | vidro | `0 20 40 −12 black@60%` |
| Z3 | capa em foco | `0 26 34 −14 cover-accent@60%` |

### Movimento
| Token | Valor |
|---|---|
| `ease-emphasized` | `cubic-bezier(.2,.8,.2,1)` |
| `dur-fast` | 120 ms (entrada do tilt) |
| `dur-base` | 250–350 ms (chips, dock, switches, chrome do reader) |
| `dur-slow` | 480–700 ms (sheets, cover flow, atmosfera, saída do tilt) |
| `tilt-max` | ±11° (grade), ±15° (detalhes) |
| `shimmer` | 1,5 s linear, infinito |

## 3. Componentes e mapeamento para o código do Komikku

| Componente do protótipo | Substitui no Komikku | Observações |
|---|---|---|
| `CoverTheme` (tokens + paleta por capa) | `theme/TachiyomiTheme.kt`, `colorscheme/*` | Manter alguns esquemas fixos como "Cor fixa". Extrair paleta com `androidx.palette`. |
| `CoverCard` (tilt 3D, parallax, brilho, badge) | `library/components`, `browse/*` (grid de mangá) | Camadas: fundo, sol, relevo, kanji, brilho. Ver 4.1. |
| `GlassDock` | navegação em `tachiyomi/ui/home` | 5 itens, item ativo sobe 3 dp e escala 1,04. |
| `GlassBar` (reader top/bottom) | `reader/appbars` | Some em 350 ms, `pointer-events` desligados quando oculta. |
| `GlassSheet` | `components/AdaptiveSheet.kt`, `TabbedDialog.kt` | Sobe 480 ms, scrim 62%. |
| `Chip` / `SegmentedTabs` | `components/Chip.kt`, `TabbedScreen.kt` | Altura 44, ativo = `cover-accent`. |
| `Pill` / `Badge` | `presentation-core/components/Pill.kt`, `Badges.kt` | Contador sobre capa usa `cover-accent`. |
| `SettingRow` + `Switch` | `presentation-core/components/SettingsItems.kt` | Altura mínima 60–64, trilho 52×30. |
| `SectionCard` | `presentation-core/components/SectionCard.kt` | `surface-1` + hairline. |
| `Skeleton` | novo | Brilho 1,5 s; parado com "reduzir animações". |
| `Banner` (progresso da biblioteca) | `components/Banners.kt` | Barra no topo + ação Parar/Atualizar. |
| `UndoToast` | snackbar existente | 4 s, ação "Desfazer". |
| `EmptyState` | `components/EmptyScreen.kt` | Borda tracejada, título + 1 linha. |
| `CoverFlow` | novo | `HorizontalPager` + `graphicsLayer`. |
| `TapZonesPreview` | `reader/settings` | 3 zonas coloridas em 3D. |

## 4. Especificação de movimento (equivalente em Compose)

### 4.1 Tilt 3D da capa
```kotlin
// Esboço. Verificar API do Compose usada no repo.
fun Modifier.tilt3d(state: TiltState, enabled: Boolean) = if (!enabled) this else
    graphicsLayer {
        rotationX = state.rx            // (0.5 - y) * 22
        rotationY = state.ry            // (x - 0.5) * 22
        cameraDistance = 12f * density
        scaleX = if (state.active) 1.05f else 1f
        scaleY = scaleX
    }
```
- Entrada 120 ms, saída 600 ms (`spring` com `dampingRatio = 0.7`).
- Parallax: sol desloca `(−ry·0,9, rx·0,9)`, relevo `(ry·0,6, −rx·0,4)`. Brilho: gradiente radial na posição do ponteiro, `BlendMode.Softlight`.
- No aparelho: ponteiro vira toque/arrasto; giroscópio opcional via `SensorManager` (ligar só com a preferência "Parallax por giroscópio").

| Elemento | Gatilho | Animação | Duração | Easing |
|---|---|---|---|---|
| Capa (grade) | toque/arrasto | tilt + parallax + brilho | 120 ms in / 600 ms out | spring 0,7 |
| Atmosfera da tela | foco em outra capa | `animateColorAsState` dos gradientes | 700 ms | ease |
| Cover flow | trocar item | translateX 82 dp, rotateY ∓36°, translateZ −100 dp, brilho 72% | 600 ms | emphasized |
| Dock | trocar aba | item sobe 3 dp, escala 1,04, fundo `cover-veil` | 300 ms | emphasized |
| Capa → detalhes | abrir título | elemento compartilhado | 400 ms | emphasized |
| Sheet de filtros/reader | abrir/fechar | translateY 105% → 0, scrim 0 → 62% | 480 ms / 350 ms | emphasized |
| Barras do reader | toque na página | opacidade + translateY (−24 / +32 dp) | 350 ms | emphasized |
| Cartão de virada de capítulo | fim do capítulo | rotateY −10°, rotateX 4° | 400 ms | emphasized |
| Barras de gráfico | trocar período | altura | 600 ms | emphasized |
| Skeleton | carregando | faixa de brilho | 1,5 s | linear |
| Remoção de item | tocar remover | translateX 60 dp, escala 0,96, opacidade 0 | 350 ms | emphasized |

### 4.2 Fallback obrigatório
Implementado em `presentation-core/.../theme/MotionPolicy.kt` (`LocalMotionPolicy`, `rememberMotionPolicy`). Tudo desligado (tilt, parallax, vidro) quando qualquer condição for verdadeira:
- `ActivityManager.isLowRamDevice`
- economia de bateria ativa (`PowerManager.isPowerSaveMode`)
- escala de animação do sistema = 0 (`ANIMATOR_DURATION_SCALE`) ou preferência "Reduzir movimento" do app

Em aparelhos normais, tilt e parallax ficam ligados em qualquer API; só o vidro (blur com `RenderEffect`) exige API 31+. Shaders AGSL, quando entrarem, exigem API 33+ e devem ter um campo próprio na política.

Versão simples: troca de cor por fade de 150 ms, capa sem inclinação, vidro vira cor sólida `surface-glass @ 92%`.

## 5. Telas (protótipo → código)

| Onda | Tela do protótipo | Origem no Komikku | Estados a cobrir |
|---|---|---|---|
| 1 | Biblioteca | `presentation/library`, `ui/library` | vazio, carregando, seleção em massa, categoria oculta |
| 1 | Detalhes do mangá | `presentation/manga`, `ui/manga` | sem capítulos, baixando, lido/não lido, favorito |
| 1 | Reader | `presentation/reader`, `ui/reader` | barras ocultas, fim de capítulo, erro de página |
| 1 | Navegação (dock) | `ui/home`, `ui/main` | item ativo, badge de novidades |
| 2 | Explorar | `presentation/browse` | sem fontes, buscando |
| 2 | Fontes e extensões | `presentation/browse` | nenhuma no idioma, atualização disponível |
| 2 | Navegar na fonte | `presentation/browse` | carregando mais, sem resultado, grade/lista |
| 2 | Atualizações | `presentation/updates`, `ui/updates` | atualizando, em dia, grupo expandido |
| 2 | Histórico | `presentation/history`, `ui/history` | vazio, desfazer remoção |
| 2 | Feed | `presentation/browse` (feed) | buscas salvas, fonte sem itens |
| 2 | Migrar fonte | `ui/browse/migration` | sem correspondência, revisar, em andamento |
| 3 | Mais | `presentation/more`, `ui/more` | incógnito ligado |
| 3 | Aparência | `ui/setting` | reduzir movimento, tema fixo |
| 3 | Trackers | `presentation/track` | desconectado, sincronizando |
| 3 | Downloads | `ui/download` | pausado, fila vazia |
| 3 | Categorias | `presentation/category`, `ui/category` | oculta, reordenar |
| 3 | Estatísticas | `ui/stats` | sem dados no período |
| 3 | Backup | `ui/setting` (backup) | criando, falhou, restaurar |
| 3 | Webview | `presentation/webview`, `ui/webview` | verificando, verificado |
| 3 | Erros de atualização | `presentation/libraryUpdateError` | tentar de novo, dispensar |
| 4 | Busca global | `ui/browse/source/globalsearch` | sem resultado, fonte ainda buscando |
| 4 | Configurações do reader | `presentation/reader/settings` | modo, direção, escala, zonas |
| 5 | Detalhes da fonte | `ui/browse/source` (preferências da fonte) | sem login, conectado, desinstalar |
| 5 | Escolher correspondência | `ui/browse/migration` | sem candidato, vários candidatos |
| 5 | Armazenamento | `ui/more/storage` (conferir) | tudo apagado, só lidos |
| 5 | Recomendações | `ui/manga` (recomendações do SY) | sem recomendações, carregando |
| 5 | Primeiros passos | novo (`ui/main`, primeira execução) | pular, voltar, reduzir movimento |
| 5 | Diálogos de confirmação | `presentation/components/AdaptiveSheet.kt` e diálogos existentes | remover, apagar downloads, restaurar |
| 5 | Widgets (4×2, 2×2, 2×1) | `presentation-widget` | sem leitura recente, sem novos |
| 5 | Tela de erro | `presentation/crash` | copiado, compartilhar |

Conferir o nome exato do pacote de migração e de busca global no checkout; os caminhos acima para esses dois são o ponto de partida.

## 6. Acessibilidade

- Todo alvo de toque ≥ 44 dp. Ícone sem texto leva `contentDescription` (ex.: "Voltar", "Fixar Aurora Scans").
- Switches usam semântica de `Switch`/`toggleable` com estado anunciado. Chips ativos usam `selected`.
- Cartões expansíveis (Atualizações) anunciam `expanded`.
- Ordem de foco: cabeçalho, busca, filtros, conteúdo, ação flutuante, dock.
- Tilt e parallax nunca carregam informação. Estado é sempre também texto ou ícone.
- Texto ≥ 4,5:1 (≥ 3:1 acima de 24 sp). Cores que precisam ser distinguidas também diferem em luminância.
- Respeitar `fontScale`: títulos de 40 sp quebram em duas linhas sem cortar; listas crescem em altura.

## 7. Casos de borda

- **Texto longo**: título de cartão em até 2 linhas com elipse; nome de fonte 1 linha; capítulo "Cap. N · título" 1 linha com elipse.
- **Capa sem paleta** (falha ou imagem cinza): usar acento padrão `#F2A65A`.
- **Capa muito clara**: baixar luminância do `cover-base`; o texto continua em `paper-0`.
- **Lista grande**: grade com `LazyVerticalGrid`; tilt só no item sob o toque; sem sombras coloridas fora da janela visível.
- **Sem rede**: Navegar e Busca mostram estado vazio com "Tentar de novo"; Feed mantém o último resultado.
- **Idiomas longos** (alemão, russo): chips crescem em largura e rolam na horizontal.

## 8. Plano de port por módulo

| Fase | Entrega | Módulos | Pronto quando |
|---|---|---|---|
| 0 | Fork e identidade | raiz, `app` | novo `applicationId`, nome, ícone, assinatura, CI; extensões do ecossistema carregam |
| 1 | Tokens e tema | `presentation-core/theme`, `app/.../presentation/theme` | `CoverTheme` com fallback; esquemas antigos ainda selecionáveis |
| 2 | Componentes base | `presentation-core/components`, `presentation/components` | tabela da seção 3 coberta, com previews |
| 3 | Onda 1 | `presentation/{library,manga,reader}`, `ui/home` | fluxo biblioteca → detalhes → reader fecha sem regressão |
| 4 | Onda 2 | `presentation/{browse,updates,history}` | explorar, instalar extensão, ler, migrar |
| 5 | Ondas 3 e 4 | `presentation/{more,track,category,webview,libraryUpdateError}`, `ui/{setting,stats,download}` | paridade de funções com o Komikku |
| 6 | Desempenho e polimento | `macrobenchmark` | metas da seção 9 atingidas em aparelho de referência |

Fase 0, já aplicada em `chore/fork-identity` (sem commit):
- `applicationId` `app.komikku` → `app.komikkuredesign` (debug `.dev`, preview `.beta`, foss `.foss`, benchmark `.benchmark` herdam o sufixo). `namespace` mantido.
- `versionCode 1`, `versionName 0.1.0`.
- `app_name` → "Komikku Redesign".
- Pacote de benchmark atualizado em `StartupBenchmark.kt` e `BaselineProfileGenerator.kt`.
- Remote `origin` renomeado para `upstream`.
- Ainda pendente: ícone, assinatura própria (keystore), `README`/`LICENSE` de atribuição, CI do fork, `google-services` se usar telemetria, e o nome definitivo da marca.

Estado das fases seguintes (branches empilhadas `chore/fork-identity` → `feat/cover-theme-tokens` → `feat/base-components`):
- **Fase 1 feita**: `CoverPalette`, `MotionPolicy`, `Modifier.tilt3d`, ligação em `TachiyomiTheme`, testes em `presentation-core`.
- **Fase 2 em andamento**: feitos `CoverChip`, `CoverBadge`, `SectionLabel`, `CoverProgress`, `GlassSurface` (com blur via Haze quando há `LocalGlassBackdrop` e a política permite), `Modifier.shimmer()` (`presentation-core/.../components/CoverComponents.kt`), fontes Bricolage Grotesque e DM Sans aplicadas ao app todo (`RedesignTypography.kt`), tilt na grade da biblioteca e do browse (`CommonMangaItem.kt`) e item ativo do dock com accent e elevação (`HomeScreen.kt`). Faltam: `CoverCard` completo (parallax em camadas, brilho), dock flutuante (exige repensar os insets de cada aba), `SettingRow`, `Banner`, `UndoToast`, `CoverFlow`, e ligar `GlassSurface` com um `HazeState` real nas barras do reader.
- Verificado em aparelho (aparelho Android 16), tema Redesign: Biblioteca (cabeçalho, cartão "Continuar lendo", chips, dock flutuante), Detalhes do mangá (título em display, capa maior com tilt), Reader (barras arredondadas), Navegar (cabeçalho, chips, fontes em cartões), Histórico (cabeçalho e rótulos de seção) e Atualizações (estado vazio com cabeçalho). Mais e Configurações (preferências como cartões arredondados, vale para todas as telas de ajustes). Não conferidos: tilt ao toque, Atualizações com itens, Extensões, Feed e Migrar.
- Modo claro conferido no aparelho (Biblioteca e Detalhes). Paisagem conferida na Biblioteca: o dock agora para em 560 dp e o cartão "Continuar lendo" em 640 dp. Tablet (`isTabletUi`, com `NavigationRail`) ainda não conferido.
- Fora do que foi feito: dock sobre o conteúdo com blur real (exige rever o espaço inferior de cada aba), cover flow, elemento compartilhado capa → detalhes, cartão de virada de capítulo, widgets e onboarding redesenhados.
- Aviso do Android 16 em builds de debug: libs nativas herdadas do upstream (`libsqliteJni`, `libimagedecoder`, `libarchive-jni`, `libquickjs`...) não estão alinhadas a 16 KB. Não é da nossa mudança; resolver antes de publicar na Play Store.

Regras do fork:
- Hard fork da UI. Só trazer do upstream correções de `domain`, `data`, `source-api` e segurança, por cherry-pick.
- Manter o contrato de `source-api` e o formato de repositório de extensões.
- Licença Apache 2.0: manter avisos e atribuição aos projetos originais.
- Textos: reaproveitar `i18n` e adicionar strings novas em `i18n-kmk`/módulo próprio, sem duplicar chaves.

## 9. Critérios de aceite

- Rolagem da biblioteca e grade do Explorar sem queda perceptível: p95 de tempo de frame ≤ 16 ms em aparelho de referência (60 Hz), medido com `macrobenchmark`.
- Abertura do reader até primeira página: sem regressão em relação ao Komikku base.
- Com "Reduzir movimento" ligado, nenhuma tela usa rotação, parallax ou shader.
- Teste em API 26, 33 e 34, claro/escuro/AMOLED, fonte 130% e idioma longo.
- Nenhuma tela perde função existente no Komikku (lista da seção 5 + telas não prototipadas, listadas abaixo).

## 10. Fora do protótipo (ainda a desenhar)

As telas da onda 5 acima cobrem o que estava pendente. Ainda sem desenho: as subtelas profundas de Configurações do Komikku (leitura avançada, rede, segurança com bloqueio por biometria, parental/conteúdo), notificações do sistema e o seletor de pasta. Seguir os mesmos tokens e padrões de estado.

Widgets: Android não garante blur nem 3D. Usar cor sólida derivada da capa, cantos de 28–32 dp e a mesma tipografia.
