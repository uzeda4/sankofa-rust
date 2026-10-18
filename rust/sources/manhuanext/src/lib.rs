//! Manhuanext, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.manhuanext` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{Site, Theme};

pub struct Manhuanext;

impl Theme for Manhuanext {
    const NAME: &'static str = r#"Manhuanext"#;
    const BASE_URL: &'static str = r#"https://manhuanext.com"#;
}

export_source!(Site<Manhuanext>);
