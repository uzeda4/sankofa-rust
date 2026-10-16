# Dot-source this from the other scripts: `. "$PSScriptRoot\ndk-env.ps1"`.
# Sets the environment cargo needs to cross-build for Android on Windows, so nothing machine-specific lives in the
# repository (no linker paths, and no local paths baked into the built libraries).
#
#   NDK        Android NDK root; defaults to the newest one in the SDK (ANDROID_HOME or %LOCALAPPDATA%\Android\Sdk).
#   Rust path  every local path that would end up inside a .so (panic messages, debug info) is remapped.
param([string]$Ndk = '')

if (-not $Ndk) {
    $sdk = if ($env:ANDROID_HOME) { $env:ANDROID_HOME } else { "$env:LOCALAPPDATA\Android\Sdk" }
    $Ndk = (Get-ChildItem "$sdk\ndk" -Directory | Sort-Object { [version]$_.Name } | Select-Object -Last 1).FullName
}
$bin = "$Ndk\toolchains\llvm\prebuilt\windows-x86_64\bin" -replace '\\', '/'
$rustRoot = (Resolve-Path "$PSScriptRoot\..").Path

$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
$env:CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER = "$bin/aarch64-linux-android26-clang.cmd"
$env:CC_aarch64_linux_android = "$bin/aarch64-linux-android26-clang.cmd"
$env:AR_aarch64_linux_android = "$bin/llvm-ar.exe"

# Extra cargo arguments for Android targets only (host build scripts must not get the Android linker flags).
# TOML literal strings ('...') keep Windows paths, including ones with spaces, intact.
$cargoRustflags = @(
    "'-C'", "'link-arg=-Wl,-z,max-page-size=16384'",   # 16 KB page size devices
    "'--remap-path-prefix=$env:USERPROFILE\.cargo=/cargo'",
    "'--remap-path-prefix=$env:USERPROFILE\.rustup=/rustup'",
    "'--remap-path-prefix=$rustRoot=/src'"
) -join ','
$CargoAndroid = @('--config', "target.aarch64-linux-android.rustflags=[$cargoRustflags]")
