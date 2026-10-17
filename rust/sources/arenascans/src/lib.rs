//! Arena Scans, written in Rust. A MangaThemesia site: everything comes from `sk-theme-mangathemesia`.
//! Port of the Kotlin extension `en.arenascans` (version id 1, so its id is `md5("arena scans/en/1")`).

use sk_source_sdk::export_source;
use sk_theme_mangathemesia::{Site, Theme};

pub struct ArenaScans;

impl Theme for ArenaScans {
    const NAME: &'static str = "Arena Scans";
    const BASE_URL: &'static str = "https://arenascan.com";
}

export_source!(Site<ArenaScans>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_id_is_the_one_the_extension_had_on_the_device() {
        assert_eq!(sk_source::extension_id(ArenaScans::NAME, ArenaScans::LANG, ArenaScans::VERSION_ID), 8_861_274_191_478_178_487);
    }
}
