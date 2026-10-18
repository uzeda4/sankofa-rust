//! ManhwaManhua, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.manhwamanhua` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct ManhwaManhua;

impl Theme for ManhwaManhua {
    const NAME: &'static str = r#"ManhwaManhua"#;
    const BASE_URL: &'static str = r#"https://manhwamanhua.com"#;
    const FILTER_NON_MANGA_ITEMS: bool = false;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<ManhwaManhua>);
