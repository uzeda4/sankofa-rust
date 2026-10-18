//! Manhua Plus, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.manhuaplus` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{Site, Theme};

pub struct ManhuaPlus;

impl Theme for ManhuaPlus {
    const NAME: &'static str = r#"Manhua Plus"#;
    const BASE_URL: &'static str = r#"https://manhuaplus.com"#;
    const FILTER_NON_MANGA_ITEMS: bool = false;
    const PAGE_LIST_SELECTOR: &'static str = r#".read-container img"#;
}

export_source!(Site<ManhuaPlus>);
