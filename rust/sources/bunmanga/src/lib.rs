//! Bun Manga, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.bunmanga` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{Site, Theme};

pub struct BunManga;

impl Theme for BunManga {
    const NAME: &'static str = r#"Bun Manga"#;
    const BASE_URL: &'static str = r#"https://bunmanga.com"#;
}

export_source!(Site<BunManga>);
