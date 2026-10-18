//! Read Kingdom Manga Online, written in Rust. A MangaCatalog site: everything comes from `sk-theme-mangacatalog`.
//! Port of the Keiyoushi extension `en.readkingdommangaonline` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_mangacatalog::{Site, Theme};

pub struct ReadKingdomMangaOnline;

impl Theme for ReadKingdomMangaOnline {
    const NAME: &'static str = r#"Read Kingdom Manga Online"#;
    const BASE_URL: &'static str = r#"https://ww6.readkingdom.com"#;
    const SOURCES: &'static [(&'static str, &'static str)] = &[(r#"Kingdom"#, r#"/manga/kingdom/"#), (r#"Li Mu"#, r#"/manga/li-mu/"#), (r#"Meng Wu & Chu Zi"#, r#"/manga/meng-wu-and-chu-zi-one-shot/"#)];
}

export_source!(Site<ReadKingdomMangaOnline>);
