//! Spmanhwa, written in Rust. A Madara site: everything comes from `sk-theme-madara`.
//! Port of the Keiyoushi extension `en.spmanhwa` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_madara::{Site, Theme};

pub struct Spmanhwa;

impl Theme for Spmanhwa {
    const NAME: &'static str = r#"Spmanhwa"#;
    const BASE_URL: &'static str = r#"https://spmanhwa.online"#;
}

export_source!(Site<Spmanhwa>);
