//! Manga18Free, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.manga18free` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct Manga18Free;

impl Theme for Manga18Free {
    const NAME: &'static str = r#"Manga18Free"#;
    const BASE_URL: &'static str = r#"https://manga18free.com"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::AdminAjax;
}

export_source!(Site<Manga18Free>);
