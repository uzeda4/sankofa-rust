//! Read Jujutsu Kaisen Manga Online, written in Rust. A MangaCatalog site: everything comes from `sk-theme-mangacatalog`.
//! Port of the Keiyoushi extension `en.readjujutsukaisenmangaonline` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_mangacatalog::{Site, Theme};

pub struct ReadJujutsuKaisenMangaOnline;

impl Theme for ReadJujutsuKaisenMangaOnline {
    const NAME: &'static str = r#"Read Jujutsu Kaisen Manga Online"#;
    const BASE_URL: &'static str = r#"https://ww6.readjujutsukaisen.com"#;
    const SOURCES: &'static [(&'static str, &'static str)] = &[(r#"Jujutsu Kaisen"#, r#"/manga/jujutsu-kaisen/"#), (r#"Jujutsu Kaisen 0"#, r#"/manga/jujutsu-kaisen-0/"#), (r#"JJK Colored"#, r#"/manga/jujutsu-kaisen-colored/"#), (r#"Fan Scan"#, r#"/manga/jujutsu-kaisen-fan-scan/"#), (r#"JJK Light Novel"#, r#"/manga/jujutsu-kaisen-first-light-novel/"#), (r#"2nd Light Novel"#, r#"/manga/jujutsu-kaisen-second-light-novel/"#), (r#"No.9"#, r#"/manga/no-9/"#), (r#"Fanbook"#, r#"/manga/jujutsu-kaisen-official-fanbook/"#)];
}

export_source!(Site<ReadJujutsuKaisenMangaOnline>);
