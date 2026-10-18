//! Read Solo Leveling Manga Manhwa Online, written in Rust. A MangaCatalog site: everything comes from `sk-theme-mangacatalog`.
//! Port of the Keiyoushi extension `en.readsololevelingmangamanhwaonline` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_mangacatalog::{Site, Theme};

pub struct ReadSoloLevelingMangaManhwaOnline;

impl Theme for ReadSoloLevelingMangaManhwaOnline {
    const NAME: &'static str = r#"Read Solo Leveling Manga Manhwa Online"#;
    const BASE_URL: &'static str = r#"https://ww4.readsololeveling.org"#;
    const SOURCES: &'static [(&'static str, &'static str)] = &[(r#"Solo Leveling Manhwa"#, r#"/manga/solo-leveling/"#), (r#"Solo Leveling Light Novel"#, r#"/manga/solo-leveling-light-novel/"#), (r#"Solo Leveling : Ragnarok"#, r#"/manga/solo-leveling-ragnarok/"#), (r#"SL: Ragnarok Novel"#, r#"/manga/solo-leveling-ragnarok-novel/"#)];
}

export_source!(Site<ReadSoloLevelingMangaManhwaOnline>);
