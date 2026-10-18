//! Madara Scans, written in Rust. A MangaThemesia site: everything comes from `sk-theme-mangathemesia`.
//! Port of the Keiyoushi extension `en.madarascans` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_mangathemesia::{Site, Theme};

pub struct MadaraScans;

impl Theme for MadaraScans {
    const NAME: &'static str = r#"Madara Scans"#;
    const BASE_URL: &'static str = r#"https://madarascans.net"#;
}

export_source!(Site<MadaraScans>);
