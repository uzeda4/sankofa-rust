//! MangaManiacs, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.mangamaniacs` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{Site, Theme};

pub struct MangaManiacs;

impl Theme for MangaManiacs {
    const NAME: &'static str = r#"MangaManiacs"#;
    const BASE_URL: &'static str = r#"https://mangamaniacs.org"#;
}

export_source!(Site<MangaManiacs>);
