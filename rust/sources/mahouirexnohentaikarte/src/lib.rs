//! Mahouirexnohentaikarte, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.mahouirexnohentaikarte` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct Mahouirexnohentaikarte;

impl Theme for Mahouirexnohentaikarte {
    const NAME: &'static str = r#"Mahouirexnohentaikarte"#;
    const BASE_URL: &'static str = r#"https://mahouirexnohentaikarte.com"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjaxQuery;
}

export_source!(Site<Mahouirexnohentaikarte>);
