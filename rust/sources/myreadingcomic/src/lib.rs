//! MyReadingComic, written in Rust. A Yiimanga site: everything comes from `sk-theme-yiimanga`.
//! Port of the Keiyoushi extension `en.myreadingcomic` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_yiimanga::{Site, Theme};

pub struct MyReadingComic;

impl Theme for MyReadingComic {
    const NAME: &'static str = r#"MyReadingComic"#;
    const BASE_URL: &'static str = r#"https://myreadingcomic.com"#;
    const MANGA_PATH: &'static str = r#"comic"#;
    const SEARCH_PARAM: &'static str = r#"q"#;
    const POPULAR_PATH: Option<&'static str> = Some(r#"porn-comics"#);
}

export_source!(Site<MyReadingComic>);
