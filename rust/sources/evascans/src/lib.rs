//! Eva Scans, written in Rust. A MangaThemesia site: everything comes from `sk-theme-mangathemesia`.
//! Port of the Keiyoushi extension `en.evascans` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_mangathemesia::{Site, Theme};

pub struct EvaScans;

impl Theme for EvaScans {
    const NAME: &'static str = r#"Eva Scans"#;
    const BASE_URL: &'static str = r#"https://evascans.net"#;
}

export_source!(Site<EvaScans>);
