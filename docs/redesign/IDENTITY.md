# Sankofa — identidade visual

## Nome e conceito
**Sankofa** é um termo Akan (Gana) geralmente traduzido como "volte e busque": aprender com o que ficou para trás para seguir adiante. Funciona para um leitor de mangá: voltar ao capítulo, retomar a leitura, redescobrir um título.

Atribuição: o conceito e o provérbio pertencem à cultura Akan e à tradição dos símbolos Adinkra. O app **não reproduz** nenhuma arte Adinkra existente; a marca é um desenho original inspirado na forma do pássaro que olha para trás com um ovo no bico. Manter o crédito ao citar o nome em materiais públicos (store, README, site).

## Marca
- Pássaro de perfil, pescoço curvado para trás, ovo no bico, pés firmes no chão.
- Leitura em 48 px: silhueta clara, ovo âmbar como único ponto de cor.
- Fonte do desenho: `app/src/main/res/drawable/ic_launcher_foreground.xml` (viewport 108, dentro da zona segura de 66 dp).

| Variante | Arquivo | Uso |
|---|---|---|
| Ícone adaptativo | `ic_launcher_background.xml` + `ic_launcher_foreground.xml` | launcher |
| Monocromático | `ic_launcher_monochrome.xml` | ícone temático Android 13+ (ovo vira anel) |
| Splash / marca compacta | `ic_redesign_mark.xml`, `ic_splash.xml` (+ `-night`) | splash, telas vazias |

## Cores
| Token | Hex | Papel |
|---|---|---|
| Ink | `#0F0E11` | fundo escuro, pássaro sobre âmbar |
| Paper | `#F4F0EA` | fundo claro, pássaro sobre ink |
| Âmbar (acento) | `#F2A65A` | ovo/sol, acento padrão do app |

Regra: o ovo é sempre a única área em cor de acento; sobre a capa de um título o acento passa a ser o da capa (ver `CoverPalette`).

## Tipografia
- Marca/títulos: Bricolage Grotesque (já embutida, OFL).
- Texto: DM Sans (OFL).
- Wordmark: "Sankofa" em Bricolage Grotesque peso 700, caixa mista, espaçamento −1%.

## Uso
- Área livre ao redor do ícone: metade da largura do ovo.
- Não girar, não espelhar (o olhar para trás é o sentido do nome), não trocar a cor do ovo.
- Fundo mínimo de contraste 4,5:1 entre pássaro e fundo.

## Identificadores
- `app_name`: Sankofa
- `applicationId`: `app.sankofa` (debug `.dev`, preview `.beta`, foss `.foss`, benchmark `.benchmark`). Instala como app separado de builds anteriores `app.komikkuredesign`.
