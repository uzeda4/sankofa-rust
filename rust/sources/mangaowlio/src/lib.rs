//! MangaOwl.io (unoriginal), written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.mangaowlio` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct MangaOwlIo;

impl Theme for MangaOwlIo {
    const NAME: &'static str = r#"MangaOwl.io (unoriginal)"#;
    const BASE_URL: &'static str = r#"https://mangaowl.io"#;
    const MANGA_SUB_STRING: &'static str = r#"read-1"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<MangaOwlIo>);
