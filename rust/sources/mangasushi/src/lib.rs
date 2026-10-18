//! Mangasushi, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.mangasushi` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct Mangasushi;

impl Theme for Mangasushi {
    const NAME: &'static str = r#"Mangasushi"#;
    const BASE_URL: &'static str = r#"https://mangasushi.org"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<Mangasushi>);
