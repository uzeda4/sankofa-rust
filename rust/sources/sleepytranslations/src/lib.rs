//! Sleepy Translations, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.sleepytranslations` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct SleepyTranslations;

impl Theme for SleepyTranslations {
    const NAME: &'static str = r#"Sleepy Translations"#;
    const BASE_URL: &'static str = r#"https://sleepytranslations.com"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<SleepyTranslations>);
