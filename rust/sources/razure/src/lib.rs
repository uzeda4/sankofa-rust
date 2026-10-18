//! Razure, written in Rust. A MangaThemesia site: everything comes from `sk-theme-mangathemesia`.
//! Port of the Keiyoushi extension `en.razure` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_mangathemesia::{Site, Theme};

pub struct Razure;

impl Theme for Razure {
    const NAME: &'static str = r#"Razure"#;
    const BASE_URL: &'static str = r#"https://razure.org"#;
    const MANGA_URL_DIRECTORY: &'static str = r#"/series"#;
    const CHAPTER_LIST_SELECTOR: &'static str = r#"#chapterlist li:not([data-num*='🔒'])"#;
    const LIST_SELECTOR: &'static str = r#".listupd .bs .bsx:not(:has(.novelabel))"#;
}

export_source!(Site<Razure>);
