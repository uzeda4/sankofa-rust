//! Cucumber Manga, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.cucumbermanga` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct CucumberManga;

impl Theme for CucumberManga {
    const NAME: &'static str = r#"Cucumber Manga"#;
    const BASE_URL: &'static str = r#"https://cucumbermanga.com"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<CucumberManga>);
