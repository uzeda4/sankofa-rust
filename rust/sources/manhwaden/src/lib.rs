//! ManhwaDen, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.manhwaden` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct ManhwaDen;

impl Theme for ManhwaDen {
    const NAME: &'static str = r#"ManhwaDen"#;
    const BASE_URL: &'static str = r#"https://www.manhwaden.com"#;
    const NO_AJAX: bool = true;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
    const FILTER_NON_MANGA_ITEMS: bool = false;
}

export_source!(Site<ManhwaDen>);
