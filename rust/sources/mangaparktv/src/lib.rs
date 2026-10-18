//! MangaPark.tv (unoriginal), written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.mangaparktv` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct MangaParkTv;

impl Theme for MangaParkTv {
    const NAME: &'static str = r#"MangaPark.tv (unoriginal)"#;
    const BASE_URL: &'static str = r#"https://mangapark.tv"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<MangaParkTv>);
