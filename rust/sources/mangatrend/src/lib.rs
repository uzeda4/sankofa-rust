//! Manga Trend, written in Rust. A MangaThemesia site: everything comes from `sk-theme-mangathemesia`.
//! Port of the Keiyoushi extension `en.mangatrend` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_mangathemesia::{Site, Theme};

pub struct MangaTrend;

impl Theme for MangaTrend {
    const NAME: &'static str = r#"Manga Trend"#;
    const BASE_URL: &'static str = r#"https://mangatrend.org"#;
}

export_source!(Site<MangaTrend>);
