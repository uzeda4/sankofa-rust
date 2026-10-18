//! YaoiScan (Unoriginal), written in Rust. A Yiimanga site: everything comes from `sk-theme-yiimanga`.
//! Port of the Keiyoushi extension `en.yaoiscanme` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_yiimanga::{Site, Theme};

pub struct YaoiScanMe;

impl Theme for YaoiScanMe {
    const NAME: &'static str = r#"YaoiScan (Unoriginal)"#;
    const BASE_URL: &'static str = r#"https://yaoiscan.me"#;
    const MANGA_PATH: &'static str = r#"read"#;
    const SEARCH_PARAM: &'static str = r#"q"#;
}

export_source!(Site<YaoiScanMe>);
