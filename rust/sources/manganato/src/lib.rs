//! Manganato, written in Rust. A MangaBox site: everything comes from `sk-theme-mangabox`.
//! Port of the Keiyoushi extension `en.manganelo` (Apache-2.0). Not ported: the message that asked to migrate titles saved
//! with an address of a former domain.

use sk_source_sdk::export_source;
use sk_theme_mangabox::{Site, Theme};

pub struct Manganato;

impl Theme for Manganato {
    const NAME: &'static str = "Manganato";
    const BASE_URL: &'static str = "https://www.natomanga.com";
}

export_source!(Site<Manganato>);
