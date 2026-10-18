//! S2Manga, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.s2manga` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_source_sdk::html::El;
use sk_theme_madara::{Site, Theme, default_image};

pub struct S2Manga;

impl Theme for S2Manga {
    const NAME: &'static str = r#"S2Manga"#;
    const BASE_URL: &'static str = r#"https://s2read.com"#;

    fn image_from_element(el: &El) -> Option<String> {
        default_image(el).map(|s| s.trim().to_string())
    }
}

export_source!(Site<S2Manga>);
