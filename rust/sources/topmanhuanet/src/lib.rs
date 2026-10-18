//! TopManhua.net, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.topmanhuanet` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct TopManhuaNet;

impl Theme for TopManhuaNet {
    const NAME: &'static str = r#"TopManhua.net"#;
    const BASE_URL: &'static str = r#"https://topmanhua.net"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<TopManhuaNet>);
