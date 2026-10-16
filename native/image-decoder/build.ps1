# Rebuilds libimagedecoder.so (the native half of tachiyomiorg/image-decoder, the JNI library behind Coil's
# AVIF/HEIF/JXL decoding and the reader's image view) with 16 KB ELF alignment, for arm64-v8a.
#
# Why: the published artifact (com.github.tachiyomiorg:image-decoder:41c059e540) was linked with 4 KB segment
# alignment, so Android on 16 KB-page devices flags the app. The sources at that same commit, linked with the
# current NDK and -DANDROID_SUPPORT_FLEXIBLE_PAGE_SIZES=ON, give the same library (same JNI exports, same
# dependencies, same size) with 0x4000 alignment. The Java classes still come from the Maven artifact.
#
# Needs: Android NDK (path below), CMake, Ninja, Git, Python with `pip install meson`. Takes ~10 minutes.
# Output: native/image-decoder/prebuilt/arm64-v8a/libimagedecoder.so (committed; app/build.gradle.kts picks it
# over the AAR's copy). Re-run only when the pinned commit or the NDK changes.
param(
    [string]$Work = "$PSScriptRoot\..\..\..\sankofa-native",
    [string]$Ndk = "$env:LOCALAPPDATA\Android\Sdk\ndk\29.0.14206865"
)
$ErrorActionPreference = 'Stop'
$commit = '41c059e540'
$dav1dTag = '1.4.0'
New-Item -ItemType Directory -Force $Work | Out-Null
$Work = (Resolve-Path $Work).Path
$bin = "$Ndk\toolchains\llvm\prebuilt\windows-x86_64\bin" -replace '\', '/'
$env:PATH = "$env:LOCALAPPDATA\Microsoft\WinGet\Links;$env:PATH"

# 1. sources
if (-not (Test-Path "$Work\image-decoder")) { git clone --quiet https://github.com/tachiyomiorg/image-decoder.git "$Work\image-decoder" }
git -C "$Work\image-decoder" checkout --quiet $commit
if (-not (Test-Path "$Work\dav1d")) { git clone --quiet --depth 1 --branch $dav1dTag https://github.com/videolan/dav1d.git "$Work\dav1d" }
$cpp = "$Work\image-decoder\library\src\main\cpp"

# 2. dav1d with meson (the upstream recipe shells out to bash and `uname`, which does not work on Windows)
@"
[binaries]
c     = '$bin/aarch64-linux-android21-clang.cmd'
cpp   = '$bin/aarch64-linux-android21-clang++.cmd'
ar    = '$bin/llvm-ar.exe'
strip = '$bin/llvm-strip.exe'

[properties]
needs_exe_wrapper = true

[host_machine]
system = 'android'
cpu_family = 'aarch64'
cpu = 'aarch64'
endian = 'little'
"@ | Set-Content "$Work\android_arm64.txt" -Encoding ascii
if (-not (Test-Path "$Work\dav1d-arm64\src\libdav1d.a")) {
    python -m mesonbuild.mesonmain setup "$Work\dav1d-arm64" "$Work\dav1d" --cross-file "$Work\android_arm64.txt" `
        --buildtype release --default-library static -Denable_tools=false -Denable_tests=false `
        -Dc_link_args="-Wl,-z,max-page-size=16384"
    Push-Location "$Work\dav1d-arm64"; ninja; Pop-Location
}

# 3. the decoder: use the prebuilt dav1d, and replace two PATCH_COMMANDs that need a POSIX shell
Copy-Item "$PSScriptRoot\dav1d.cmake" "$cpp\libheif\dav1d.cmake" -Force
foreach ($f in "$cpp\lcms\CMakeLists.txt", "$cpp\libheif\CMakeLists.txt") {
    (Get-Content $f -Raw) -replace 'PATCH_COMMAND\s+git apply[^\r\n]*', 'PATCH_COMMAND   ${CMAKE_COMMAND} -E true' `
        -replace 'PATCH_COMMAND\s+echo "" >[^\r\n]*', 'PATCH_COMMAND   ${CMAKE_COMMAND} -E true' | Set-Content $f -Encoding ascii
}
# libheif's gdk-pixbuf placeholder must be an empty-but-valid CMake file (the echo above did that on Linux)
$build = "$Work\build-arm64"
cmake -S $cpp -B $build -G Ninja "-DCMAKE_TOOLCHAIN_FILE=$Ndk\build\cmake\android.toolchain.cmake" `
    -DANDROID_ABI=arm64-v8a -DANDROID_PLATFORM=android-21 -DANDROID_SUPPORT_FLEXIBLE_PAGE_SIZES=ON `
    -DCMAKE_BUILD_TYPE=Release "-DDAV1D_PREBUILT_DIR=$Work\dav1d-arm64" "-DDAV1D_SOURCE_DIR=$Work\dav1d" `
    -DCMAKE_POLICY_VERSION_MINIMUM=3.5
if ($LASTEXITCODE -ne 0) {
    # First configure downloads the dependencies; libheif's gdk-pixbuf stub then needs to be valid CMake.
    Set-Content "$Work\_deps\libheif-src\gdk-pixbuf\CMakeLists.txt" "# not needed" -Encoding ascii
    cmake -S $cpp -B $build -G Ninja "-DCMAKE_TOOLCHAIN_FILE=$Ndk\build\cmake\android.toolchain.cmake" `
        -DANDROID_ABI=arm64-v8a -DANDROID_PLATFORM=android-21 -DANDROID_SUPPORT_FLEXIBLE_PAGE_SIZES=ON `
        -DCMAKE_BUILD_TYPE=Release "-DDAV1D_PREBUILT_DIR=$Work\dav1d-arm64" "-DDAV1D_SOURCE_DIR=$Work\dav1d" `
        -DCMAKE_POLICY_VERSION_MINIMUM=3.5
}
ninja -C $build imagedecoder

# 4. strip and publish
$out = "$PSScriptRoot\prebuilt\arm64-v8a"
New-Item -ItemType Directory -Force $out | Out-Null
& "$Ndk\toolchains\llvm\prebuilt\windows-x86_64\bin\llvm-strip.exe" --strip-unneeded "$build\libimagedecoder.so" -o "$out\libimagedecoder.so"
"{0}  {1:N2} MB  sha256 {2}" -f "$out\libimagedecoder.so", ((Get-Item "$out\libimagedecoder.so").Length / 1MB), (Get-FileHash "$out\libimagedecoder.so").Hash.ToLower()
