//! Aunt Manhwa, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.auntmanhwa` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{Site, Theme};

pub struct AuntManhwa;

impl Theme for AuntManhwa {
    const NAME: &'static str = r#"Aunt Manhwa"#;
    const BASE_URL: &'static str = r#"https://auntmanhwa.com"#;
}

export_source!(Site<AuntManhwa>);
