//! 4PornComic, written in Rust. A Yiimanga site: everything comes from `sk-theme-yiimanga`.
//! Port of the Keiyoushi extension `en.fourporncomic` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_yiimanga::{Site, Theme};

pub struct FourPornComic;

impl Theme for FourPornComic {
    const NAME: &'static str = r#"4PornComic"#;
    const BASE_URL: &'static str = r#"https://4porncomic.com"#;
    const MANGA_PATH: &'static str = r#"comic"#;
    const SEARCH_PARAM: &'static str = r#"q"#;
    const POPULAR_PATH: Option<&'static str> = Some(r#"porn-comics"#);
}

export_source!(Site<FourPornComic>);
