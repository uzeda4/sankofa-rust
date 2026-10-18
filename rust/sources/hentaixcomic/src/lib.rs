//! HentaiXComic, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.hentaixcomic` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct HentaiXComic;

impl Theme for HentaiXComic {
    const NAME: &'static str = r#"HentaiXComic"#;
    const BASE_URL: &'static str = r#"https://hentaixcomic.com"#;
    const CHAPTER_DATE_PATTERN: &'static str = r#"MMM d, uuuu"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<HentaiXComic>);
