//! Decadence Scans, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.decadencescans` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct DecadenceScans;

impl Theme for DecadenceScans {
    const NAME: &'static str = r#"Decadence Scans"#;
    const BASE_URL: &'static str = r#"https://reader.decadencescans.com"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<DecadenceScans>);
