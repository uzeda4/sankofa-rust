//! Jinmangas, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.jinmangas` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct Jinmangas;

impl Theme for Jinmangas {
    const NAME: &'static str = r#"Jinmangas"#;
    const BASE_URL: &'static str = r#"https://mangafree.info"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<Jinmangas>);
