//! Lagoon Scans, written in Rust. A MangaThemesia site: everything comes from `sk-theme-mangathemesia`.
//! Port of the Keiyoushi extension `en.lagoonscans` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_mangathemesia::{Site, Theme};

pub struct LagoonScans;

impl Theme for LagoonScans {
    const NAME: &'static str = r#"Lagoon Scans"#;
    const BASE_URL: &'static str = r#"https://lagoonscans.com"#;
}

export_source!(Site<LagoonScans>);
