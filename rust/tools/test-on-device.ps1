# Cross-compiles `cargo test -p <crate>` for arm64 and runs it on the connected device.
# Optional: -DbFile <path to a tachiyomi.db copy (+ -wal/-shm)> is pushed and exposed as SK_TEST_DB.
# Everything pushed to /data/local/tmp/sk is deleted afterwards.
param([Parameter(Mandatory)][string]$Crate, [string]$DbFile)
$ErrorActionPreference = 'Continue'
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
$adb = "$env:LOCALAPPDATA\Android\Sdk\platform-tools\adb.exe"
Push-Location "$PSScriptRoot\.."
try {
    $json = cargo test -p $Crate --target aarch64-linux-android --no-run --message-format json 2>$null
    $exe = ($json | ConvertFrom-Json | Where-Object { $_.executable -and $_.profile.test } | Select-Object -Last 1).executable
    & $adb shell "mkdir -p /data/local/tmp/sk" | Out-Null
    & $adb push $exe /data/local/tmp/sk/t | Out-Null
    $env_ = "SK_TEST_NET=1 "
    if ($DbFile) {
        foreach ($s in '', '-wal', '-shm') { if (Test-Path "$DbFile$s") { & $adb push "$DbFile$s" "/data/local/tmp/sk/tachiyomi.db$s" | Out-Null } }
        $env_ += "SK_TEST_DB=/data/local/tmp/sk/tachiyomi.db "
    }
    & $adb shell "chmod +x /data/local/tmp/sk/t && cd /data/local/tmp/sk && ${env_}./t --nocapture 2>&1"
} finally {
    & $adb shell "rm -rf /data/local/tmp/sk" | Out-Null
    Pop-Location
}



