//! Zinmanga, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.zinmanga` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{Site, Theme};

pub struct Zinmanga;

impl Theme for Zinmanga {
    const NAME: &'static str = r#"Zinmanga"#;
    const BASE_URL: &'static str = r#"https://mangazin.org"#;
    const FILTER_NON_MANGA_ITEMS: bool = false;
}

export_source!(Site<Zinmanga>);
