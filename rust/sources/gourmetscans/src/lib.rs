//! Gourmet Scans, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.gourmetscans` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{Site, Theme};

pub struct GourmetScans;

impl Theme for GourmetScans {
    const NAME: &'static str = r#"Gourmet Scans"#;
    const BASE_URL: &'static str = r#"https://gourmetsupremacy.com"#;
    const MANGA_SUB_STRING: &'static str = r#"project"#;
    const GENRE_DIRECTORY: &'static str = r#"genre"#;
    const AJAX_TEMPLATE: &'static str = r#"wp-manga/content/content-archive"#;
}

export_source!(Site<GourmetScans>);
