# Changelog

Sankofa Rust is a fork of [Komikku](https://github.com/komikku-app/komikku) (Apache 2.0). Changes from before the fork
live in [docs/upstream/CHANGELOG-komikku.md](docs/upstream/CHANGELOG-komikku.md).

Sections: `Added`, `Changed`, `Improved`, `Removed`, `Fixed`, `Other`. Versions follow [SemVer](https://semver.org/).
The text of the matching release on GitHub is what the app shows in **About > What's new**.

## [Unreleased]

## [0.1.4] - 2026-10-18

### Added
- **Browse > Migrate: "Move titles to Rust sources"** (the arrows icon at the top). Library titles of sources that are no
  longer installed (an extension you removed) move to the Rust source of the same name and language, with their chapters,
  history and categories, and no search. The dialog lists each source and its titles, all ticked: clear a source or open
  it to pick title by title. A source whose titles all move takes its pin and saved searches along.

### Fixed
- MangaDex (Rust) v3: opening a title failed ("unknown variant `update`"); the published package was older than the app.
- Manganato v5: natomanga.com listed titles in a layout the parser could not read (an empty list) and challenges its list
  and search pages with Cloudflare; the default address is now manganato.gg (the others stay a choice in "Domain"), the
  text search goes through the site's quick search, and the requests of the site's pages are paced (3 per second) so a
  library update does not get HTTP 503. Titles saved with an address of the former domains (chapmanganato.to, or only
  `/manga-ab123456`) are found again by their name when opened or updated, with chapters from the new address.
- Mangakakalot v3: the text search goes through the site's quick search, which the Cloudflare challenge does not guard,
  so searching (and migrating) finds titles.
- Read Comics Online shows "No results found" (and a migration finds nothing) while the site asks for its Cloudflare
  "Verify you are human" check. Nothing is wrong with the source: open it, tap the globe icon (WebView) and tick the box
  once.

### Other
- Repository serial 9 (138 sources): MangaDex v3, Manganato v5, Mangakakalot v3. Use "Update all" in Settings > Browse >
  Rust sources.
- ReadComicOnline (readcomiconline.li) cannot be ported: the site is offline and the extension is gone from the Keiyoushi
  repository. Move those titles to another source by searching.

## [0.1.3] - 2026-10-18

### Added
- A search button in Settings > Browse > Rust sources: type part of a name to narrow the list of sources to download
  ("Install all shown" follows it).
- 8 more sources in Rust, the APK extensions installed on the phone that were not Rust yet: Tapas, HentaiFox (en, ja,
  zh, ko, all), MadaraDex, MangaGG, Manganato, Mangabat, Rage Scans and Grabber Zone (repository serial 8: 138 sources).
  What was and was not converted: `docs/redesign/KEIYOUSHI-EN.md`.

### Fixed
- Mangakakalot: mangakakalot.gg now answers a Cloudflare challenge on every page; the default address is the mirror
  mangakakalove.com, which serves titles, chapters and pictures without one (the other address stays a choice in the
  source's "Domain" setting). Mangakakalot v2 in the repository: use "Update all".
- Madara sites that list chapters in pages (MangaGG, Top Manhua and others) failed with HTTP 400: the request had no form
  body. The generated sources are v2.

## [0.1.2] - 2026-10-17

### Added
- **88 new sources in Rust** (English sources of the Keiyoushi extension repository, ported by a generator and checked
  against the live sites), in the official repository: 64 Madara sites, 8 MangaThemesia, 8 MangaCatalog and 8 Yiimanga
  (15 of them behind Cloudflare). New themes `sk-theme-mangacatalog` and `sk-theme-yiimanga`, and Madara sites that list
  without ajax. What was converted and what was not: `docs/redesign/KEIYOUSHI-EN.md`.
- Sources in Rust carry a "RUST" tag in Browse, so they are told apart from APK extensions with the same name.
- Browse > Extensions has a "Rust sources" entry, apart from the APK extensions, that opens the Rust source repository.

### Fixed
- Luscious shows the site's own message when it refuses a filter (for example "Real People" is blocked by region and
  some selections need a login) instead of a bare "HTTP 400" with the whole address (Luscious v3, serial 6).
- Rust sources screen: "Update all" and the other bulk actions say which source failed and why, refresh the list when
  done, are translated to Portuguese, and the repository line shows the serial it really has.
- Luscious in Rust sent no language to the site (a bare "+"): every Luscious source listed all languages. It now sends
  its own language plus the ones ticked in the filters (Luscious v2 in the repository, serial 5; use "Update all").
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
