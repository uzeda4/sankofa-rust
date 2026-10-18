//! Manhwa18.today, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.manhwa18today` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct Manhwa18Today;

impl Theme for Manhwa18Today {
    const NAME: &'static str = r#"Manhwa18.today"#;
    const BASE_URL: &'static str = r#"https://www.manhwa18.today"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<Manhwa18Today>);
