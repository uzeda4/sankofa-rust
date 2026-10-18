//! TritiniaScans, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.tritiniascans` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct TritiniaScans;

impl Theme for TritiniaScans {
    const NAME: &'static str = r#"TritiniaScans"#;
    const BASE_URL: &'static str = r#"https://tritinia.org"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<TritiniaScans>);
