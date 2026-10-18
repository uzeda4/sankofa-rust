//! Rizz Comic (unoriginal), written in Rust. A MangaThemesia site: everything comes from `sk-theme-mangathemesia`.
//! Port of the Keiyoushi extension `en.rizzcomicunoriginal` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_mangathemesia::{Site, Theme};

pub struct RizzComicUnoriginal;

impl Theme for RizzComicUnoriginal {
    const NAME: &'static str = r#"Rizz Comic (unoriginal)"#;
    const BASE_URL: &'static str = r#"https://rizzcomic.com"#;
}

export_source!(Site<RizzComicUnoriginal>);
