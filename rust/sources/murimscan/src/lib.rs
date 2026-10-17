//! MurimScan, written in Rust. A ZeistManga site: see `sk-theme-zeist`. Port of the Kotlin extension
//! `en.murimscan` (its id is `md5("murimscan/en/2")`, version 2 since it moved from Madara).

use sk_source_sdk::export_source;
use sk_theme_zeist::{Site, Theme};

pub struct MurimScan;

impl Theme for MurimScan {
    const NAME: &'static str = "MurimScan";
    const BASE_URL: &'static str = "https://www.murimscans.site";
    const VERSION_ID: u32 = 2;
    const SUPPORTS_LATEST: bool = false;

    const DETAILS_SELECTOR: &'static str = "main";
    const DETAILS_GENRES: &'static str = "dl.flex:contains(Genre) a[rel=tag], dl.flex:contains(Type) a[rel=tag]";
    const DETAILS_INFO: &'static str = "dl.flex";
    const DETAILS_INFO_TITLE: &'static str = "dt";
    const DETAILS_INFO_DESCRIPTION: &'static str = "dd";
    const PAGE_SELECTOR: &'static str = ".post-body, .check-box";
}

export_source!(Site<MurimScan>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_id_is_the_one_the_extension_had_on_the_device() {
        assert_eq!(sk_source::extension_id(MurimScan::NAME, MurimScan::LANG, MurimScan::VERSION_ID), 5_524_031_489_110_351_257);
    }
}
