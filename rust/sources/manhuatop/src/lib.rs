//! ManhuaTop, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.manhuatop` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct ManhuaTop;

impl Theme for ManhuaTop {
    const NAME: &'static str = r#"ManhuaTop"#;
    const BASE_URL: &'static str = r#"https://manhuatop.org"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
    const CHAPTER_DATE_PATTERN: &'static str = r#"MM/dd/yyyy"#;
    const MANGA_SUB_STRING: &'static str = r#"manhua"#;
    const FILTER_NON_MANGA_ITEMS: bool = false;
}

export_source!(Site<ManhuaTop>);
