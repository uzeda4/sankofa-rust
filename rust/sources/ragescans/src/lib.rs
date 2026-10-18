//! Rage Scans, written in Rust. A MangaThemesia site: everything comes from `sk-theme-mangathemesia`.
//! Port of the Keiyoushi extension `en.ragescans` (Apache-2.0). Not ported: the extension removed the `#comments` block
//! before reading a title's details, so a comment could not be mistaken for a detail.

use sk_source_sdk::export_source;
use sk_theme_mangathemesia::{Site, Theme};

pub struct RageScans;

impl Theme for RageScans {
    const NAME: &'static str = "Rage Scans";
    const BASE_URL: &'static str = "https://ragescans.com";
    // locked chapters (they open a modal instead of a reader) are left out
    const CHAPTER_LIST_SELECTOR: &'static str = "li:has(.chbox .eph-num):not(:has([data-bs-target='#lockedChapterModal']))";
}

export_source!(Site<RageScans>);
