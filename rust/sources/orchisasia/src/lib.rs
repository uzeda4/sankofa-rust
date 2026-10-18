//! Orchisasia, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.orchisasia` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{Site, Theme};

pub struct Orchisasia;

impl Theme for Orchisasia {
    const NAME: &'static str = r#"Orchisasia"#;
    const BASE_URL: &'static str = r#"https://www.orchisasia.org"#;
    const MANGA_SUB_STRING: &'static str = r#"comic"#;
}

export_source!(Site<Orchisasia>);
