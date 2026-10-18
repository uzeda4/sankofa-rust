//! GalaxyDegenScans, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.galaxydegenscans` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct GalaxyDegenScans;

impl Theme for GalaxyDegenScans {
    const NAME: &'static str = r#"GalaxyDegenScans"#;
    const BASE_URL: &'static str = r#"https://gdscans.com"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<GalaxyDegenScans>);
