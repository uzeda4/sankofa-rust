//! GingeRTooN, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.gingertoon` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct GingeRTooN;

impl Theme for GingeRTooN {
    const NAME: &'static str = r#"GingeRTooN"#;
    const BASE_URL: &'static str = r#"https://gingertoon.com"#;
    const NO_AJAX: bool = true;
    const CHAPTER_DATE_PATTERN: &'static str = r#"MM.dd.uuuu"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<GingeRTooN>);
