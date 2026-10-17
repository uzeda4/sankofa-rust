//! SDK for writing a Sankofa source in Rust. Implement [`Source`], then call [`export_source!`]:
//!
//! ```ignore
//! #[derive(Default)]
//! struct MySource;
//! impl sk_source_sdk::Source for MySource { /* info, popular, details, chapters, pages */ }
//! sk_source_sdk::export_source!(MySource);
//! ```
//!
//! Build with `cargo build --release --target wasm32-unknown-unknown`, then sign and package the `.wasm` with
//! `sk-pack`. The ABI (JSON over linear memory) is documented in `docs/redesign/RUST-SOURCES.md`.
//!
//! What a source can use: [`http`] for the network, [`html`] for scraping pages with jsoup-style selectors,
//! [`filters`] for search filters, [`Info::prefs`] for settings, and `lang()`/`pref_*` to read the language it was
//! installed for and the user's settings.

mod date;
mod filters;
mod http;
mod image;
mod types;

pub use date::{civil_from_days, days_from_civil, millis, minus, parse_date, parse_iso, Unit};
pub use filters::{FilterDef, Filters, Tri};
pub use image::image_size;
pub use http::{
    cookie, get, http_get, http_get_json, log, now_ms, post, send_all, HttpRequest, HttpResponse, Request, Response, Result,
};
#[cfg(not(target_arch = "wasm32"))]
pub use http::set_mock_http;
#[cfg(feature = "html")]
pub use sk_html as html;
pub use serde_json::{self, json, Value};
pub use types::*;
pub use url::{self, Url};

use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;

/// What a source implements. Everything except `info`, `popular`, `details`, `chapters` and `pages` is optional.
pub trait Source {
    fn info(&self) -> Info;

    fn popular(&self, page: u32) -> Result<MangaPage>;

    fn latest(&self, _page: u32) -> Result<MangaPage> {
        Err("latest updates are not supported".into())
    }

    fn search(&self, _page: u32, _query: &str, _filters: &Filters) -> Result<MangaPage> {
        Err("search is not supported".into())
    }

    /// The filters to draw. `data` is what [`Source::filter_data`] returned earlier (cached by the host), or
    /// `None` before the first fetch.
    fn filters(&self, _data: Option<&Value>) -> Vec<FilterDef> {
        Vec::new()
    }

    /// Data that filters need from the site (genre lists). Only called when `Info::supports_filter_fetching`.
    fn filter_data(&self) -> Result<Option<Value>> {
        Ok(None)
    }

    fn details(&self, manga: &Manga) -> Result<Manga>;

    fn chapters(&self, manga: &Manga) -> Result<Vec<Chapter>>;

    /// Details and chapters in one go. The default calls both in turn; override to fetch them in parallel or
    /// from one page.
    fn update(&self, manga: &Manga, fetch_details: bool, fetch_chapters: bool) -> Result<Update> {
        Ok(Update {
            manga: if fetch_details { Some(self.details(manga)?) } else { None },
            chapters: if fetch_chapters { Some(self.chapters(manga)?) } else { None },
        })
    }

    fn pages(&self, chapter: &Chapter) -> Result<Vec<Page>>;

    /// Opens a title from a link on the source's own site (`None` when the link is not one of its titles).
    fn by_url(&self, _url: &str) -> Result<Option<Manga>> {
        Ok(None)
    }

    /// The address that opens `manga` in a browser, when it is not simply the site plus the stored address. Only called
    /// when `Info::custom_urls`.
    fn manga_url(&self, _manga: &Manga) -> Option<String> {
        None
    }

    /// Same for a chapter.
    fn chapter_url(&self, _chapter: &Chapter) -> Option<String> {
        None
    }

    /// Titles related to `manga`, when the site has such a list. Only called when `Info::supports_related`.
    fn related(&self, _manga: &Manga) -> Result<Vec<Manga>> {
        Ok(Vec::new())
    }
}

/// The answer to `update`: whatever was fetched.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Update {
    pub manga: Option<Manga>,
    pub chapters: Option<Vec<Chapter>>,
}

// ------------------------------------------------------------------ context

/// What the host sends with every request: the language this copy was installed for and the user's settings.
#[derive(Debug, Clone, Default, Deserialize)]
struct Ctx {
    #[serde(default)]
    lang: String,
    #[serde(default)]
    prefs: HashMap<String, Value>,
    /// What `Source::filter_data` returned last time (cached by the host), for searches that need it.
    #[serde(default)]
    filter_data: Option<Value>,
}

thread_local! {
    static CTX: RefCell<Ctx> = RefCell::new(Ctx::default());
}

/// The language this copy of the source was installed for (a package can be installed once per language).
pub fn lang() -> String {
    CTX.with(|c| c.borrow().lang.clone())
}

/// The cached result of `Source::filter_data` (genre lists and the like), once the app has fetched it.
pub fn filter_data() -> Option<Value> {
    CTX.with(|c| c.borrow().filter_data.clone())
}

pub fn pref_bool(key: &str, default: bool) -> bool {
    CTX.with(|c| c.borrow().prefs.get(key).and_then(Value::as_bool).unwrap_or(default))
}

pub fn pref_str(key: &str, default: &str) -> String {
    CTX.with(|c| c.borrow().prefs.get(key).and_then(Value::as_str).unwrap_or(default).to_string())
}

/// The site address in use: the mirror the user picked in settings (see `Info::mirrors`), else `default`.
pub fn base_url(default: &str) -> String {
    pref_str("mirror", default).trim_end_matches('/').to_string()
}

pub fn pref_list(key: &str) -> Vec<String> {
    CTX.with(|c| {
        c.borrow()
            .prefs
            .get(key)
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
            .unwrap_or_default()
    })
}

/// Sets the context directly (tests on the PC; on the device the host sends it with each request).
pub fn set_context(lang: &str, prefs: &[(&str, Value)]) {
    CTX.with(|c| {
        let filter_data = c.borrow().filter_data.clone();
        *c.borrow_mut() = Ctx { lang: lang.into(), prefs: prefs.iter().map(|(k, v)| (k.to_string(), v.clone())).collect(), filter_data };
    });
}

/// Sets what `filter_data()` returns (tests on the PC).
pub fn set_filter_data(data: Option<Value>) {
    CTX.with(|c| c.borrow_mut().filter_data = data);
}

// ------------------------------------------------------------------ url helpers

/// Percent-encodes a query-string component.
pub fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// The part of `url` after the scheme and host (`https://a.org/x?y=1#z` -> `/x?y=1`), or `url` unchanged when it is
/// already a path. Titles and chapters are stored this way so they survive a domain change.
///
/// Percent escapes are decoded (`%C3%A9` becomes `é`), exactly as `setUrlWithoutDomain` of the Kotlin extensions did,
/// because a library built with an extension keeps the addresses it stored and the same title must match them.
pub fn path_of(url: &str) -> String {
    use percent_encoding::percent_decode_str;
    let decode = |s: &str| percent_decode_str(s).decode_utf8_lossy().into_owned();
    match Url::parse(url) {
        Ok(u) => {
            let mut p = decode(u.path());
            if let Some(q) = u.query() {
                p.push('?');
                p.push_str(&decode(q));
            }
            p
        }
        Err(_) => url.to_string(),
    }
}

/// What follows the first `delimiter` in `s` (`substringAfter`): the whole string when it is not there.
pub fn substring_after(s: &str, delimiter: &str) -> String {
    s.split_once(delimiter).map(|(_, rest)| rest.to_string()).unwrap_or_else(|| s.to_string())
}

/// Joins `base` and a stored path or URL into an absolute URL.
pub fn absolute(base: &str, path: &str) -> String {
    Url::parse(base).and_then(|b| b.join(path)).map(|u| u.to_string()).unwrap_or_else(|_| format!("{base}{path}"))
}

// ------------------------------------------------------------------ dispatch

#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum Call {
    Info,
    Filters { #[serde(default)] data: Option<Value> },
    FilterData,
    Popular { page: u32 },
    Latest { page: u32 },
    Search { page: u32, #[serde(default)] query: String, #[serde(default)] filters: Filters },
    Update { manga: Manga, #[serde(default)] fetch_details: bool, #[serde(default)] fetch_chapters: bool },
    Details { manga: Manga },
    Chapters { manga: Manga },
    Pages { chapter: Chapter },
    ByUrl { url: String },
    Related { manga: Manga },
    MangaUrl { manga: Manga },
    ChapterUrl { chapter: Chapter },
}

/// Runs one JSON request against `source` and returns the JSON response. Used by [`export_source!`]; also handy for
/// testing a source on the PC with [`set_mock_http`].
pub fn dispatch<S: Source>(source: &S, request: &str) -> String {
    fn wrap<T: Serialize>(key: &str, r: Result<T>) -> Value {
        match r {
            Ok(v) => json!({ key: v }),
            Err(e) => json!({ "error": e }),
        }
    }
    let parsed: Value = match serde_json::from_str(request) {
        Ok(v) => v,
        Err(e) => return json!({ "error": format!("bad request: {e}") }).to_string(),
    };
    if let Some(ctx) = parsed.get("ctx") {
        if let Ok(c) = serde_json::from_value::<Ctx>(ctx.clone()) {
            CTX.with(|cell| *cell.borrow_mut() = c);
        }
    }
    let value = match serde_json::from_value::<Call>(parsed) {
        Err(e) => json!({ "error": format!("bad request: {e}") }),
        Ok(Call::Info) => json!({ "info": source.info(), "abi": 2 }),
        Ok(Call::Filters { data }) => json!({ "filters": source.filters(data.as_ref()) }),
        Ok(Call::FilterData) => wrap("data", source.filter_data()),
        Ok(Call::Popular { page }) => wrap("page", source.popular(page)),
        Ok(Call::Latest { page }) => wrap("page", source.latest(page)),
        Ok(Call::Search { page, query, filters }) => wrap("page", source.search(page, &query, &filters)),
        Ok(Call::Update { manga, fetch_details, fetch_chapters }) => wrap("update", source.update(&manga, fetch_details, fetch_chapters)),
        Ok(Call::Details { manga }) => wrap("manga", source.details(&manga)),
        Ok(Call::Chapters { manga }) => wrap("chapters", source.chapters(&manga)),
        Ok(Call::Pages { chapter }) => wrap("pages", source.pages(&chapter)),
        Ok(Call::ByUrl { url }) => wrap("manga", source.by_url(&url)),
        Ok(Call::Related { manga }) => wrap("mangas", source.related(&manga)),
        Ok(Call::MangaUrl { manga }) => json!({ "url": source.manga_url(&manga) }),
        Ok(Call::ChapterUrl { chapter }) => json!({ "url": source.chapter_url(&chapter) }),
    };
    value.to_string()
}

/// Makes a panic show up in the host log before the module traps.
pub fn init_panic_hook() {
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        std::panic::set_hook(Box::new(|info| log(&format!("panic: {info}"))));
    });
}

/// Guest allocator and entry point; see the crate docs. `$ty` must implement [`Source`] and `Default`.
#[macro_export]
macro_rules! export_source {
    ($ty:ty) => {
        #[no_mangle]
        pub extern "C" fn sk_alloc(len: i32) -> i32 {
            let mut v = Vec::<u8>::with_capacity(len.max(1) as usize);
            let ptr = v.as_mut_ptr();
            std::mem::forget(v);
            ptr as i32
        }

        #[no_mangle]
        pub unsafe extern "C" fn sk_free(ptr: i32, len: i32) {
            drop(Vec::from_raw_parts(ptr as *mut u8, 0, len.max(1) as usize));
        }

        #[no_mangle]
        pub unsafe extern "C" fn sk_call(ptr: i32, len: i32) -> i64 {
            $crate::init_panic_hook();
            let request = std::slice::from_raw_parts(ptr as *const u8, len as usize);
            let response = match std::str::from_utf8(request) {
                Ok(r) => $crate::dispatch(&<$ty as Default>::default(), r),
                Err(_) => r#"{"error":"request is not UTF-8"}"#.to_string(),
            };
            let mut bytes = response.into_bytes();
            bytes.shrink_to_fit();
            let (out_ptr, out_len) = (bytes.as_mut_ptr() as i64, bytes.len() as i64);
            std::mem::forget(bytes);
            (out_ptr << 32) | out_len
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Fake;
    impl Source for Fake {
        fn info(&self) -> Info {
            Info { name: "Fake".into(), lang: lang(), base_url: "https://x".into(), ..Default::default() }
        }
        fn popular(&self, page: u32) -> Result<MangaPage> {
            Ok(MangaPage { mangas: vec![Manga { url: format!("/m/{page}"), title: "T".into(), ..Default::default() }], has_next: true })
        }
        fn search(&self, _page: u32, query: &str, filters: &Filters) -> Result<MangaPage> {
            let title = format!("{query}|{}|{}", filters.text("author"), pref_bool("nsfw", false));
            Ok(MangaPage { mangas: vec![Manga { title, ..Default::default() }], has_next: false })
        }
        fn details(&self, manga: &Manga) -> Result<Manga> {
            Err(format!("no {}", manga.url))
        }
        fn chapters(&self, _: &Manga) -> Result<Vec<Chapter>> {
            Ok(vec![Chapter::default()])
        }
        fn pages(&self, _: &Chapter) -> Result<Vec<Page>> {
            Ok(vec![Page::new(0, "https://x/1.jpg")])
        }
    }

    #[test]
    fn requests_are_routed_and_errors_are_reported() {
        let s = Fake;
        assert!(dispatch(&s, r#"{"op":"info","ctx":{"lang":"pt-BR"}}"#).contains(r#""lang":"pt-BR""#));
        assert!(dispatch(&s, r#"{"op":"popular","page":3}"#).contains(r#""url":"/m/3""#));
        assert_eq!(dispatch(&s, r#"{"op":"details","manga":{"url":"/m/1","title":"t"}}"#), r#"{"error":"no /m/1"}"#);
        assert!(dispatch(&s, r#"{"op":"latest","page":1}"#).contains("not supported"));
        assert!(dispatch(&s, "garbage").contains("bad request"));
        assert!(dispatch(&s, r#"{"op":"nope"}"#).contains("bad request"));
    }

    #[test]
    fn filters_and_settings_reach_the_source() {
        let r = dispatch(
            &Fake,
            r#"{"op":"search","page":1,"query":"q","filters":{"author":" Bob "},"ctx":{"lang":"en","prefs":{"nsfw":true}}}"#,
        );
        assert!(r.contains(r#""title":"q|Bob|true""#), "{r}");
        // the context is replaced by each request
        let r = dispatch(&Fake, r#"{"op":"search","page":1,"query":"q","ctx":{"lang":"en"}}"#);
        assert!(r.contains(r#""title":"q||false""#), "{r}");
    }

    #[test]
    fn update_defaults_to_details_and_chapters() {
        let r = dispatch(&Fake, r#"{"op":"update","manga":{"url":"/m","title":"t"},"fetch_details":false,"fetch_chapters":true}"#);
        assert!(r.contains(r#""update":{"chapters":[{"#) && r.contains(r#""manga":null"#), "{r}");
        let r = dispatch(&Fake, r#"{"op":"update","manga":{"url":"/m","title":"t"},"fetch_details":true,"fetch_chapters":false}"#);
        assert!(r.contains("no /m"), "{r}");
    }

    #[test]
    fn requests_use_the_mock_network_and_forms_are_encoded() {
        set_mock_http(|r| {
            let seen = format!("{} {} {:?} {:?}", r.method, r.url, r.content_type, r.body);
            Ok(HttpResponse { status: 200, body: seen, url: format!("{}#final", r.url), ..Default::default() })
        });
        let r = post("https://x.org/ajax").form(&[("a b", "c&d")]).send().unwrap();
        assert!(r.body.contains(r#"POST https://x.org/ajax Some("application/x-www-form-urlencoded") Some("a%20b=c%26d")"#), "{}", r.body);
        assert_eq!(r.url, "https://x.org/ajax#final");
        set_mock_http(|_| Ok(HttpResponse { status: 503, ..Default::default() }));
        assert!(get("https://x.org/").send().unwrap_err().contains("503"));
        assert_eq!(get("https://x.org/").allow_errors().send().unwrap().status, 503);
        let many = send_all(vec![get("https://a/"), get("https://b/").allow_errors()]);
        assert!(many[0].is_err() && many[1].is_ok());
    }

    #[test]
    fn urls_and_paths() {
        assert_eq!(url_encode("a b&ç"), "a%20b%26%C3%A7");
        assert_eq!(path_of("https://x.org/manga/a/?p=1#f"), "/manga/a/?p=1");
        assert_eq!(path_of("/already"), "/already");
        assert_eq!(path_of("https://x.org/manga/%E3%81%82/?q=%C3%A9"), "/manga/あ/?q=é", "decoded, like URI.getPath");
        assert_eq!(substring_after("https://x.org/a/b", "https://x.org"), "/a/b");
        assert_eq!(substring_after("https://y.org/a", "https://x.org"), "https://y.org/a");
        assert_eq!(absolute("https://x.org", "/manga/a"), "https://x.org/manga/a");
        assert_eq!(absolute("https://x.org/", "https://y.org/z"), "https://y.org/z");
    }
}
