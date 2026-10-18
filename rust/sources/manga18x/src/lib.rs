//! Manga 18x, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.manga18x` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct Manga18x;

impl Theme for Manga18x {
    const NAME: &'static str = r#"Manga 18x"#;
    const BASE_URL: &'static str = r#"https://manga18x.net"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<Manga18x>);
