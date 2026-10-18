//! Mangaforfree.com, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.mangaforfreecom` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct Mangaforfreecom;

impl Theme for Mangaforfreecom {
    const NAME: &'static str = r#"Mangaforfree.com"#;
    const BASE_URL: &'static str = r#"https://mangaforfree.com"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::AdminAjax;
}

export_source!(Site<Mangaforfreecom>);
