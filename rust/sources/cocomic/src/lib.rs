//! Cocomic, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.cocomic` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_source_sdk::RateLimit;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct Cocomic;

impl Theme for Cocomic {
    const NAME: &'static str = r#"Cocomic"#;
    const BASE_URL: &'static str = r#"https://cocomic.co"#;
    const RATE_LIMIT: Option<RateLimit> = Some(RateLimit { permits: 1, period_ms: 2000, host: String::new(), skip_path: String::new() });
    const RATE_LIMIT_SKIP_PATH: &'static str = r#"/wp-content/uploads/"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
    const CHAPTER_LIST_SELECTOR: &'static str = r#"li.wp-manga-chapter:not(.premium)"#;
}

export_source!(Site<Cocomic>);
