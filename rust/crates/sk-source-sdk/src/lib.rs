//! SDK for writing a Sankofa source in Rust. Implement [`Source`], then call [`export_source!`]:
//!
//! ```ignore
//! #[derive(Default)]
//! struct MySource;
//! impl sk_source_sdk::Source for MySource { /* info, popular, details, chapters, pages */ }
//! sk_source_sdk::export_source!(MySource);
//! ```
//!
//! Build with `cargo build --release --target wasm32-unknown-unknown` and drop the `.wasm` into the app's
//! `rust-sources` folder. The ABI is documented in `docs/redesign/RUST-SOURCES.md`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Info {
    pub name: String,
    /// BCP-47-ish language code, or `all`.
    pub lang: String,
    pub base_url: String,
    #[serde(default)]
    pub supports_latest: bool,
    /// Stable numeric id. Leave 0 to let the host derive one from name and language.
    #[serde(default)]
    pub id: i64,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MangaPage {
    pub mangas: Vec<Manga>,
    pub has_next: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Chapter {
    pub url: String,
    pub name: String,
    #[serde(default)]
    pub chapter_number: f32,
    #[serde(default)]
    pub scanlator: Option<String>,
    /// Milliseconds since the Unix epoch; 0 when unknown.
    #[serde(default)]
    pub date_upload: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Page {
    pub index: u32,
    /// Page URL when the image URL must be resolved later; usually empty.
    #[serde(default)]
    pub url: String,
    pub image_url: String,
}

pub type Result<T> = std::result::Result<T, String>;

/// What a source implements. Everything except `info`, `popular`, `details`, `chapters` and `pages` is optional.
pub trait Source {
    fn info(&self) -> Info;
    fn popular(&self, page: u32) -> Result<MangaPage>;
    fn latest(&self, _page: u32) -> Result<MangaPage> {
        Err("latest updates are not supported".into())
    }
    fn search(&self, _page: u32, _query: &str) -> Result<MangaPage> {
        Err("search is not supported".into())
    }
    fn details(&self, url: &str) -> Result<Manga>;
    fn chapters(&self, url: &str) -> Result<Vec<Chapter>>;
    fn pages(&self, chapter_url: &str) -> Result<Vec<Page>>;
}

#[derive(Debug, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub error: Option<String>,
}

#[cfg(target_arch = "wasm32")]
mod host {
    #[link(wasm_import_module = "sk")]
    extern "C" {
        pub fn http_get(url_ptr: *const u8, url_len: i32, hdr_ptr: *const u8, hdr_len: i32) -> i64;
        pub fn log(ptr: *const u8, len: i32);
    }
}

/// GET through the host (which owns the network, TLS, cookies and rate limits).
pub fn http_get(url: &str, headers: &[(&str, &str)]) -> Result<HttpResponse> {
    #[cfg(target_arch = "wasm32")]
    {
        let hdr = serde_json::to_string(headers).map_err(|e| e.to_string())?;
        let packed = unsafe { host::http_get(url.as_ptr(), url.len() as i32, hdr.as_ptr(), hdr.len() as i32) };
        let bytes = unsafe { take_guest_bytes(packed) }.ok_or("host returned no response")?;
        let resp: HttpResponse = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        match resp.error {
            Some(e) => Err(e),
            None if !(200..300).contains(&resp.status) => Err(format!("HTTP {}", resp.status)),
            None => Ok(resp),
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (url, headers);
        Err("http_get is only available inside the Sankofa runtime".into())
    }
}

/// Convenience: GET and parse the body as JSON.
pub fn http_get_json<T: for<'de> Deserialize<'de>>(url: &str, headers: &[(&str, &str)]) -> Result<T> {
    serde_json::from_str(&http_get(url, headers)?.body).map_err(|e| format!("bad JSON from {url}: {e}"))
}

/// Writes a line to the host log.
pub fn log(message: &str) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        host::log(message.as_ptr(), message.len() as i32);
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = message;
}

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

#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum Request {
    Info,
    Popular { page: u32 },
    Latest { page: u32 },
    Search { page: u32, query: String },
    Details { url: String },
    Chapters { url: String },
    Pages { url: String },
}

/// Runs one JSON request against `source` and returns the JSON response. Used by [`export_source!`]; also
/// handy for testing a source on the host with a fake `http_get`.
pub fn dispatch<S: Source>(source: &S, request: &str) -> String {
    fn wrap<T: Serialize>(key: &str, r: Result<T>) -> serde_json::Value {
        match r {
            Ok(v) => serde_json::json!({ key: v }),
            Err(e) => serde_json::json!({ "error": e }),
        }
    }
    let value = match serde_json::from_str::<Request>(request) {
        Err(e) => serde_json::json!({ "error": format!("bad request: {e}") }),
        Ok(Request::Info) => serde_json::json!({ "info": source.info() }),
        Ok(Request::Popular { page }) => wrap("page", source.popular(page)),
        Ok(Request::Latest { page }) => wrap("page", source.latest(page)),
        Ok(Request::Search { page, query }) => wrap("page", source.search(page, &query)),
        Ok(Request::Details { url }) => wrap("manga", source.details(&url)),
        Ok(Request::Chapters { url }) => wrap("chapters", source.chapters(&url)),
        Ok(Request::Pages { url }) => wrap("pages", source.pages(&url)),
    };
    value.to_string()
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

/// Reads and frees a `(ptr << 32) | len` buffer the host wrote into this module's memory.
#[cfg(target_arch = "wasm32")]
unsafe fn take_guest_bytes(packed: i64) -> Option<Vec<u8>> {
    let (ptr, len) = ((packed >> 32) as u32 as usize, packed as u32 as usize);
    if ptr == 0 {
        return None;
    }
    Some(Vec::from_raw_parts(ptr as *mut u8, len, len.max(1)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Fake;
    impl Source for Fake {
        fn info(&self) -> Info {
            Info { name: "Fake".into(), lang: "en".into(), base_url: "https://x".into(), ..Default::default() }
        }
        fn popular(&self, page: u32) -> Result<MangaPage> {
            Ok(MangaPage { mangas: vec![Manga { url: format!("/m/{page}"), title: "T".into(), ..Default::default() }], has_next: true })
        }
        fn details(&self, _: &str) -> Result<Manga> {
            Err("nope".into())
        }
        fn chapters(&self, _: &str) -> Result<Vec<Chapter>> {
            Ok(vec![])
        }
        fn pages(&self, _: &str) -> Result<Vec<Page>> {
            Ok(vec![])
        }
    }

    #[test]
    fn requests_are_routed_and_errors_are_reported() {
        let s = Fake;
        assert!(dispatch(&s, r#"{"op":"info"}"#).contains(r#""name":"Fake""#));
        assert!(dispatch(&s, r#"{"op":"popular","page":3}"#).contains(r#""url":"/m/3""#));
        assert_eq!(dispatch(&s, r#"{"op":"details","url":"/m/1"}"#), r#"{"error":"nope"}"#);
        assert!(dispatch(&s, r#"{"op":"search","page":1,"query":"a"}"#).contains("not supported"));
        assert!(dispatch(&s, "garbage").contains("bad request"));
    }

    #[test]
    fn query_strings_are_percent_encoded() {
        assert_eq!(url_encode("a b&ç"), "a%20b%26%C3%A7");
    }
}
