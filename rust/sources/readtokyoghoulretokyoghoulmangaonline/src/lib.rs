//! Read Tokyo Ghoul Re & Tokyo Ghoul Manga Online, written in Rust. A MangaCatalog site: everything comes from `sk-theme-mangacatalog`.
//! Port of the Keiyoushi extension `en.readtokyoghoulretokyoghoulmangaonline` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_mangacatalog::{Site, Theme};

pub struct ReadTokyoGhoulReTokyoGhoulMangaOnline;

impl Theme for ReadTokyoGhoulReTokyoGhoulMangaOnline {
    const NAME: &'static str = r#"Read Tokyo Ghoul Re & Tokyo Ghoul Manga Online"#;
    const BASE_URL: &'static str = r#"https://ww12.tokyoghoulre.com"#;
    const SOURCES: &'static [(&'static str, &'static str)] = &[(r#"Tokyo Ghoul"#, r#"/manga/tokyo-ghoul/"#), (r#"Tokyo Ghoul Jack"#, r#"/manga/tokyo-ghoul-jack/"#), (r#"Tokyo Ghoul: re Colored"#, r#"/manga/tokyo-ghoulre-colored/"#), (r#"Gorilla"#, r#"/manga/this-gorilla-will-die-in-1-day/"#), (r#"Zakki"#, r#"/manga/tokyo-ghoul-zakki/"#), (r#"Light Novel"#, r#"/manga/tokyo-ghoul-re-light-novels/"#), (r#"Choujin X"#, r#"/manga/choujin-x/"#), (r#"Tokyo Ghoul re"#, r#"/manga/tokyo-ghoulre/"#)];
}

export_source!(Site<ReadTokyoGhoulReTokyoGhoulMangaOnline>);
