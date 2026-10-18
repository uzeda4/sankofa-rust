//! Manhwa Reads, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.manhwareads` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct ManhwaReads;

impl Theme for ManhwaReads {
    const NAME: &'static str = r#"Manhwa Reads"#;
    const BASE_URL: &'static str = r#"https://manhwareads.com"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<ManhwaReads>);
