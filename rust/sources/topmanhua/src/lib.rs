//! Top Manhua, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.topmanhua` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_source_sdk::RateLimit;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct TopManhua;

impl Theme for TopManhua {
    const NAME: &'static str = r#"Top Manhua"#;
    const BASE_URL: &'static str = r#"https://mangatop.org"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjaxPaginated;
    const CHAPTER_DATE_PATTERN: &'static str = r#"MM/dd/yy"#;
    const RATE_LIMIT: Option<RateLimit> = Some(RateLimit { permits: 2, period_ms: 1000, host: String::new(), skip_path: String::new() });
    const RATE_LIMIT_SKIP_PATH: &'static str = r#"/wp-content/uploads/"#;
    const FILTER_NON_MANGA_ITEMS: bool = false;
    const MANGA_SUB_STRING: &'static str = r#"series"#;
}

export_source!(Site<TopManhua>);
