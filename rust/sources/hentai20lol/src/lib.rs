//! Hentai20 (Unoriginal), written in Rust. A Yiimanga site: everything comes from `sk-theme-yiimanga`.
//! Port of the Keiyoushi extension `en.hentai20lol` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_yiimanga::{Site, Theme};

pub struct Hentai20Lol;

impl Theme for Hentai20Lol {
    const NAME: &'static str = r#"Hentai20 (Unoriginal)"#;
    const BASE_URL: &'static str = r#"https://hentai20.lol"#;
    const MANGA_PATH: &'static str = r#"manga"#;
    const POPULAR_PATH: Option<&'static str> = Some(r#"manga-lists"#);
    const SUPPORTS_FILTER_FETCHING: bool = true;
}

export_source!(Site<Hentai20Lol>);
