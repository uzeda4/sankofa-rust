//! Hunlight Comics, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.hunlightcomics` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct HunlightComics;

impl Theme for HunlightComics {
    const NAME: &'static str = r#"Hunlight Comics"#;
    const BASE_URL: &'static str = r#"https://hunlightcomics.com"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<HunlightComics>);
