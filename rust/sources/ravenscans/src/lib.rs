//! Raven Scans, written in Rust. A MangaThemesia site: everything comes from `sk-theme-mangathemesia`.
//! Port of the Keiyoushi extension `en.ravenscans` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_mangathemesia::{Site, Theme};

pub struct RavenScans;

impl Theme for RavenScans {
    const NAME: &'static str = r#"Raven Scans"#;
    const BASE_URL: &'static str = r#"https://ravenscans.org"#;
    const MANGA_URL_DIRECTORY: &'static str = r#"/manga"#;
}

export_source!(Site<RavenScans>);
