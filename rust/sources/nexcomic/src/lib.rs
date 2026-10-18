//! NexComic, written in Rust. A MangaThemesia site: everything comes from `sk-theme-mangathemesia`.
//! Port of the Keiyoushi extension `en.nexcomic` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_mangathemesia::{Site, Theme};

pub struct NexComic;

impl Theme for NexComic {
    const NAME: &'static str = r#"NexComic"#;
    const BASE_URL: &'static str = r#"https://nexcomic.com"#;
}

export_source!(Site<NexComic>);
