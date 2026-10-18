//! HentaiXYuri, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.hentaixyuri` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct HentaiXYuri;

impl Theme for HentaiXYuri {
    const NAME: &'static str = r#"HentaiXYuri"#;
    const BASE_URL: &'static str = r#"https://hentaixyuri.com"#;
    const CHAPTER_DATE_PATTERN: &'static str = r#"MMM d, uuuu"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<HentaiXYuri>);
