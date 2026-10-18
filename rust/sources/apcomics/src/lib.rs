//! AP Comics, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.apcomics` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct Apcomics;

impl Theme for Apcomics {
    const NAME: &'static str = r#"AP Comics"#;
    const BASE_URL: &'static str = r#"https://apcomics.org"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<Apcomics>);
