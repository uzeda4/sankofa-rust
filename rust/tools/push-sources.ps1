# Copies the built source modules (.wasm) into a debug build of the app on the connected device, replacing what is
# there, and restarts the app. Debug builds load unsigned modules; release builds only load signed packages.
#   .\tools\push-sources.ps1                      # every source crate under rust/sources except bench
#   .\tools\push-sources.ps1 mangakakalot arenascans
param([Parameter(ValueFromRemainingArguments)][string[]]$Names = @())
$Package = if ($env:SK_APP) { $env:SK_APP } else { 'app.sankofa.rust.dev' }
$ErrorActionPreference = 'Stop'
$adb = "$env:LOCALAPPDATA\Android\Sdk\platform-tools\adb.exe"
$release = Resolve-Path "$PSScriptRoot\..\target\wasm32-unknown-unknown\release"
$files = Get-ChildItem "$release\sk_source_*.wasm" | Where-Object { $_.BaseName -ne 'sk_source_bench' }
if ($Names) { $files = $files | Where-Object { $Names -contains ($_.BaseName -replace '^sk_source_', '') } }
if (-not $files) { throw "no modules built for: $Names (cargo build -p sk-source-<name> --release --target wasm32-unknown-unknown)" }

& $adb shell "run-as $Package sh -c 'mkdir -p files/rust-sources && rm -f files/rust-sources/*'"
foreach ($f in $files) {
    $name = ($f.BaseName -replace '^sk_source_', '') + '.wasm'
    & $adb push $f.FullName "/data/local/tmp/$name" | Out-Null
    & $adb shell "run-as $Package cp /data/local/tmp/$name files/rust-sources/$name && rm /data/local/tmp/$name"
    "pushed $name ($([math]::Round($f.Length / 1KB)) KB)"
}
& $adb shell am force-stop $Package
& $adb shell monkey -p $Package -c android.intent.category.LAUNCHER 1 | Out-Null
