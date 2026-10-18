//! Mangabat, written in Rust. A MangaBox site: everything comes from `sk-theme-mangabox`.
//! Port of the Keiyoushi extension `en.mangabat` (Apache-2.0). Not ported: the message that asked to migrate titles saved
//! with an address of a former domain.

use sk_source_sdk::export_source;
use sk_theme_mangabox::{Site, Theme};

pub struct Mangabat;

impl Theme for Mangabat {
    const NAME: &'static str = "Mangabat";
    const BASE_URL: &'static str = "https://www.mangabats.com";
}

export_source!(Site<Mangabat>);
