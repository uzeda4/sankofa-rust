//! Galaxy Manga, written in Rust. A MangaThemesia site: everything comes from `sk-theme-mangathemesia`.
//! Port of the Keiyoushi extension `en.galaxymanga` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_mangathemesia::{Site, Theme};

pub struct GalaxyManga;

impl Theme for GalaxyManga {
    const NAME: &'static str = r#"Galaxy Manga"#;
    const BASE_URL: &'static str = r#"https://galaxymanga.io"#;
}

export_source!(Site<GalaxyManga>);
