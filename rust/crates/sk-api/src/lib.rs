//! The surface the Kotlin app sees: one small, blocking API over the `sk-*` crates, exported with UniFFI.
//! Call everything from a background dispatcher; nothing here touches the UI.

uniffi::setup_scaffolding!();

#[derive(Debug, uniffi::Error)]
pub enum SkError {
    Db { reason: String },
    Net { reason: String },
}

impl std::fmt::Display for SkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Db { reason } | Self::Net { reason } => f.write_str(reason),
        }
    }
}

#[derive(Debug, uniffi::Record)]
pub struct LibraryEntry {
    pub id: i64,
    pub title: String,
    /// Same numeric values as `SManga` (0 unknown, 1 ongoing, 2 completed, ...).
    pub status: u8,
    pub chapters_total: i64,
    pub chapters_read: i64,
}

#[derive(Debug, uniffi::Record)]
pub struct CoreInfo {
    pub version: String,
    /// `PRAGMA user_version` of the database, i.e. how many migrations the Kotlin side applied.
    pub schema_version: i64,
    pub favorites: u32,
}

/// Version of the Rust core, so the app can show which native library it loaded.
#[uniffi::export]
pub fn core_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// The library (favorites) read straight from the app's database file, by title.
#[uniffi::export]
pub fn library(db_path: String) -> Result<Vec<LibraryEntry>, SkError> {
    let db = sk_db::Db::open(&db_path).map_err(db_err)?;
    db.library()
        .map_err(db_err)?
        .into_iter()
        .map(|m| {
            let (chapters_total, chapters_read) = db.chapter_counts(m.id).map_err(db_err)?;
            Ok(LibraryEntry { id: m.id, title: m.title, status: m.status as u8, chapters_total, chapters_read })
        })
        .collect()
}

#[uniffi::export]
pub fn core_info(db_path: String) -> Result<CoreInfo, SkError> {
    let db = sk_db::Db::open(&db_path).map_err(db_err)?;
    Ok(CoreInfo {
        version: core_version(),
        schema_version: db.schema_version().map_err(db_err)?,
        favorites: db.library().map_err(db_err)?.len() as u32,
    })
}

/// GET a page as text. `headers` are `(name, value)` pairs, e.g. a `Referer`.
#[uniffi::export]
pub fn fetch_text(url: String, headers: Vec<Header>) -> Result<String, SkError> {
    let pairs: Vec<(&str, &str)> = headers.iter().map(|h| (h.name.as_str(), h.value.as_str())).collect();
    let bytes = sk_net::Client::new().get_bytes_with(&url, &pairs).map_err(|e| SkError::Net { reason: e.to_string() })?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

#[derive(Debug, uniffi::Record)]
pub struct Header {
    pub name: String,
    pub value: String,
}

fn db_err(e: sk_db::Error) -> SkError {
    SkError::Db { reason: e.to_string() }
}


// ---- Sources written in Rust (WebAssembly) ----

use sk_source::{Http, SharedSource, WasmSource};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Debug, uniffi::Record)]
pub struct RustSourceInfo {
    pub id: i64,
    pub name: String,
    pub lang: String,
    pub base_url: String,
    pub supports_latest: bool,
}

fn registry() -> &'static Mutex<HashMap<i64, SharedSource>> {
    static REGISTRY: OnceLock<Mutex<HashMap<i64, SharedSource>>> = OnceLock::new();
    REGISTRY.get_or_init(Default::default)
}

/// Sources get their network from `sk-net`; non-2xx answers are returned as a status, not an error.
struct NetHttp(sk_net::Client);

impl Http for NetHttp {
    fn get(&self, url: &str, headers: &[(String, String)]) -> Result<(u16, String), String> {
        let pairs: Vec<(&str, &str)> = headers.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        match self.0.get_bytes_with(url, &pairs) {
            Ok(bytes) => Ok((200, String::from_utf8_lossy(&bytes).into_owned())),
            Err(sk_net::Error::Status(s)) => Ok((s, String::new())),
            Err(e) => Err(e.to_string()),
        }
    }
}

/// FNV-1a over `name/lang`, cleared to 63 bits so it is a valid positive `Long` on the Kotlin side.
fn derive_id(name: &str, lang: &str) -> i64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in format!("{name}/{lang}").bytes() {
        h = (h ^ b as u64).wrapping_mul(0x100000001b3);
    }
    (h & (i64::MAX as u64)) as i64
}

/// Loads every `*.wasm` source in `dir`, replacing what was loaded before. A module that fails to load is
/// skipped (and logged by the caller through the returned list not containing it).
#[uniffi::export]
pub fn load_rust_sources(dir: String) -> Vec<RustSourceInfo> {
    let http: Arc<dyn Http> = Arc::new(NetHttp(sk_net::Client::new()));
    let mut loaded = HashMap::new();
    let mut infos = Vec::new();
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .map(|d| d.filter_map(|e| e.ok()).map(|e| e.path()).collect())
        .unwrap_or_default();
    files.sort();
    for path in files.into_iter().filter(|p| p.extension().is_some_and(|e| e == "wasm")) {
        let Ok(bytes) = std::fs::read(&path) else { continue };
        let Ok(mut source) = WasmSource::load(&bytes, http.clone()) else { continue };
        let Ok(reply) = source.call(r#"{"op":"info"}"#) else { continue };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&reply) else { continue };
        let i = &v["info"];
        let (Some(name), Some(lang)) = (i["name"].as_str(), i["lang"].as_str()) else { continue };
        let id = match i["id"].as_i64() {
            Some(id) if id > 0 => id,
            _ => derive_id(name, lang),
        };
        infos.push(RustSourceInfo {
            id,
            name: name.to_string(),
            lang: lang.to_string(),
            base_url: i["base_url"].as_str().unwrap_or_default().to_string(),
            supports_latest: i["supports_latest"].as_bool().unwrap_or(false),
        });
        loaded.insert(id, Arc::new(Mutex::new(source)));
    }
    *registry().lock().unwrap() = loaded;
    infos
}

/// Sends one JSON request (see `docs/redesign/RUST-SOURCES.md`) to a loaded source and returns its JSON reply.
/// Blocking, and serialized per source; call from a background dispatcher.
#[uniffi::export]
pub fn call_rust_source(id: i64, request: String) -> Result<String, SkError> {
    let source = registry()
        .lock()
        .unwrap()
        .get(&id)
        .cloned()
        .ok_or_else(|| SkError::Net { reason: format!("no Rust source with id {id}") })?;
    let mut guard = source.lock().unwrap();
    guard.call(&request).map_err(|reason| SkError::Net { reason })
}
