//! Data that crosses the source/host boundary as JSON.

use serde::{Deserialize, Serialize};
use serde_json::Value;

fn one() -> u32 {
    1
}

fn unknown_number() -> f32 {
    -1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RateLimit {
    /// Requests allowed per period.
    pub permits: u32,
    pub period_ms: u64,
    /// Limit only requests to these hosts (`api.site.com,site.com`, each with its own count); empty means every request,
    /// images included.
    #[serde(default)]
    pub host: String,
    /// Requests whose path starts with this are not limited (`/wp-content/uploads/`: the pictures of a WordPress site).
    #[serde(default)]
    pub skip_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info {
    pub name: String,
    /// Language code or `all`. A package installed for several languages reports the one it was installed for
    /// (`sk_source_sdk::lang()`).
    pub lang: String,
    pub base_url: String,
    /// Other addresses of the same site. When there are any the app adds a "Domain" setting (key `mirror`) and
    /// `sk_source_sdk::base_url()` returns the one the user chose.
    #[serde(default)]
    pub mirrors: Vec<String>,
    #[serde(default)]
    pub supports_latest: bool,
    /// Stable numeric id. Leave 0 and the host derives it the way extensions did (MD5 of
    /// `name.lowercase()/lang/version_id`), so a source converted from an extension keeps its library entries.
    #[serde(default)]
    pub id: i64,
    #[serde(default = "one")]
    pub version_id: u32,
    /// The source lists genres and the like from the site (`Source::filter_data`); the host caches them.
    #[serde(default)]
    pub supports_filter_fetching: bool,
    /// The source implements `Source::related`.
    #[serde(default)]
    pub supports_related: bool,
    #[serde(default)]
    pub rate_limit: Option<RateLimit>,
    /// Settings the user can change; their values arrive in every request (`pref_*`).
    #[serde(default)]
    pub prefs: Vec<PrefDef>,
    /// Headers added to every request, covers and chapter images included.
    #[serde(default)]
    pub headers: Vec<(String, String)>,
    /// Cookies the app sets in its cookie jar before the first request (an age check, a reading mode).
    #[serde(default)]
    pub cookies: Vec<CookieDef>,
    /// The source implements `Source::manga_url` / `Source::chapter_url` (the address to open in a browser differs from
    /// what is stored).
    #[serde(default)]
    pub custom_urls: bool,
}

impl Default for Info {
    /// `version_id` is 1 here as it is when a module's answer leaves it out, so `..Default::default()` keeps the id an extension of the same name had.
    fn default() -> Self {
        Self {
            name: String::new(),
            lang: String::new(),
            base_url: String::new(),
            mirrors: Vec::new(),
            supports_latest: false,
            id: 0,
            version_id: 1,
            supports_filter_fetching: false,
            supports_related: false,
            rate_limit: None,
            prefs: Vec::new(),
            headers: Vec::new(),
            cookies: Vec::new(),
            custom_urls: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieDef {
    /// Host name the cookie belongs to (`fanfox.net`, or a subdomain such as `m.fanfox.net`).
    pub host: String,
    pub name: String,
    pub value: String,
}

impl CookieDef {
    pub fn new(host: &str, name: &str, value: &str) -> Self {
        Self { host: host.into(), name: name.into(), value: value.into() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Manga {
    /// Path or URL that identifies the title inside this source; passed back to `details` and `chapters`.
    pub url: String,
    pub title: String,
    #[serde(default)]
    pub thumbnail_url: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub artist: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub genre: Vec<String>,
    /// Same values as `SManga`: 0 unknown, 1 ongoing, 2 completed, 3 licensed, 4 publishing finished,
    /// 5 cancelled, 6 on hiatus.
    #[serde(default)]
    pub status: u8,
    /// True once `description` and the rest were read from the title's own page; lists leave it false.
    #[serde(default)]
    pub initialized: bool,
    /// Anything the source wants back with this title later (ids found on the page, a slug). A JSON object.
    #[serde(default)]
    pub memo: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MangaPage {
    pub mangas: Vec<Manga>,
    pub has_next: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub url: String,
    pub name: String,
    /// `-1` when unknown, as in `SChapter`.
    #[serde(default = "unknown_number")]
    pub chapter_number: f32,
    #[serde(default)]
    pub scanlator: Option<String>,
    /// Milliseconds since the Unix epoch; 0 when unknown.
    #[serde(default)]
    pub date_upload: i64,
    #[serde(default)]
    pub memo: Option<Value>,
}

impl Default for Chapter {
    fn default() -> Self {
        Self { url: String::new(), name: String::new(), chapter_number: -1.0, scanlator: None, date_upload: 0, memo: None }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Page {
    pub index: u32,
    /// Page URL when the image URL must be resolved later; usually empty.
    #[serde(default)]
    pub url: String,
    pub image_url: String,
    /// Headers for this image only (a `Referer`, say), on top of `Info::headers`.
    #[serde(default)]
    pub headers: Vec<(String, String)>,
    /// Edits the app applies to the downloaded image, in order (see [`ImageOp`]).
    #[serde(default)]
    pub ops: Vec<ImageOp>,
    /// Other origins (`https://cdn2.example.org`) that serve the same paths. When an image fails, the app retries it
    /// on each of these in order, for every address it downloads for this page.
    #[serde(default)]
    pub mirrors: Vec<String>,
}

impl Page {
    pub fn new(index: u32, image_url: impl Into<String>) -> Self {
        Self { index, image_url: image_url.into(), ..Default::default() }
    }
}

/// Work on an image that is cheap in the app (a `Bitmap`) and far too slow in WebAssembly.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum ImageOp {
    /// Download `urls` (this page's `image_url` is the first) and stack them top to bottom into one image.
    /// `width`/`height` are the sizes of the result.
    Merge { urls: Vec<String>, width: u32, height: u32 },
    /// Cut the image into `tiles` and put each source rectangle at its destination rectangle.
    Unscramble { tiles: Vec<Tile>, width: u32, height: u32 },
    /// The picture was cut into `cols` x `rows` equal tiles and shuffled: the tile at position `i` (row by row) belongs
    /// at position `order[i]`. The size of a tile comes from the downloaded picture.
    Shuffle { cols: u32, rows: u32, order: Vec<u32> },
    /// Draw text on a plain background instead of downloading anything (author's notes and the like).
    Text { title: String, body: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tile {
    pub sx: u32,
    pub sy: u32,
    pub dx: u32,
    pub dy: u32,
    pub w: u32,
    pub h: u32,
}

// ------------------------------------------------------------------ settings

/// A setting shown on the source's preferences screen.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PrefDef {
    Switch { key: String, title: String, #[serde(default)] summary: String, default: bool },
    /// Pick one of `entries` (`label`, `value`); the stored value is the `value`.
    List { key: String, title: String, #[serde(default)] summary: String, entries: Vec<(String, String)>, default: String },
    /// Pick any number of `entries`; the stored value is the list of values.
    MultiList { key: String, title: String, #[serde(default)] summary: String, entries: Vec<(String, String)>, default: Vec<String> },
    Text { key: String, title: String, #[serde(default)] summary: String, default: String },
}

impl PrefDef {
    pub fn switch(key: &str, title: &str, summary: &str, default: bool) -> Self {
        Self::Switch { key: key.into(), title: title.into(), summary: summary.into(), default }
    }

    pub fn list(key: &str, title: &str, entries: &[(&str, &str)], default: &str) -> Self {
        Self::List {
            key: key.into(),
            title: title.into(),
            summary: String::new(),
            entries: entries.iter().map(|(l, v)| (l.to_string(), v.to_string())).collect(),
            default: default.into(),
        }
    }

    pub fn multi_list(key: &str, title: &str, entries: &[(&str, &str)], default: &[&str]) -> Self {
        Self::MultiList {
            key: key.into(),
            title: title.into(),
            summary: String::new(),
            entries: entries.iter().map(|(l, v)| (l.to_string(), v.to_string())).collect(),
            default: default.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn text(key: &str, title: &str, summary: &str, default: &str) -> Self {
        Self::Text { key: key.into(), title: title.into(), summary: summary.into(), default: default.into() }
    }
}
