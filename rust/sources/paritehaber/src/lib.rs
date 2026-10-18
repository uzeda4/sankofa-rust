//! Paritehaber, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.paritehaber` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{Site, Theme};

pub struct Paritehaber;

impl Theme for Paritehaber {
    const NAME: &'static str = r#"Paritehaber"#;
    const BASE_URL: &'static str = r#"https://www.paritehaber.com"#;
}

export_source!(Site<Paritehaber>);
