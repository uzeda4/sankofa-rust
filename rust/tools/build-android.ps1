# Builds the Rust core (crates/sk-api) for Android and drops libsk_api.so into the app's jniLibs,
# then regenerates the Kotlin bindings (app/src/main/java/uniffi/sk_api/sk_api.kt).
# Only arm64-v8a is built by default; add other ABIs once their rustup targets are installed.
param([string[]]$Abis = @('arm64-v8a'))
$ErrorActionPreference = 'Continue'
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
$triples = @{ 'arm64-v8a' = 'aarch64-linux-android'; 'armeabi-v7a' = 'armv7-linux-androideabi'; 'x86_64' = 'x86_64-linux-android'; 'x86' = 'i686-linux-android' }
$root = Resolve-Path "$PSScriptRoot\.."
Push-Location $root
try {
    foreach ($abi in $Abis) {
        $t = $triples[$abi]
        cargo build --release --target $t -p sk-api
        $dst = "$root\..\app\src\main\jniLibs\$abi"
        New-Item -ItemType Directory -Force $dst | Out-Null
        Copy-Item "$root\target\$t\release\libsk_api.so" "$dst\libsk_api.so" -Force
        "ABI ${abi}: $((Get-Item "$dst\libsk_api.so").Length / 1MB) MB"
    }
    # UniFFI reads its metadata from an unstripped (debug) build; it is the same for every ABI.
    $t0 = $triples[$Abis[0]]
    cargo build --target $t0 -p sk-api
    cargo run -q -p uniffi-bindgen -- generate --library "$root\target\$t0\debug\libsk_api.so" --language kotlin --out-dir "$root\..\app\src\main\java" --no-format
} finally { Pop-Location }
