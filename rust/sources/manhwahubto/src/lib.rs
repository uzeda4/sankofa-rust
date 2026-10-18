//! ManhwaHub.to, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.manhwahubto` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{Site, Theme};

pub struct ManhwaHubTo;

impl Theme for ManhwaHubTo {
    const NAME: &'static str = r#"ManhwaHub.to"#;
    const BASE_URL: &'static str = r#"https://manhwahub.to"#;
    const MANGA_SUB_STRING: &'static str = r#"manhwa"#;
    const GENRE_DIRECTORY: &'static str = r#"genre"#;
}

export_source!(Site<ManhwaHubTo>);
