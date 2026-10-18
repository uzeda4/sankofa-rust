//! MadaraDex, written in Rust. A Madara site that asks for a browser fingerprint cookie (`mdx_fp`) and refreshes an
//! auth cookie (`mdx_auth`) with a request of its own; everything else comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.madaradex` (Apache-2.0). The extension also retried a picture that its CDN
//! refused with a fresh auth; here the auth is refreshed before each listing, title and chapter request instead.

use sk_source_sdk::*;
use sk_theme_madara::{Site, Theme};

struct Dex;

impl Theme for Dex {
    const NAME: &'static str = "MadaraDex";
    const BASE_URL: &'static str = "https://madaradex.org";
    const MANGA_SUB_STRING: &'static str = "title";
    const GENRE_DIRECTORY: &'static str = "genre";
}

/// The site hands out `mdx_auth` in answer to this call; the app's cookie jar keeps it for the pictures too.
fn refresh_auth() {
    let _ = post("https://madaradex.org/wp-admin/admin-ajax.php").header("X-Requested-With", "XMLHttpRequest").form(&[("action", "mdx_auth_refresh")]).allow_errors().send();
}

/// 16 bytes in hex, different per start of the app (the extension drew them at random).
fn fingerprint() -> String {
    let mut x = now_ms() as u64 ^ 0x9E37_79B9_7F4A_7C15;
    (0..2)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            format!("{x:016x}")
        })
        .collect()
}

#[derive(Default)]
pub struct MadaraDex(Site<Dex>);

impl Source for MadaraDex {
    fn info(&self) -> Info {
        let mut info = self.0.info();
        info.headers.push(("sec-fetch-site".into(), "same-site".into()));
        info.cookies.push(CookieDef::new("madaradex.org", "mdx_fp", &fingerprint()));
        info
    }

    fn popular(&self, page: u32) -> Result<MangaPage> {
        refresh_auth();
        self.0.popular(page)
    }

    fn latest(&self, page: u32) -> Result<MangaPage> {
        refresh_auth();
        self.0.latest(page)
    }

    fn search(&self, page: u32, query: &str, filters: &Filters) -> Result<MangaPage> {
        refresh_auth();
        self.0.search(page, query, filters)
    }

    fn filters(&self, data: Option<&Value>) -> Vec<FilterDef> {
        self.0.filters(data)
    }

    fn filter_data(&self) -> Result<Option<Value>> {
        self.0.filter_data()
    }

    fn details(&self, manga: &Manga) -> Result<Manga> {
        refresh_auth();
        self.0.details(manga)
    }

    fn chapters(&self, manga: &Manga) -> Result<Vec<Chapter>> {
        refresh_auth();
        self.0.chapters(manga)
    }

    fn update(&self, manga: &Manga, fetch_details: bool, fetch_chapters: bool) -> Result<Update> {
        refresh_auth();
        self.0.update(manga, fetch_details, fetch_chapters)
    }

    fn pages(&self, chapter: &Chapter) -> Result<Vec<Page>> {
        refresh_auth();
        self.0.pages(chapter)
    }

    fn by_url(&self, url: &str) -> Result<Option<Manga>> {
        self.0.by_url(url)
    }

    fn manga_url(&self, manga: &Manga) -> Option<String> {
        self.0.manga_url(manga)
    }

    fn chapter_url(&self, chapter: &Chapter) -> Option<String> {
        self.0.chapter_url(chapter)
    }

    fn related(&self, manga: &Manga) -> Result<Vec<Manga>> {
        self.0.related(manga)
    }
}

export_source!(MadaraDex);
