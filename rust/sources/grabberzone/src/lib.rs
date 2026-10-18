//! Grabber Zone, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `all.grabberzone` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{Site, Theme};

pub struct GrabberZone;

impl Theme for GrabberZone {
    const NAME: &'static str = "Grabber Zone";
    const LANG: &'static str = "all";
    const BASE_URL: &'static str = "https://grabber.zone";
    const MANGA_SUB_STRING: &'static str = "comics";
    const CHAPTER_DATE_PATTERN: &'static str = "dd.MM.yyyy";
    // the chapter's name is the second link of its row
    const CHAPTER_NAME_SELECTOR: Option<&'static str> = Some("a + a");
}

export_source!(Site<GrabberZone>);
