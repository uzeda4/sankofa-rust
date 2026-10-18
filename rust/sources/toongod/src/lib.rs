//! ToonGod, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.toongod` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{Site, Theme};

pub struct ToonGod;

impl Theme for ToonGod {
    const NAME: &'static str = r#"ToonGod"#;
    const BASE_URL: &'static str = r#"https://www.toongod.org"#;
    const NO_AJAX: bool = true;
    const CHAPTER_DATE_PATTERN: &'static str = r#"d MMM yyyy"#;
    const MANGA_SUB_STRING: &'static str = r#"webtoons"#;
}

export_source!(Site<ToonGod>);
