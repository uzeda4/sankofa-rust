//! Manga Eclipse, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.mangaeclipse` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{Site, Theme};

pub struct MangaEclipse;

impl Theme for MangaEclipse {
    const NAME: &'static str = r#"Manga Eclipse"#;
    const BASE_URL: &'static str = r#"https://mangaeclipse.com"#;
}

export_source!(Site<MangaEclipse>);
