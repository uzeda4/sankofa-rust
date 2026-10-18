//! King of Shojo, written in Rust. A MangaThemesia site: everything comes from `sk-theme-mangathemesia`.
//! Port of the Keiyoushi extension `en.kingofshojo` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_mangathemesia::{Site, Theme};

pub struct KingofShojo;

impl Theme for KingofShojo {
    const NAME: &'static str = r#"King of Shojo"#;
    const BASE_URL: &'static str = r#"https://kingofshojo.com"#;
}

export_source!(Site<KingofShojo>);
