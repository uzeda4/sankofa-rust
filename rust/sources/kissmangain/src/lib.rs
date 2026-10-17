//! Kissmanga.in, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Kotlin extension `en.kissmangain` (version id 1, so its id is `md5("kissmanga.in/en/1")`).

use sk_source_sdk::export_source;
use sk_source_sdk::html::El;
use sk_theme_madara::{default_image, ChapterMode, Site, Theme};

pub struct KissmangaIn;

impl Theme for KissmangaIn {
    const NAME: &'static str = "Kissmanga.in";
    const BASE_URL: &'static str = "https://kissmanga.in";
    const MANGA_SUB_STRING: &'static str = "kissmanga";
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;

    fn image_from_element(el: &El) -> Option<String> {
        default_image(el).map(|s| s.trim().to_string())
    }
}

export_source!(Site<KissmangaIn>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_id_is_the_one_the_extension_had_on_the_device() {
        assert_eq!(sk_source::extension_id(KissmangaIn::NAME, KissmangaIn::LANG, KissmangaIn::VERSION_ID), 3_112_384_885_023_225_761);
    }
}
