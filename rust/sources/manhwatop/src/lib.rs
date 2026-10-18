//! Manhwatop, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.manhwatop` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct Manhwatop;

impl Theme for Manhwatop {
    const NAME: &'static str = r#"Manhwatop"#;
    const BASE_URL: &'static str = r#"https://manhwatop.com"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
    const FILTER_NON_MANGA_ITEMS: bool = false;
}

export_source!(Site<Manhwatop>);
