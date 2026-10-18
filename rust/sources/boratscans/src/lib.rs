//! Borat Scans, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.boratscans` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct BoratScans;

impl Theme for BoratScans {
    const NAME: &'static str = r#"Borat Scans"#;
    const BASE_URL: &'static str = r#"https://boratscans.com"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<BoratScans>);
