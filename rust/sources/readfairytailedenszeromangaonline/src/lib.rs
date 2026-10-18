//! Read Fairy Tail & Edens Zero Manga Online, written in Rust. A MangaCatalog site: everything comes from `sk-theme-mangacatalog`.
//! Port of the Keiyoushi extension `en.readfairytailedenszeromangaonline` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_mangacatalog::{Site, Theme};

pub struct ReadFairyTailEdensZeroMangaOnline;

impl Theme for ReadFairyTailEdensZeroMangaOnline {
    const NAME: &'static str = r#"Read Fairy Tail & Edens Zero Manga Online"#;
    const BASE_URL: &'static str = r#"https://ww9.readfairytail.com"#;
    const SOURCES: &'static [(&'static str, &'static str)] = &[(r#"Eden's Zero"#, r#"/manga/edens-zero/"#), (r#"Fairy Tail"#, r#"/manga/fairy-tail/"#), (r#"FT Zero"#, r#"/manga/fairy-tail-zero/"#), (r#"FT City Hero"#, r#"/manga/fairy-tail-city-hero/"#), (r#"Hero’s"#, r#"/manga/heros/"#), (r#"FT Happy Adv"#, r#"/manga/fairy-tail-happys-grand-adventure/"#), (r#"FT 100 Year"#, r#"/manga/fairy-tail-100-years-quest/"#), (r#"FT Ice Trail"#, r#"/manga/fairy-tail-ice-trail/"#), (r#"FT x Taizai"#, r#"/manga/fairy-tail-x-nanatsu-no-taizai-christmas-special/"#), (r#"Parasyte x FT"#, r#"/manga/parasyte-x-fairy-tail/"#), (r#"Gaiden 1"#, r#"/manga/fairy-tail-gaiden-raigo-issen/"#), (r#"FT x Rave"#, r#"/manga/fairy-tail-x-rave/"#), (r#"Monster Hunter"#, r#"/manga/monster-hunter-orage/"#), (r#"Rave Master"#, r#"/manga/rave-master/"#), (r#"Dead Rock"#, r#"/manga/dead-rock/"#), (r#"Fairy Girls"#, r#"/manga/fairy-girls/"#), (r#"Gaiden 4"#, r#"/manga/fairy-tail-gaiden-raigo-issen/"#), (r#"Gaiden 2"#, r#"/manga/fairy-tail-gaiden-kengami-no-souryuu/"#), (r#"Gaiden 3"#, r#"/manga/fairy-tail-gaiden-road-knight/"#), (r#"FT x 7DS"#, r#"/manga/fairy-tail-x-nanatsu-no-taizai-christmas-special/"#)];
}

export_source!(Site<ReadFairyTailEdensZeroMangaOnline>);
