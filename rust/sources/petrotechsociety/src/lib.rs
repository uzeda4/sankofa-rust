//! Petrotechsociety, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.petrotechsociety` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{Site, Theme};

pub struct Petrotechsociety;

impl Theme for Petrotechsociety {
    const NAME: &'static str = r#"Petrotechsociety"#;
    const BASE_URL: &'static str = r#"https://www.petrotechsociety.org"#;
    const CHAPTER_DATE_PATTERN: &'static str = r#"dd/MM/yyyy"#;
}

export_source!(Site<Petrotechsociety>);
