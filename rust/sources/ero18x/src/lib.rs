//! Ero18x, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.ero18x` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct Ero18x;

impl Theme for Ero18x {
    const NAME: &'static str = r#"Ero18x"#;
    const BASE_URL: &'static str = r#"https://ero18x.com"#;
    const NO_AJAX: bool = true;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<Ero18x>);
