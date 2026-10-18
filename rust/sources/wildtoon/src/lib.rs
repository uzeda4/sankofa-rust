//! Wildtoon, written in Rust. A MangaThemesia site: everything comes from `sk-theme-mangathemesia`.
//! Port of the Keiyoushi extension `en.wildtoon` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_mangathemesia::{Site, Theme};

pub struct Wildtoon;

impl Theme for Wildtoon {
    const NAME: &'static str = r#"Wildtoon"#;
    const BASE_URL: &'static str = r#"https://wildtoon.com"#;
    const MANGA_URL_DIRECTORY: &'static str = r#"/webtoon"#;
}

export_source!(Site<Wildtoon>);
