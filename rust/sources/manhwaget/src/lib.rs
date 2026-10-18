//! ManhwaGet, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.manhwaget` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct ManhwaGet;

impl Theme for ManhwaGet {
    const NAME: &'static str = r#"ManhwaGet"#;
    const BASE_URL: &'static str = r#"https://manhwaget.com"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<ManhwaGet>);
