//! YaoiScan, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.yaoiscan` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{Site, Theme};

pub struct YaoiScan;

impl Theme for YaoiScan {
    const NAME: &'static str = r#"YaoiScan"#;
    const BASE_URL: &'static str = r#"https://yaoiscan.com"#;
}

export_source!(Site<YaoiScan>);
