//! Hades Scans, written in Rust. A MangaThemesia site: everything comes from `sk-theme-mangathemesia`.
//! Port of the Keiyoushi extension `en.hadesscans` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_mangathemesia::{Site, Theme};

pub struct HadesScans;

impl Theme for HadesScans {
    const NAME: &'static str = r#"Hades Scans"#;
    const BASE_URL: &'static str = r#"https://hadesscans.com"#;
}

export_source!(Site<HadesScans>);
