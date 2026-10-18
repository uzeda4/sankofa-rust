//! ManhwaGalaxy, written in Rust. A Yiimanga site: everything comes from `sk-theme-yiimanga`.
//! Port of the Keiyoushi extension `en.manhwagalaxy` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_yiimanga::{Site, Theme};

pub struct ManhwaGalaxy;

impl Theme for ManhwaGalaxy {
    const NAME: &'static str = r#"ManhwaGalaxy"#;
    const BASE_URL: &'static str = r#"https://manhwagalaxy.com"#;
    const MANGA_PATH: &'static str = r#"manhwa"#;
    const SEARCH_PARAM: &'static str = r#"q"#;
}

export_source!(Site<ManhwaGalaxy>);
