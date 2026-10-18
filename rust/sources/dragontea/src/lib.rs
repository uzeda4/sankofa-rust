//! DragonTea, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.dragontea` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_source_sdk::RateLimit;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct DragonTea;

impl Theme for DragonTea {
    const NAME: &'static str = r#"DragonTea"#;
    const BASE_URL: &'static str = r#"https://dragontea.ink"#;
    const CHAPTER_DATE_PATTERN: &'static str = r#"MM/dd/yyyy"#;
    const MANGA_SUB_STRING: &'static str = r#"novel"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
    const RATE_LIMIT: Option<RateLimit> = Some(RateLimit { permits: 1, period_ms: 1000, host: String::new(), skip_path: String::new() });
    const RATE_LIMIT_SKIP_PATH: &'static str = r#"/wp-content/uploads/"#;
}

export_source!(Site<DragonTea>);
