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
    // the site was redesigned: the synopsis, status and genre links no longer sit in `dl.flex` (old selectors kept for old pages)
    const DETAILS_DESCRIPTION: &'static str = "#synopsis, .synopsis";
    const DETAILS_STATUS: &'static str = "div.text-sm > span.font-medium:last-child";
    const DETAILS_GENRES: &'static str = "dl.flex:contains(Genre) a[rel=tag], dl.flex:contains(Type) a[rel=tag], div.flex-wrap > a[rel=tag]";
    const DETAILS_INFO: &'static str = "dl.flex";
    const DETAILS_INFO_TITLE: &'static str = "dt";
    const DETAILS_INFO_DESCRIPTION: &'static str = "dd";
    const PAGE_SELECTOR: &'static str = ".post-body, .check-box";
}

export_source!(Site<MurimScan>);

#[cfg(test)]
mod tests {
    use super::*;
    use sk_source_sdk::{set_mock_http, HttpResponse, Manga, Source};

    #[test]
    fn the_id_is_the_one_the_extension_had_on_the_device() {
        assert_eq!(sk_source::extension_id(MurimScan::NAME, MurimScan::LANG, MurimScan::VERSION_ID), 5_524_031_489_110_351_257);
    }

    #[test]
    fn the_redesigned_title_page_gives_synopsis_status_and_genres() {
        set_mock_http(|r| Ok(HttpResponse { status: 200, body: include_str!("../tests/fixtures/detail.html").into(), url: r.url.clone(), ..Default::default() }));
        let m = Site::<MurimScan>::default().details(&Manga { url: "/2026/04/the-lone-necromancer.html".into(), title: "The Lone Necromancer".into(), ..Default::default() }).unwrap();
        assert_eq!(m.status, 1);
        assert_eq!(m.genre, ["Action", "Isekai", "System"]);
        assert!(m.description.unwrap().starts_with("Seongwu is a former special forces soldier"));
    }
}
