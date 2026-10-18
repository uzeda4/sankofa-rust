//! KokoMangas, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.wearehunger` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct Wearehunger;

impl Theme for Wearehunger {
    const NAME: &'static str = r#"KokoMangas"#;
    const BASE_URL: &'static str = r#"https://kokomangas.com"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<Wearehunger>);
