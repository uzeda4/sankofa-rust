//! Anisa Scans, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.anisascans` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_source_sdk::RateLimit;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct AnisaScans;

impl Theme for AnisaScans {
    const NAME: &'static str = r#"Anisa Scans"#;
    const BASE_URL: &'static str = r#"https://anisascans.in"#;
    const RATE_LIMIT: Option<RateLimit> = Some(RateLimit { permits: 3, period_ms: 1000, host: String::new(), skip_path: String::new() });
    const RATE_LIMIT_SKIP_PATH: &'static str = r#"/wp-content/uploads/"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<AnisaScans>);
