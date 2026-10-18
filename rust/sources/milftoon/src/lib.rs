//! Milftoon, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.milftoon` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{Site, Theme};

pub struct Milftoon;

impl Theme for Milftoon {
    const NAME: &'static str = r#"Milftoon"#;
    const BASE_URL: &'static str = r#"https://milftoon.xxx"#;
    const NO_AJAX: bool = true;
    const MANGA_SUB_STRING: &'static str = r#"comics"#;
}

export_source!(Site<Milftoon>);
