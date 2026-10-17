//! Sources written in Rust (WebAssembly): install, trust, load and call. See `docs/redesign/RUST-SOURCES.md`.
//!
//! Installed sources are signed `.sksource` packages (see `sk_source::package`). A package runs only if its
//! signature is valid **and** its publisher's fingerprint is in the trusted list the app passes in. Raw `.wasm`
//! files are loaded only when the app says unsigned sources are allowed (debug builds).

use crate::SkError;
use sk_source::package::{self, fingerprint, Package};
use sk_source::repo;
use sk_source::{Http, HttpReply, HttpRequest, SharedSource, WasmSource};
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};

const EXTENSION: &str = "sksource";

#[derive(Debug, uniffi::Record)]
pub struct RustSourceInfo {
    pub id: i64,
    pub name: String,
    pub lang: String,
    pub base_url: String,
    pub supports_latest: bool,
    /// File inside the sources directory, so the app can uninstall it.
    pub file_name: String,
    pub version: u32,
    pub publisher: String,
    /// Publisher key fingerprint; empty for an unsigned `.wasm` loaded in a debug build.
    pub fingerprint: String,
    /// The module's whole `info` answer as JSON (settings, rate limit, extra headers, flags); the app reads what it
    /// needs from it so new fields never change this record.
    pub info_json: String,
}

/// What the user is asked to trust before an install.
#[derive(Debug, uniffi::Record)]
pub struct PackageInfo {
    pub name: String,
    pub lang: String,
    pub version: u32,
    pub publisher: String,
    pub fingerprint: String,
}

#[derive(Debug, uniffi::Record)]
pub struct Rejected {
    pub file_name: String,
    pub reason: String,
    /// Set when the package is fine but its publisher is not trusted (yet), so the UI can offer to trust it.
    pub fingerprint: String,
}

#[derive(Debug, uniffi::Record)]
pub struct LoadReport {
    pub sources: Vec<RustSourceInfo>,
    pub rejected: Vec<Rejected>,
}

/// A loaded source and the language it was installed for (sent to it with every request).
struct Loaded {
    source: SharedSource,
    lang: String,
}

fn registry() -> &'static Mutex<HashMap<i64, Loaded>> {
    static REGISTRY: OnceLock<Mutex<HashMap<i64, Loaded>>> = OnceLock::new();
    REGISTRY.get_or_init(Default::default)
}

/// The app's HTTP client, as Kotlin implements it: Cloudflare handling, cookies, rate limits and the user agent
/// the user chose are all in there, so sources get them for free.
#[uniffi::export(with_foreign)]
pub trait HttpBridge: Send + Sync {
    /// `request_json` is a `{method, url, headers, body, content_type, follow_redirects}` object; the answer is
    /// `{status, headers, body, url}` or `{error}`. `source_id` picks the source's own headers and rate limit.
    fn request(&self, source_id: i64, request_json: String) -> String;

    /// The value of the cookie `name` the app holds for `url`, or an empty string.
    fn cookie(&self, source_id: i64, url: String, name: String) -> String;
}

fn bridge() -> &'static RwLock<Option<Arc<dyn HttpBridge>>> {
    static BRIDGE: OnceLock<RwLock<Option<Arc<dyn HttpBridge>>>> = OnceLock::new();
    BRIDGE.get_or_init(Default::default)
}

/// Installs the network the sources use. Call once at start-up; without it sources fall back to a plain client.
#[uniffi::export]
pub fn set_http_bridge(b: Arc<dyn HttpBridge>) {
    *bridge().write().unwrap() = Some(b);
}

/// True when a source may request `url` (not a local or private address). The app asks this for every redirect hop.
#[uniffi::export]
pub fn is_url_allowed(url: String) -> bool {
    !sk_source::is_blocked_url(&url)
}

/// What a source's requests go through: the app's client when it is installed, `sk-net` otherwise (GET only).
struct NetHttp {
    /// Known after the source answered `info`; requests made before that (none today) use 0.
    id: Arc<AtomicI64>,
    fallback: sk_net::Client,
}

impl Http for NetHttp {
    fn cookie(&self, url: &str, name: &str) -> Option<String> {
        let b = bridge().read().unwrap().clone()?;
        Some(b.cookie(self.id.load(Ordering::Relaxed), url.to_string(), name.to_string())).filter(|v| !v.is_empty())
    }

    fn request(&self, req: &HttpRequest) -> HttpReply {
        if let Some(b) = bridge().read().unwrap().clone() {
            let json = serde_json::to_string(req).unwrap_or_default();
            return serde_json::from_str(&b.request(self.id.load(Ordering::Relaxed), json))
                .unwrap_or_else(|e| HttpReply::failed(format!("bad answer from the app's network: {e}")));
        }
        if !req.method.is_empty() && !req.method.eq_ignore_ascii_case("GET") {
            return HttpReply::failed("only GET works without the app's network");
        }
        let pairs: Vec<(&str, &str)> = req.headers.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        match self.fallback.get_bytes_with(&req.url, &pairs) {
            Ok(bytes) => HttpReply { status: 200, body: String::from_utf8_lossy(&bytes).into_owned(), url: req.url.clone(), ..Default::default() },
            Err(sk_net::Error::Status(s)) => HttpReply { status: s, url: req.url.clone(), ..Default::default() },
            Err(e) => HttpReply::failed(e.to_string()),
        }
    }
}

/// Id of modules from before `version_id` existed: FNV-1a over `name/lang`, cleared to 63 bits.
fn legacy_id(name: &str, lang: &str) -> i64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in format!("{name}/{lang}").bytes() {
        h = (h ^ b as u64).wrapping_mul(0x100000001b3);
    }
    (h & (i64::MAX as u64)) as i64
}

fn pkg_err(e: package::PackageError) -> SkError {
    SkError::Package { reason: e.to_string() }
}

/// Checks a package's integrity and signature without installing or trusting anything.
#[uniffi::export]
pub fn inspect_source_package(bytes: Vec<u8>) -> Result<PackageInfo, SkError> {
    let Package { manifest: m, .. } = package::decode_and_verify(&bytes).map_err(pkg_err)?;
    Ok(PackageInfo { fingerprint: fingerprint(&m.public_key), name: m.name, lang: m.lang, version: m.version, publisher: m.publisher })
}

/// Verifies, checks trust, test-loads the module and writes it into `dir`. Replaces an older version from the
/// same publisher; refuses to replace a source of the same name that another publisher signed.
#[uniffi::export]
pub fn install_source_package(bytes: Vec<u8>, dir: String, trusted: Vec<String>) -> Result<RustSourceInfo, SkError> {
    let pkg = package::decode_and_verify(&bytes).map_err(pkg_err)?;
    let fp = fingerprint(&pkg.manifest.public_key);
    if !trusted.contains(&fp) {
        return Err(SkError::Package { reason: format!("the publisher {fp} is not trusted") });
    }
    let file_name = format!("{}.{EXTENSION}", slug(&pkg.manifest.name, &pkg.manifest.lang));
    let path = Path::new(&dir).join(&file_name);
    if let Ok(existing) = std::fs::read(&path) {
        match package::decode_and_verify(&existing) {
            Ok(old) if fingerprint(&old.manifest.public_key) != fp => {
                return Err(SkError::Package { reason: "a source with this name from another publisher is installed".into() });
            }
            Ok(old) if old.manifest.version > pkg.manifest.version => {
                return Err(SkError::Package { reason: "a newer version is already installed".into() });
            }
            _ => {}
        }
    }
    // Prove the module speaks the ABI before it replaces anything.
    let info = describe(&pkg, &file_name, &fp)?;
    std::fs::create_dir_all(&dir).map_err(|e| SkError::Package { reason: e.to_string() })?;
    std::fs::write(&path, &bytes).map_err(|e| SkError::Package { reason: e.to_string() })?;
    Ok(info)
}

#[derive(Debug, uniffi::Record)]
pub struct RepoEntryInfo {
    pub name: String,
    pub lang: String,
    pub version: u32,
    /// Absolute URL of the `.sksource` file.
    pub package_url: String,
    pub sha256: String,
    pub size: u64,
}

#[derive(Debug, uniffi::Record)]
pub struct RepoInfo {
    pub name: String,
    pub public_key: String,
    pub fingerprint: String,
    pub serial: u64,
    pub sources: Vec<RepoEntryInfo>,
}

fn download(url: &str) -> Result<Vec<u8>, SkError> {
    sk_net::Client::new().get_bytes(url).map_err(|e| SkError::Net { reason: format!("{url}: {e}") })
}

/// Downloads and verifies a repository index.
///
/// `expected_fingerprint` (empty the first time, when the user is still deciding) must match the index's key,
/// and `min_serial` is the highest serial seen before: an older index is refused, so a stale list cannot be replayed.
#[uniffi::export]
pub fn fetch_repo_index(url: String, expected_fingerprint: String, min_serial: u64) -> Result<RepoInfo, SkError> {
    let bytes = download(&url)?;
    let index = repo::decode_and_verify(&bytes).map_err(|e| SkError::Package { reason: e.to_string() })?;
    let fp = fingerprint(&index.public_key);
    if !expected_fingerprint.is_empty() && fp != expected_fingerprint {
        return Err(SkError::Package { reason: format!("the repository key changed (now {fp})") });
    }
    if index.serial < min_serial {
        return Err(SkError::Package {
            reason: format!("the repository served an older index (serial {} < {min_serial})", index.serial),
        });
    }
    Ok(RepoInfo {
        sources: index
            .sources
            .iter()
            .map(|e| RepoEntryInfo {
                name: e.name.clone(),
                lang: e.lang.clone(),
                version: e.version,
                package_url: repo::resolve_url(&url, &e.package),
                sha256: e.sha256.clone(),
                size: e.size,
            })
            .collect(),
        name: index.name,
        public_key: index.public_key,
        fingerprint: fp,
        serial: index.serial,
    })
}

/// Downloads a package listed in a repository, checks it against the index's hash, then installs it like
/// [`install_source_package`] (signature and trusted publisher still required).
#[uniffi::export]
pub fn install_from_repo(package_url: String, sha256: String, dir: String, trusted: Vec<String>) -> Result<RustSourceInfo, SkError> {
    let bytes = download(&package_url)?;
    if package::sha256_hex(&bytes) != sha256.to_ascii_lowercase() {
        return Err(SkError::Package { reason: "the downloaded package does not match the repository index".into() });
    }
    install_source_package(bytes, dir, trusted)
}

/// Removes an installed package or raw module by file name (no path separators allowed).
#[uniffi::export]
pub fn uninstall_source(dir: String, file_name: String) -> Result<(), SkError> {
    if file_name.contains(['/', '\\']) || file_name.starts_with('.') {
        return Err(SkError::Package { reason: "invalid file name".into() });
    }
    std::fs::remove_file(Path::new(&dir).join(file_name)).map_err(|e| SkError::Package { reason: e.to_string() })
}

fn slug(name: &str, lang: &str) -> String {
    let clean: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    format!("{}-{}", clean.trim_matches('-'), lang.to_lowercase().replace(|c: char| !c.is_ascii_alphanumeric(), ""))
}

/// Instantiates the module, asks it for its info and registers it.
fn describe(pkg: &Package, file_name: &str, fp: &str) -> Result<RustSourceInfo, SkError> {
    let (info, source) = instantiate(&pkg.wasm, file_name, fp, pkg.manifest.version, &pkg.manifest.publisher, &pkg.manifest.lang)?;
    registry().lock().unwrap().insert(info.id, source);
    Ok(info)
}

fn instantiate(
    wasm: &[u8],
    file_name: &str,
    fp: &str,
    version: u32,
    publisher: &str,
    install_lang: &str,
) -> Result<(RustSourceInfo, Loaded), SkError> {
    let id_cell = Arc::new(AtomicI64::new(0));
    let http: Arc<dyn Http> = Arc::new(NetHttp { id: id_cell.clone(), fallback: sk_net::Client::new() });
    let bad = |reason: String| SkError::Package { reason };
    let mut source = WasmSource::load(wasm, http).map_err(bad)?;
    let ask = serde_json::json!({ "op": "info", "ctx": { "lang": install_lang } }).to_string();
    let reply = source.call(&ask).map_err(bad)?;
    let v: serde_json::Value = serde_json::from_str(&reply).map_err(|e| bad(e.to_string()))?;
    let i = &v["info"];
    let Some(name) = i["name"].as_str() else {
        return Err(bad("the source did not report a name".into()));
    };
    // An unsigned module has no manifest; it speaks for itself.
    let lang = if install_lang.is_empty() { i["lang"].as_str().unwrap_or_default() } else { install_lang };
    if lang.is_empty() {
        return Err(bad("the source did not report a language".into()));
    }
    let id = match i["id"].as_i64() {
        Some(id) if id > 0 => id,
        // Newer modules report `version_id`; derive the id the way extensions did.
        _ => match i["version_id"].as_u64() {
            Some(v) => sk_source::extension_id(name, lang, v as u32),
            None => legacy_id(name, lang),
        },
    };
    id_cell.store(id, Ordering::Relaxed);
    let info = RustSourceInfo {
        id,
        name: name.to_string(),
        lang: lang.to_string(),
        base_url: i["base_url"].as_str().unwrap_or_default().to_string(),
        supports_latest: i["supports_latest"].as_bool().unwrap_or(false),
        file_name: file_name.to_string(),
        version,
        publisher: publisher.to_string(),
        fingerprint: fp.to_string(),
        info_json: i.to_string(),
    };
    Ok((info, Loaded { source: Arc::new(Mutex::new(source)), lang: lang.to_string() }))
}

/// Loads the installed sources from `dir`, replacing what was loaded before.
///
/// `.sksource` packages need a valid signature and a publisher in `trusted`; the rest are reported in
/// `rejected`. Raw `.wasm` modules load only when `allow_unsigned` is true.
#[uniffi::export]
pub fn load_rust_sources(dir: String, trusted: Vec<String>, allow_unsigned: bool) -> LoadReport {
    let mut loaded = HashMap::new();
    let mut report = LoadReport { sources: Vec::new(), rejected: Vec::new() };
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .map(|d| d.filter_map(|e| e.ok()).map(|e| e.path()).collect())
        .unwrap_or_default();
    files.sort();
    for path in files {
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default().to_string();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or_default();
        let Ok(bytes) = std::fs::read(&path) else { continue };
        let result = match ext {
            EXTENSION => match package::decode_and_verify(&bytes) {
                Err(e) => Err((e.to_string(), String::new())),
                Ok(pkg) => {
                    let fp = fingerprint(&pkg.manifest.public_key);
                    if trusted.contains(&fp) {
                        instantiate(&pkg.wasm, &file_name, &fp, pkg.manifest.version, &pkg.manifest.publisher, &pkg.manifest.lang)
                            .map_err(|e| (e.to_string(), String::new()))
                    } else {
                        Err(("the publisher is not trusted".to_string(), fp))
                    }
                }
            },
            "wasm" if allow_unsigned => {
                instantiate(&bytes, &file_name, "", 0, "unsigned", "").map_err(|e| (e.to_string(), String::new()))
            }
            _ => continue,
        };
        match result {
            Ok((info, source)) => {
                loaded.insert(info.id, source);
                report.sources.push(info);
            }
            Err((reason, fingerprint)) => report.rejected.push(Rejected { file_name, reason, fingerprint }),
        }
    }
    *registry().lock().unwrap() = loaded;
    report
}

/// Sends one JSON request (see `docs/redesign/RUST-SOURCES.md`) to a loaded source and returns its JSON reply.
/// Blocking, and serialized per source; call from a background dispatcher.
#[uniffi::export]
pub fn call_rust_source(id: i64, request: String) -> Result<String, SkError> {
    let (source, lang) = registry()
        .lock()
        .unwrap()
        .get(&id)
        .map(|l| (l.source.clone(), l.lang.clone()))
        .ok_or_else(|| SkError::Net { reason: format!("no Rust source with id {id}") })?;
    let request = with_language(&request, &lang);
    let mut guard = source.lock().unwrap();
    guard.call(&request).map_err(|reason| SkError::Net { reason })
}

/// Adds the installed language to the request's `ctx` (the app only knows the user's settings).
fn with_language(request: &str, lang: &str) -> String {
    let Ok(mut v) = serde_json::from_str::<serde_json::Value>(request) else { return request.to_string() };
    if let Some(obj) = v.as_object_mut() {
        let ctx = obj.entry("ctx").or_insert_with(|| serde_json::json!({}));
        if let Some(c) = ctx.as_object_mut() {
            c.insert("lang".into(), lang.into());
        }
    }
    v.to_string()
}

#[cfg(test)]
mod tests {
    use super::{slug, with_language};

    #[test]
    fn the_installed_language_rides_in_ctx() {
        let r = with_language(r#"{"op":"popular","page":1,"ctx":{"prefs":{"a":1}}}"#, "pt-BR");
        assert!(r.contains(r#""lang":"pt-BR""#) && r.contains(r#""prefs""#), "{r}");
        assert!(with_language(r#"{"op":"info"}"#, "en").contains(r#""ctx":{"lang":"en"}"#));
        assert_eq!(with_language("not json", "en"), "not json");
    }

    #[test]
    fn file_names_are_safe_and_stable() {
        assert_eq!(slug("MangaDex (Rust)", "en"), "mangadex--rust-en");
        assert!(!slug("../../etc/passwd", "en").contains(['/', '.']));
    }
}

