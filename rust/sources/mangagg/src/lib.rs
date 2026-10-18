//! MangaGG, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.mangagg` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_source_sdk::html::El;
use sk_theme_madara::{default_image, ChapterMode, Site, Theme};

pub struct MangaGg;

impl Theme for MangaGg {
    const NAME: &'static str = "MangaGG";
    const BASE_URL: &'static str = "https://mangagg.com";
    const MANGA_SUB_STRING: &'static str = "comic";
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjaxPaginated;
    const CHAPTER_DATE_PATTERN: &'static str = "MM/dd/yyyy";

    /// The first lazy-loading attribute that has an address (trimmed: a leading space breaks relative links).
    fn image_from_element(el: &El) -> Option<String> {
        let url = ["data-src", "data-lazy-src", "data-cfsrc", "data-manga-src", "src"]
            .iter()
            .map(|a| el.attr(a).trim().to_string())
            .find(|u| !u.is_empty())
            .unwrap_or_default();
        if url.starts_with("http") {
            Some(url)
        } else {
            default_image(el).map(|s| s.trim().to_string())
        }
    }
}

export_source!(Site<MangaGg>);
