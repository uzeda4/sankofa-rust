//! ManhuaUS, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.manhuaus` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct ManhuaUS;

impl Theme for ManhuaUS {
    const NAME: &'static str = r#"ManhuaUS"#;
    const BASE_URL: &'static str = r#"https://manhuaus.com"#;
    const FILTER_NON_MANGA_ITEMS: bool = false;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<ManhuaUS>);
