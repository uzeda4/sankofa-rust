//! Athrea Scans, written in Rust. A MangaThemesia site: everything comes from `sk-theme-mangathemesia`.
//! Port of the Keiyoushi extension `en.athreascans` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_mangathemesia::{Site, Theme};

pub struct AthreaScans;

impl Theme for AthreaScans {
    const NAME: &'static str = r#"Athrea Scans"#;
    const BASE_URL: &'static str = r#"https://athreascans.com"#;
}

export_source!(Site<AthreaScans>);
