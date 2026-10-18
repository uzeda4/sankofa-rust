//! AllPornComic, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.allporncomic` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{Site, Theme};

pub struct AllPornComic;

impl Theme for AllPornComic {
    const NAME: &'static str = r#"AllPornComic"#;
    const BASE_URL: &'static str = r#"https://allporncomic.com"#;
    const NO_AJAX: bool = true;
    const MANGA_SUB_STRING: &'static str = r#"porncomic"#;
    const SUPPORTS_POST_ID: bool = false;
}

export_source!(Site<AllPornComic>);
