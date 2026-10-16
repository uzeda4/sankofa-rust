# libimagedecoder with 16 KB alignment

Android devices with 16 KB memory pages (and the Play Store requirement that follows) need every native library's
`PT_LOAD` segments aligned to 16 KB. Every `.so` in this app already was, except `libimagedecoder.so`, which comes
from `com.github.tachiyomiorg:image-decoder:41c059e540` (JNI half of Coil's AVIF/HEIF/JXL decoding and the reader's
image view) and was linked with 4 KB alignment. That project is unmaintained (last commit July 2024) and its
successor `ca.mpreg:imagedecoder` has a different API (no `decode(sampleSize)`, no crop-borders or ICC input), so
moving to it means rewriting the reader's image pipeline.

Instead, `build.ps1` builds the **same sources at the same commit** with the current NDK and
`-DANDROID_SUPPORT_FLEXIBLE_PAGE_SIZES=ON`, for `arm64-v8a`, and `app/build.gradle.kts` makes the APK take this copy
(`jniLibs.srcDir` + `pickFirsts`). The Java classes still come from the Maven artifact.

| Check | Shipped AAR | This build |
|---|---|---|
| Size (stripped) | 4.95 MB | 4.96 MB |
| JNI exports | 4 `Java_tachiyomi_decoder_ImageDecoder_*` | the same |
| `NEEDED` libraries | libz, libandroid, libjnigraphics, liblog, libm, libdl, libc | the same |
| ELF `LOAD` alignment | `0x1000` | `0x4000` |
| Decodes an AVIF on a phone (dav1d included) | yes | yes (640x960 test image) |

`prebuilt/arm64-v8a/libimagedecoder.so` is committed (5 MB). SHA-256 of the current build:
`45c63967cb0a7d9bc6673a668ff82f8e922d8a35f2332b4ac9c4b9008ff09b8c`.

Not covered: `x86_64` (emulators; dav1d there needs `nasm`) and the 32-bit ABIs, which 16 KB devices do not run.
Those keep the AAR's copy. Revisit this whole directory if the app moves to `ca.mpreg:imagedecoder`.
