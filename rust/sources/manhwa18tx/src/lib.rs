//! Manhwa18TX, written in Rust. A Yiimanga site: everything comes from `sk-theme-yiimanga`.
//! Port of the Keiyoushi extension `en.manhwa18tx` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_yiimanga::{Site, Theme};

pub struct Manhwa18Tx;

impl Theme for Manhwa18Tx {
    const NAME: &'static str = r#"Manhwa18TX"#;
    const BASE_URL: &'static str = r#"https://manhwa18tx.com"#;
    const MANGA_PATH: &'static str = r#"manhwa"#;
    const SEARCH_PARAM: &'static str = r#"q"#;
}

export_source!(Site<Manhwa18Tx>);
