//! Toonizy, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.toonizy` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct Toonizy;

impl Theme for Toonizy {
    const NAME: &'static str = r#"Toonizy"#;
    const BASE_URL: &'static str = r#"https://toonizy.com"#;
    const CHAPTER_DATE_PATTERN: &'static str = r#"MMM d, yy"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
    const MANGA_SUB_STRING: &'static str = r#"webtoon"#;
    const DETAILS_TITLE: &'static str = r#".post-content h3"#;
}

export_source!(Site<Toonizy>);
