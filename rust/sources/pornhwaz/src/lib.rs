//! Pornhwaz, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.pornhwaz` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{Site, Theme};

pub struct Pornhwaz;

impl Theme for Pornhwaz {
    const NAME: &'static str = r#"Pornhwaz"#;
    const BASE_URL: &'static str = r#"https://www.pornhwaz.com"#;
}

export_source!(Site<Pornhwaz>);
