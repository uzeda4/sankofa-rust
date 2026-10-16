# Publicar uma versão que o atualizador entende

O app só procura atualizações no repositório com que foi compilado. Sem isso o atualizador e o "O que há de novo" ficam desligados (nada aponta para o Komikku).

## Compilar para um repositório
```text
./gradlew :app:assembleRelease -Penable-updater -Psankofa.updateRepo=<dono>/<repositorio> \
          "-Predesign.signing=<signing.properties>"
```
- `-Penable-updater` liga a verificação (e o download automático, que o usuário pode desligar nas configurações).
- `-Psankofa.updateRepo` é o `dono/repositorio` do GitHub que publica os releases.
- `-Psankofa.updateApi=<url>` (opcional, só para testes) troca `https://api.github.com` por um servidor local.

## O release no GitHub
1. **Tag** `vX.Y.Z` (por exemplo `v0.2.0`). O app compara com o `versionName` do APK (`0.2.0`); um sufixo `-1234` do build é ignorado. Suba o `versionName` e o `versionCode` em `app/build.gradle.kts` antes.
2. **Arquivos** (exatamente estes nomes; o app escolhe pelo trecho `-<abi>` e usa o primeiro ABI do aparelho):
   `app-arm64-v8a-release.apk`, `app-armeabi-v7a-release.apk`, `app-x86_64-release.apk`, `app-x86-release.apk` (e `app-universal-release.apk`, usado quando nenhum casa).
3. **Texto do release** = o que aparece em Sobre > O que há de novo e na tela "Nova versão disponível". Markdown simples; `@usuario` vira link. Copie a seção da versão de `CHANGELOG.md`.
4. **Não** marque como rascunho nem pré-lançamento: o app ignora ambos.
5. **Assinatura:** o Android só instala a atualização se o APK for assinado pela mesma chave do app instalado. Use sempre o mesmo `redesign.jks`; trocar de chave exige desinstalar.

## Como testei
Servidor local imitando `GET /repos/<dono>/<repo>/releases/latest` e `/releases`, `adb reverse`, e um build debug com `-Psankofa.updateRepo=test/sankofa -Psankofa.updateApi=http://127.0.0.1:8766`: o app mostrou "Nova versão disponível v0.2.0" com as notas e o botão de download, e "O que há de novo" listou o release. A instalação do APK em si (a janela do Android) não foi percorrida.

## Ainda é do Komikku
Os fluxos de CI de `.github/workflows` são os do projeto original (publicam no repositório e no site dele). Não serviriam num repositório do fork sem revisão; hoje a publicação é manual como acima. Links de ajuda (`komikku-app.github.io/docs/...`) em telas de ajuda, onboarding e Discord RPC também continuam apontando para a documentação dele.
