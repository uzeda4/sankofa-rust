//! Aster Scans, written in Rust. A MangaThemesia site: everything comes from `sk-theme-mangathemesia`.
//! Port of the Keiyoushi extension `en.asterscans` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_mangathemesia::{Site, Theme};

pub struct AsterScans;

impl Theme for AsterScans {
    const NAME: &'static str = r#"Aster Scans"#;
    const BASE_URL: &'static str = r#"https://asterscans.com"#;
}

export_source!(Site<AsterScans>);
