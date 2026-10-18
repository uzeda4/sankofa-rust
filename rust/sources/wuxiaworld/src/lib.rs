//! WuxiaWorld, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.wuxiaworld` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct WuxiaWorld;

impl Theme for WuxiaWorld {
    const NAME: &'static str = r#"WuxiaWorld"#;
    const BASE_URL: &'static str = r#"https://wuxiaworld.site"#;
    const MANGA_SUB_STRING: &'static str = r#"novel"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<WuxiaWorld>);
