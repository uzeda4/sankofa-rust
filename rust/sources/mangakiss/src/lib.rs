//! Manga Kiss, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.mangakiss` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{Site, Theme};

pub struct MangaKiss;

impl Theme for MangaKiss {
    const NAME: &'static str = r#"Manga Kiss"#;
    const BASE_URL: &'static str = r#"https://mangakiss.org"#;
}

export_source!(Site<MangaKiss>);
