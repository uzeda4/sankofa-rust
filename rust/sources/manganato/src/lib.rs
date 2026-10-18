//! Manganato, written in Rust. A MangaBox site: everything comes from `sk-theme-mangabox`.
//! Port of the Keiyoushi extension `en.manganelo` (Apache-2.0). Not ported: the message that asked to migrate titles saved
//! with an address of a former domain.

use sk_source_sdk::{export_source, RateLimit};
use sk_theme_mangabox::{Site, Theme};

pub struct Manganato;

impl Theme for Manganato {
    const NAME: &'static str = "Manganato";
    // natomanga.com now lists titles in a layout the MangaBox parser does not read (the app showed an empty list), and its
    // list and search pages sit behind a Cloudflare challenge; manganato.gg serves the layout the theme knows. The
    // extension's other addresses stay a choice in the source's Domain setting.
    const BASE_URL: &'static str = "https://www.manganato.gg";
    const MIRRORS: &'static [&'static str] = &["https://www.manganato.gg", "https://www.natomanga.com", "https://www.nelomanga.net"];
    // The search page of every address sits behind a Cloudflare challenge the app cannot always solve; the quick search of
    // these two does not (manganato.gg guards that one too).
    const SEARCH_API: &'static [&'static str] = &["https://www.natomanga.com", "https://www.nelomanga.net"];
    // Library titles saved with an address of the former domains are found again by their title (ids of the old scheme).
    // The site answers a burst (a library update finding many saved titles again) with 503; the pages and the chapter
    // API of its addresses are paced, the pictures (another host) are not.
    const RATE_LIMIT: Option<RateLimit> =
        Some(RateLimit { permits: 3, period_ms: 1000, host: String::new(), skip_path: String::new() });
    const RATE_LIMIT_HOSTS: &'static str = "www.manganato.gg,www.natomanga.com,www.nelomanga.net";
    const LEGACY_HOSTS: &'static [&'static str] = &["chapmanganato.to", "manganato.com", "readmanganato.com"];
}

export_source!(Site<Manganato>);
