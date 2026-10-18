//! LHTranslation, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.lhtranslation` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct LHTranslation;

impl Theme for LHTranslation {
    const NAME: &'static str = r#"LHTranslation"#;
    const BASE_URL: &'static str = r#"https://lhtranslation.net"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<LHTranslation>);
