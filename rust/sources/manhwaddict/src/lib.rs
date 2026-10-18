//! Manhwaddict, written in Rust. A Yiimanga site: everything comes from `sk-theme-yiimanga`.
//! Port of the Keiyoushi extension `en.manhwaddict` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_yiimanga::{Site, Theme};

pub struct Manhwaddict;

impl Theme for Manhwaddict {
    const NAME: &'static str = r#"Manhwaddict"#;
    const BASE_URL: &'static str = r#"https://manhwaddict.com"#;
    const MANGA_PATH: &'static str = r#"manga"#;
    const SEARCH_PARAM: &'static str = r#"q"#;
}

export_source!(Site<Manhwaddict>);
