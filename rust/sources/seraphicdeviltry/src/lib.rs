//! SeraphicDeviltry, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `all.seraphicdeviltry` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_source_sdk::RateLimit;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct SeraphicDeviltry;

impl Theme for SeraphicDeviltry {
    const NAME: &'static str = r#"SeraphicDeviltry"#;
    const BASE_URL: &'static str = r#"https://seraphic-deviltry.com"#;
    const CHAPTER_DATE_PATTERN: &'static str = r#"MM/dd/yyyy"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
    const RATE_LIMIT: Option<RateLimit> = Some(RateLimit { permits: 3, period_ms: 1000, host: String::new(), skip_path: String::new() });
    const RATE_LIMIT_SKIP_PATH: &'static str = r#"/wp-content/uploads/"#;
}

export_source!(Site<SeraphicDeviltry>);
