//! ManhwaOnline, written in Rust. A Yiimanga site: everything comes from `sk-theme-yiimanga`.
//! Port of the Keiyoushi extension `en.manhwaonline` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_yiimanga::{Site, Theme};

pub struct ManhwaOnline;

impl Theme for ManhwaOnline {
    const NAME: &'static str = r#"ManhwaOnline"#;
    const BASE_URL: &'static str = r#"https://manhwaonline.org"#;
    const MANGA_PATH: &'static str = r#"manga"#;
    const POPULAR_PATH: Option<&'static str> = Some(r#"webtoons"#);
}

export_source!(Site<ManhwaOnline>);
