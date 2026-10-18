//! HM2D, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.hm2d` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct HM2D;

impl Theme for HM2D {
    const NAME: &'static str = r#"HM2D"#;
    const BASE_URL: &'static str = r#"https://doujindistrict.com"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<HM2D>);
