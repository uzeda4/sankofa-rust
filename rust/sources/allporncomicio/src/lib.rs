//! AllPornComic.io, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.allporncomicio` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct AllPornComicIo;

impl Theme for AllPornComicIo {
    const NAME: &'static str = r#"AllPornComic.io"#;
    const BASE_URL: &'static str = r#"https://allporncomic.io"#;
    const CHAPTER_DATE_PATTERN: &'static str = r#"dd/MM/yyyy"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<AllPornComicIo>);
