//! WebtoonScan, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.webtoonscan` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{Site, Theme};

pub struct WebtoonScan;

impl Theme for WebtoonScan {
    const NAME: &'static str = r#"WebtoonScan"#;
    const BASE_URL: &'static str = r#"https://webtoonscan.com"#;
}

export_source!(Site<WebtoonScan>);
