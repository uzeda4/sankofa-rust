//! Aqua Manga, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.aquamanga` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{Site, Theme};

pub struct AquaManga;

impl Theme for AquaManga {
    const NAME: &'static str = r#"Aqua Manga"#;
    const BASE_URL: &'static str = r#"https://aquareader.org"#;
    const NO_AJAX: bool = true;
    const SUPPORTS_POST_ID: bool = false;
    const ARCHIVE_SELECTOR: &'static str = r#".aqua-archive-card"#;
    const ARCHIVE_URL_SELECTOR: &'static str = r#".aqua-archive-card__title a"#;
    const NEXT_PAGE_SELECTOR: &'static str = r#"a.next"#;
    const DETAILS_TITLE: &'static str = r#".aqua-series-info__title"#;
    const DETAILS_THUMBNAIL: &'static str = r#".aqua-series-cover__img"#;
    const DETAILS_DESCRIPTION: &'static str = r#".aqua-series-synopsis"#;
    const DETAILS_STATUS: &'static str = r#".aqua-series-meta__status"#;
    const DETAILS_GENRE: &'static str = r#".aqua-series-genre-pill"#;
    const DETAILS_AUTHOR: &'static str = r#".aqua-series-info__creator-value a"#;
    const DETAILS_ARTIST: &'static str = r#".aqua-series-info__creator-value a"#;
    const CHAPTER_LIST_SELECTOR: &'static str = r#".aqua-ch-item"#;
    const CHAPTER_URL_SELECTOR: &'static str = r#"a"#;
    const CHAPTER_NAME_SELECTOR: Option<&'static str> = Some(r#".aqua-ch-item__name"#);
    const CHAPTER_DATE_SELECTOR: &'static str = r#".aqua-ch-item__time"#;
}

export_source!(Site<AquaManga>);
