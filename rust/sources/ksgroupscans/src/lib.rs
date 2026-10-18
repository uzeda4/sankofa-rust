//! KSGroupScans, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.ksgroupscans` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct KSGroupScans;

impl Theme for KSGroupScans {
    const NAME: &'static str = r#"KSGroupScans"#;
    const BASE_URL: &'static str = r#"https://ksgroupscans.com"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<KSGroupScans>);
