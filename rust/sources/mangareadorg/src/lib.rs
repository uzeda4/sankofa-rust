//! MangaRead.org, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.mangareadorg` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct MangaReadOrg;

impl Theme for MangaReadOrg {
    const NAME: &'static str = r#"MangaRead.org"#;
    const BASE_URL: &'static str = r#"https://www.mangaread.org"#;
    const CHAPTER_DATE_PATTERN: &'static str = r#"dd.MM.yyy"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<MangaReadOrg>);
