//! Hentai Desi, written in Rust. A MangaThemesia site: everything comes from `sk-theme-mangathemesia`.
//! Port of the Keiyoushi extension `en.hentaidesi` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_mangathemesia::{Site, Theme};

pub struct HentaiDesi;

impl Theme for HentaiDesi {
    const NAME: &'static str = r#"Hentai Desi"#;
    const BASE_URL: &'static str = r#"https://hentaidesi.com"#;
}

export_source!(Site<HentaiDesi>);
