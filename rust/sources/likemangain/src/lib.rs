//! MangaYY, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.likemangain` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct MangaYY;

impl Theme for MangaYY {
    const NAME: &'static str = r#"MangaYY"#;
    const BASE_URL: &'static str = r#"https://mangayy.org"#;
    const CHAPTER_DATE_PATTERN: &'static str = r#"dd MMMM, yyyy"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<MangaYY>);
