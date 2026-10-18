//! GakaMangas, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.gakamangas` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct GakaMangas;

impl Theme for GakaMangas {
    const NAME: &'static str = r#"GakaMangas"#;
    const BASE_URL: &'static str = r#"https://gakamangas.com"#;
    const FILTER_NON_MANGA_ITEMS: bool = false;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<GakaMangas>);
