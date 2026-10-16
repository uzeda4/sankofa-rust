<div align="center">

# Sankofa Rust

**A manga reader with a Rust core and a cover-driven interface.**
*Sankofa: go back and fetch it.*

[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0877d2?labelColor=27303D)](LICENSE)

</div>

Sankofa Rust is an Android manga reader. It is a fork of [Komikku](https://github.com/komikku-app/komikku) (itself built on
Mihon/Tachiyomi and TachiyomiSY) with three goals:

- **A new interface.** A dark, cover-driven design: the palette follows the title you are looking at, covers tilt and
  catch light, the library has a "Rediscover" cover flow, and a cover flies from the library into the details screen.
  Motion has a fallback: *Reduce motion*, power saver and low-RAM devices switch the 3D effects off.
- **A Rust core.** The database reader, the HTTPS client and a host for sources now live in Rust (`rust/`), loaded into the
  Kotlin app through UniFFI. The interface is still Jetpack Compose.
- **Sources written in Rust.** Besides Kotlin extensions, the app runs sources compiled to WebAssembly, installed from
  signed packages or from a signed repository index. A source cannot open sockets or files and cannot reach your local
  network. See [docs/redesign/RUST-SOURCES.md](docs/redesign/RUST-SOURCES.md) to write one.

Everything Komikku does today (library, categories, trackers, downloads, backups, Kotlin extensions) is still here.

## Status
Early: version 0.1.0, `arm64-v8a` only (the Rust core is built for that ABI). Every arm64 native library is 16 KB
page-size aligned. Things that do not exist yet are listed in the docs rather than promised here.

## Install
Download `app-arm64-v8a-release.apk` from the [releases](../../releases) page. The application id is `app.sankofa.rust`,
so it installs next to Komikku and next to other builds.

## Build
Requirements: Android Studio (JDK 17+), the Android SDK and NDK, and for the Rust core [Rust](https://rustup.rs) with the
`aarch64-linux-android` target.

```text
rust/tools/build-android.ps1                  # builds libsk_api.so and the Kotlin bindings (Windows)
./gradlew :app:assembleDebug                  # debug build
./gradlew :app:assembleRelease "-Predesign.signing=<your signing.properties>"
```
Without `-Psankofa.updateRepo=owner/repo` the build has no updater and no in-app changelog. Releasing:
[docs/redesign/RELEASING.md](docs/redesign/RELEASING.md). `libimagedecoder.so` is rebuilt for 16 KB alignment:
[native/image-decoder/README.md](native/image-decoder/README.md).

## Documentation
| | |
|---|---|
| [docs/redesign/IDENTITY.md](docs/redesign/IDENTITY.md) | Name, mark, palette and credit for the Akan concept behind *Sankofa* |
| [docs/redesign/HANDOFF.md](docs/redesign/HANDOFF.md) | Design tokens, motion and implementation status |
| [docs/redesign/RUST-SPIKE.md](docs/redesign/RUST-SPIKE.md) | What was measured when deciding how much of the app to write in Rust |
| [docs/redesign/RUST-SOURCES.md](docs/redesign/RUST-SOURCES.md) | Writing, signing and publishing Rust sources and repositories |
| [SECURITY.md](SECURITY.md) | How trust works in the app and how to report a problem |

The design notes are mostly written in Portuguese.

## Credits and license
Apache License 2.0, see [LICENSE](LICENSE). This project is built on the work of the Komikku, Mihon, TachiyomiSY and
Tachiyomi authors; the history before the fork is theirs and is kept, and their changelog is in
[docs/upstream/CHANGELOG-komikku.md](docs/upstream/CHANGELOG-komikku.md). Fonts (Bricolage Grotesque, DM Sans) are under
the SIL Open Font License, see `docs/redesign/licenses`.

*Sankofa* is an Akan (Ghana) word and Adinkra concept, usually shown as a bird looking back while moving forward. The mark
in this app is an original drawing and does not reproduce any existing Adinkra artwork.

This project is not affiliated with Komikku, Mihon or any content provider and does not host or distribute manga.
