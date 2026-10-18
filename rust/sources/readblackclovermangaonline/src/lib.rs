//! Read Black Clover Manga Online, written in Rust. A MangaCatalog site: everything comes from `sk-theme-mangacatalog`.
//! Port of the Keiyoushi extension `en.readblackclovermangaonline` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_mangacatalog::{Site, Theme};

pub struct ReadBlackCloverMangaOnline;

impl Theme for ReadBlackCloverMangaOnline {
    const NAME: &'static str = r#"Read Black Clover Manga Online"#;
    const BASE_URL: &'static str = r#"https://ww10.readblackclover.com"#;
    const SOURCES: &'static [(&'static str, &'static str)] = &[(r#"Black Clover"#, r#"/manga/black-clover/"#), (r#"Fan Colored"#, r#"/manga/black-clover-colored/"#), (r#"Hungry Joker"#, r#"/manga/hungry-joker/"#), (r#"Gaiden"#, r#"/manga/black-clover-gaiden-quartet-knights/"#)];
}

export_source!(Site<ReadBlackCloverMangaOnline>);
