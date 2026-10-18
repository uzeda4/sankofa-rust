//! Akaza Scans, written in Rust. A MangaThemesia site: everything comes from `sk-theme-mangathemesia`.
//! Port of the Keiyoushi extension `en.akazascans` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_mangathemesia::{Site, Theme};

pub struct AkazaScans;

impl Theme for AkazaScans {
    const NAME: &'static str = r#"Akaza Scans"#;
    const BASE_URL: &'static str = r#"https://akazascans.org"#;
}

export_source!(Site<AkazaScans>);
