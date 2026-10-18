//! Manhwa Comics, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.manhwacomics` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct ManhwaComics;

impl Theme for ManhwaComics {
    const NAME: &'static str = r#"Manhwa Comics"#;
    const BASE_URL: &'static str = r#"https://manhwacomics.com"#;
    const MANGA_SUB_STRING: &'static str = r#"manhwa"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
    const CHAPTER_DATE_PATTERN: &'static str = r#"d MMM yyyy"#;
}

export_source!(Site<ManhwaComics>);
