//! Sources written in Rust (WebAssembly): install, trust, load and call. See `docs/redesign/RUST-SOURCES.md`.
//!
//! Installed sources are signed `.sksource` packages (see `sk_source::package`). A package runs only if its
//! signature is valid **and** its publisher's fingerprint is in the trusted list the app passes in. Raw `.wasm`
//! files are loaded only when the app says unsigned sources are allowed (debug builds).

use crate::SkError;
use sk_source::package::{self, fingerprint, Package};
use sk_source::{Http, SharedSource, WasmSource};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

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
    let (info, source) = instantiate(&pkg.wasm, file_name, fp, pkg.manifest.version, &pkg.manifest.publisher)?;
    registry().lock().unwrap().insert(info.id, source);
    Ok(info)
}

fn instantiate(
    wasm: &[u8],
    file_name: &str,
    fp: &str,
    version: u32,
    publisher: &str,
) -> Result<(RustSourceInfo, SharedSource), SkError> {
    let http: Arc<dyn Http> = Arc::new(NetHttp(sk_net::Client::new()));
    let bad = |reason: String| SkError::Package { reason };
    let mut source = WasmSource::load(wasm, http).map_err(bad)?;
    let reply = source.call(r#"{"op":"info"}"#).map_err(bad)?;
    let v: serde_json::Value = serde_json::from_str(&reply).map_err(|e| bad(e.to_string()))?;
    let i = &v["info"];
    let (Some(name), Some(lang)) = (i["name"].as_str(), i["lang"].as_str()) else {
        return Err(bad("the source did not report a name and language".into()));
    };
    let id = match i["id"].as_i64() {
        Some(id) if id > 0 => id,
        _ => derive_id(name, lang),
    };
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
    };
    Ok((info, Arc::new(Mutex::new(source))))
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
                        instantiate(&pkg.wasm, &file_name, &fp, pkg.manifest.version, &pkg.manifest.publisher)
                            .map_err(|e| (e.to_string(), String::new()))
                    } else {
                        Err(("the publisher is not trusted".to_string(), fp))
                    }
                }
            },
            "wasm" if allow_unsigned => {
                instantiate(&bytes, &file_name, "", 0, "unsigned").map_err(|e| (e.to_string(), String::new()))
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
    let source = registry()
        .lock()
        .unwrap()
        .get(&id)
        .cloned()
        .ok_or_else(|| SkError::Net { reason: format!("no Rust source with id {id}") })?;
    let mut guard = source.lock().unwrap();
    guard.call(&request).map_err(|reason| SkError::Net { reason })
}

#[cfg(test)]
mod tests {
    use super::slug;

    #[test]
    fn file_names_are_safe_and_stable() {
        assert_eq!(slug("MangaDex (Rust)", "en"), "mangadex--rust-en");
        assert!(!slug("../../etc/passwd", "en").contains(['/', '.']));
    }
}

