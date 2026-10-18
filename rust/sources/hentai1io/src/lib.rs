//! Hentai1.io, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.hentai1io` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{Site, Theme};

pub struct Hentai1Io;

impl Theme for Hentai1Io {
    const NAME: &'static str = r#"Hentai1.io"#;
    const BASE_URL: &'static str = r#"https://hentai1.io"#;
    const NO_AJAX: bool = true;
}

export_source!(Site<Hentai1Io>);
