# Packages target/aarch64-linux-android/release/libsk_ui_spike.so into a debug-signed APK.
# Needs the Android SDK (build-tools + a platform) and the debug keystore from ~/.android.
param([string]$Out = "$PSScriptRoot\..\target\sk-ui-spike.apk")
$ErrorActionPreference = 'Stop'
$sdk = "$env:LOCALAPPDATA\Android\Sdk"
$bt = Get-ChildItem "$sdk\build-tools" | Sort-Object { [version]($_.Name -replace '-.*','') } | Select-Object -Last 1
$jar = Get-ChildItem "$sdk\platforms" -Filter android.jar -Recurse | Sort-Object FullName | Select-Object -Last 1
$so = "$PSScriptRoot\..\target\aarch64-linux-android\release\libsk_ui_spike.so"
$tmp = Join-Path $env:TEMP "sk-spike-apk"; Remove-Item $tmp -Recurse -Force -ErrorAction SilentlyContinue; New-Item -ItemType Directory $tmp | Out-Null

& "$($bt.FullName)\aapt2.exe" link -o "$tmp\base.apk" --manifest "$PSScriptRoot\AndroidManifest.spike.xml" -I $jar.FullName
Add-Type -AssemblyName System.IO.Compression, System.IO.Compression.FileSystem
$zip = [System.IO.Compression.ZipFile]::Open("$tmp\base.apk", 'Update')
[void][System.IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip, (Resolve-Path $so).Path, 'lib/arm64-v8a/libsk_ui_spike.so', 'NoCompression')
$zip.Dispose()
& "$($bt.FullName)\zipalign.exe" -p -f 4 "$tmp\base.apk" "$tmp\aligned.apk"
& "$($bt.FullName)\apksigner.bat" sign --ks "$env:USERPROFILE\.android\debug.keystore" --ks-pass pass:android --key-pass pass:android --out $Out "$tmp\aligned.apk"
"APK: $((Resolve-Path $Out).Path)"
