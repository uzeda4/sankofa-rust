//! Read One Piece Manga Online, written in Rust. A MangaCatalog site: everything comes from `sk-theme-mangacatalog`.
//! Port of the Keiyoushi extension `en.readonepiecemangaonline` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_mangacatalog::{Site, Theme};

pub struct ReadOnePieceMangaOnline;

impl Theme for ReadOnePieceMangaOnline {
    const NAME: &'static str = r#"Read One Piece Manga Online"#;
    const BASE_URL: &'static str = r#"https://ww13.readonepiece.com"#;
    const SOURCES: &'static [(&'static str, &'static str)] = &[(r#"One Piece"#, r#"/manga/one-piece/"#), (r#"Colored"#, r#"/manga/one-piece-digital-colored-comics/"#), (r#"Soma x Sanji"#, r#"/manga/shokugeki-no-sanji-one-shot/"#), (r#"OP x Toriko"#, r#"/manga/one-piece-x-toriko/"#), (r#"Party"#, r#"/manga/one-piece-party/"#), (r#"DB x OP"#, r#"/manga/dragon-ball-x-one-piece/"#), (r#"Wanted!"#, r#"/manga/wanted-one-piece/"#), (r#"Ace's Story"#, r#"/manga/one-piece-ace-s-story/"#), (r#"Omake"#, r#"/manga/one-piece-omake/"#), (r#"Vivre Card"#, r#"/manga/vivre-card-databook/"#), (r#"Pirate Recipes"#, r#"/manga/one-piece-pirate-recipes/"#), (r#"Databook"#, r#"/manga/one-piece-databook/"#), (r#"Ace's Story Manga"#, r#"/manga/one-piece-ace-story-manga/"#), (r#"OP Academy"#, r#"/manga/one-piece-academy/"#), (r#"MONSTERS"#, r#"/manga/monsters/"#), (r#"Zoro Novel"#, r#"/manga/one-piece-novel-zoro/"#), (r#"OP in Love"#, r#"/manga/one-piece-in-love/"#), (r#"Heroines"#, r#"/manga/one-piece-novel-heroines/"#)];
}

export_source!(Site<ReadOnePieceMangaOnline>);
