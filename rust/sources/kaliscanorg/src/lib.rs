//! KaliScan.org (unoriginal), written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.kaliscanorg` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{ChapterMode, Site, Theme};

pub struct KaliScanOrg;

impl Theme for KaliScanOrg {
    const NAME: &'static str = r#"KaliScan.org (unoriginal)"#;
    const BASE_URL: &'static str = r#"https://kaliscan.org"#;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}

export_source!(Site<KaliScanOrg>);
