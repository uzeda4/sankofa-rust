//! Asmodeus Scans, written in Rust. A Keyoapp (V2) site: see `sk-theme-keyoapp`. Port of the Kotlin extension
//! `en.asmotoon` (version id 2).

use sk_source_sdk::export_source;
use sk_theme_keyoapp::{Site, Theme};

pub struct Asmotoon;

impl Theme for Asmotoon {
    const NAME: &'static str = "Asmodeus Scans";
    const BASE_URL: &'static str = "https://asmotoon.com";
    const VERSION_ID: u32 = 2;
}

export_source!(Site<Asmotoon>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_id_is_the_one_the_extension_had_on_the_device() {
        assert_eq!(sk_source::extension_id(Asmotoon::NAME, Asmotoon::LANG, Asmotoon::VERSION_ID), 2_076_371_496_779_298_312);
    }
}
