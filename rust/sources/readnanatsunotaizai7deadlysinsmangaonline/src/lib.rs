//! Read Nanatsu no Taizai 7 Deadly Sins Manga Online, written in Rust. A MangaCatalog site: everything comes from `sk-theme-mangacatalog`.
//! Port of the Keiyoushi extension `en.readnanatsunotaizai7deadlysinsmangaonline` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_mangacatalog::{Site, Theme};

pub struct ReadNanatsuNoTaizai7DeadlySinsMangaOnline;

impl Theme for ReadNanatsuNoTaizai7DeadlySinsMangaOnline {
    const NAME: &'static str = r#"Read Nanatsu no Taizai 7 Deadly Sins Manga Online"#;
    const BASE_URL: &'static str = r#"https://ww8.read7deadlysins.com"#;
    const SOURCES: &'static [(&'static str, &'static str)] = &[(r#"Four Horsemen of the Apocalypse"#, r#"/manga/four-horsemen-of-the-apocalypse/"#), (r#"7DS: School"#, r#"/manga/mayoe-nanatsu-no-taizai-gakuen/"#), (r#"7DS:7 Days"#, r#"/manga/nanatsu-no-taizai-seven-days/"#), (r#"7DS:Vampires"#, r#"/manga/nanatsu-no-taizai-vampires-of-edinburgh/"#), (r#"Queen of Altar"#, r#"/manga/the-queen-of-the-altar/"#), (r#"7DS: 7 Colors"#, r#"/manga/nanatsu-no-taizai-nanairo-no-tsuioku/"#), (r#"7DS x FT"#, r#"/manga/fairy-tail-x-nanatsu-no-taizai-christmas-special/"#), (r#"Kongou Banchou"#, r#"/manga/kongou-banchou/"#), (r#"7DS:7 Scars"#, r#"/manga/nanatsu-no-taizai-the-seven-scars-which-they-left-behind/"#), (r#"7 Deadly Sins"#, r#"/manga/nanatsu-no-taizai/"#), (r#"Mokushiroku no Yonkishi"#, r#"/manga/four-horsemen-of-the-apocalypse/"#)];
}

export_source!(Site<ReadNanatsuNoTaizai7DeadlySinsMangaOnline>);
