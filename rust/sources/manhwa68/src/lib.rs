//! Manhwa68, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.manhwa68` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct Manhwa68;

impl Theme for Manhwa68 {
    const NAME: &'static str = r#"Manhwa68"#;
    const BASE_URL: &'static str = r#"https://manhwa68.com"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
    const CHAPTER_DATE_PATTERN: &'static str = r#"MMM d, yyyy"#;
    const FILTER_NON_MANGA_ITEMS: bool = false;
}

export_source!(Site<Manhwa68>);
