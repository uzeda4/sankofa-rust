//! LinkManga, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.linkmanga` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{Site, Theme};

pub struct LinkManga;

impl Theme for LinkManga {
    const NAME: &'static str = r#"LinkManga"#;
    const BASE_URL: &'static str = r#"https://linkmanga.com"#;
}

export_source!(Site<LinkManga>);
