//! Manhua Zonghe, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.manhuazonghe` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{Site, Theme};

pub struct ManhuaZonghe;

impl Theme for ManhuaZonghe {
    const NAME: &'static str = r#"Manhua Zonghe"#;
    const BASE_URL: &'static str = r#"https://www.manhuazonghe.com"#;
    const FILTER_NON_MANGA_ITEMS: bool = false;
    const MANGA_SUB_STRING: &'static str = r#"manhua"#;
}

export_source!(Site<ManhuaZonghe>);
