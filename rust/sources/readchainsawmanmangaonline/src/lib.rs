//! Read Chainsaw Man Manga Online, written in Rust. A MangaCatalog site: everything comes from `sk-theme-mangacatalog`.
//! Port of the Keiyoushi extension `en.readchainsawmanmangaonline` (Apache-2.0).

use sk_source_sdk::export_source;
use sk_theme_mangacatalog::{Site, Theme};

pub struct ReadChainsawManMangaOnline;

impl Theme for ReadChainsawManMangaOnline {
    const NAME: &'static str = r#"Read Chainsaw Man Manga Online"#;
    const BASE_URL: &'static str = r#"https://ww6.readchainsawman.com"#;
    const SOURCES: &'static [(&'static str, &'static str)] = &[(r#"Chainsaw Man"#, r#"/manga/chainsaw-man/"#), (r#"17-21"#, r#"/manga/17-21-fujimoto-tatsuki-tanpenshuu/"#), (r#"Fire Punch"#, r#"/manga/fire-punch/"#), (r#"Nayuta"#, r#"/manga/yogen-no-nayuta/"#), (r#"Look Back"#, r#"/manga/look-back/"#), (r#"Light Novel"#, r#"/manga/chainsaw-man-buddy-stories/"#), (r#"Colored"#, r#"/manga/chainsaw-man-colored/"#), (r#"Listen to Song"#, r#"/manga/futsuu-ni-kiite-kure/"#), (r#"Goodbye, Eri"#, r#"/manga/sayonara-eri-goodbye-eri/"#), (r#"22-26"#, r#"/manga/22-26-fujimoto-tatsuki-tanpenshuu/"#), (r#"Chainsaw Man Colored"#, r#"/manga/chainsaw-man-colored/"#), (r#"Chainsaw Man: Buddy Stories"#, r#"/manga/chainsaw-man-buddy-stories/"#)];
}

export_source!(Site<ReadChainsawManMangaOnline>);
