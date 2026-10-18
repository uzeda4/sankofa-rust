# Changelog

Sankofa Rust is a fork of [Komikku](https://github.com/komikku-app/komikku) (Apache 2.0). Changes from before the fork
live in [docs/upstream/CHANGELOG-komikku.md](docs/upstream/CHANGELOG-komikku.md).

Sections: `Added`, `Changed`, `Improved`, `Removed`, `Fixed`, `Other`. Versions follow [SemVer](https://semver.org/).
The text of the matching release on GitHub is what the app shows in **About > What's new**.

## [Unreleased]

## [0.1.2] - 2026-10-17

### Fixed
- A source in Rust and the APK extension it was ported from were shown as one item in Browse (the Rust one hid the
  extension). Sources in Rust now have an id of their own, so both can be installed and listed side by side.
  Titles saved in the library from a Rust source before this version no longer match it: add them again or use Migrate.

## [0.1.1] - 2026-10-17

### Added
- The official Rust source repository comes pre-added and trusted (its key is pinned in the app): open
  Settings > Browse > Rust sources and the sources are listed, ready to install. Removing it keeps it removed.
- 36 sources in Rust in the official repository, among them MangaDex, MangaFire, Webtoons, Asura Scans, Arena Scans,
  Asmodeus Scans, Kissmanga.in, Manga Demon, MangaFox, Mangakakalot, MurimScan, Read Comics Online and Luscious, in the
  languages each one serves, with icons.
- Rust sources: WebAssembly ABI v2 (selector helpers, cookies, rate limits per host, custom URLs) and shared theme
  crates for sites that run the same engine (Madara, Zeist).
- Rust sources screen: language filter for the available list, "Update all", "Install all shown" and "Uninstall all".

### Fixed
- Debug builds showed Komikku's yellow launcher icon; every build now uses the Sankofa mark.

## [0.1.0] - 2026-10-16

### Added
- Rust core inside the app (UniFFI): reads the app's own database, HTTPS client, and a host for sources written in
  Rust and compiled to WebAssembly.
- Sources in Rust: install from a signed `.sksource` package (Ed25519, trusted-publisher fingerprint) or from a signed
  repository index (`index.skrepo`) with rollback protection. Settings > Browse > Rust sources.
- Reference source: MangaDex in Rust. Authoring guide in `docs/redesign/RUST-SOURCES.md`.
- "Rediscover" cover flow in the library; shared cover transition from the library to the details screen; glare and
  parallax on covers; the reader's chapter card turns as it is swiped.
- Sankofa identity: app name, launcher / themed / splash icons, in-app mark, empty states.
- Redesign theme (cover-driven palette, glass dock) as the default; "Reduce motion" and tilt strength settings.

### Changed
- The app updates only from the repository it is built with (`-Psankofa.updateRepo=owner/repo`). Builds without one have
  no updater and no in-app changelog instead of pointing at Komikku's releases.
- The About screen no longer links to Komikku's website, Discord, translation project or privacy policy.
- Application id is `app.sankofa.rust` and installs next to the original Sankofa and Komikku.

### Security
- Rust sources cannot reach local or private network addresses.
- The updater's API address can be overridden only in debug builds; releases always use GitHub.
- Removed the "Sponsor" button (it pointed at Komikku's maintainer) and Komikku's CI, funding and issue templates.

### Fixed
- 16 KB page size: `libimagedecoder.so` is rebuilt with 16 KB segment alignment, so every arm64 native library passes
  the check (`native/image-decoder/README.md`).
- The update check treats `0.1.0-1234` (debug and preview builds) as version 0.1.0 instead of misreading the suffix.
