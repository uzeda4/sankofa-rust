//! Daily Manhwa, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.dailymanhwa` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct DailyManhwa;

impl Theme for DailyManhwa {
    const NAME: &'static str = r#"Daily Manhwa"#;
    const BASE_URL: &'static str = r#"https://www.dailymanhwa.com"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<DailyManhwa>);
