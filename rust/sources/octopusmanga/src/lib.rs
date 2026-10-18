//! OctopusManga, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.octopusmanga` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct OctopusManga;

impl Theme for OctopusManga {
    const NAME: &'static str = r#"OctopusManga"#;
    const BASE_URL: &'static str = r#"https://octopusmanga.com"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<OctopusManga>);
