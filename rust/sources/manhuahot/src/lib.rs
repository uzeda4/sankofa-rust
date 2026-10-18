//! ManhuaHot, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.manhuahot` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{Site, Theme};

pub struct ManhuaHot;

impl Theme for ManhuaHot {
    const NAME: &'static str = r#"ManhuaHot"#;
    const BASE_URL: &'static str = r#"https://manhuahot.com"#;
}

export_source!(Site<ManhuaHot>);
