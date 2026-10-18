//! Manga District, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.mangadistrict` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{Site, Theme};

pub struct MangaDistrict;

impl Theme for MangaDistrict {
    const NAME: &'static str = r#"Manga District"#;
    const BASE_URL: &'static str = r#"https://mangadistrict.com"#;
    const NO_AJAX: bool = true;
}

export_source!(Site<MangaDistrict>);
