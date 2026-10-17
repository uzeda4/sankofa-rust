//! Host for sources written in Rust (or anything else that compiles to WebAssembly).
//!
//! A source is a `.wasm` module that exports a tiny JSON-over-memory ABI (see `docs/redesign/RUST-SOURCES.md`):
//!
//! * `sk_alloc(len) -> ptr`, `sk_free(ptr, len)`: guest allocator, used by the host to pass bytes in.
//! * `sk_call(ptr, len) -> i64`: takes a JSON request, returns `(ptr << 32) | len` of a JSON response that
//!   lives in guest memory (the host frees it with `sk_free`).
//!
//! The host provides `sk.http(request) -> response`, `sk.http_batch(requests) -> responses` (run in parallel),
//! `sk.now_ms()`, `sk.log(msg)` and the older `sk.http_get(url, headers)`; sources never open sockets.
//! Execution is fuel-metered, so a looping source cannot hang the app.

pub mod package;
pub mod repo;
#[cfg(feature = "testing")]
pub mod testing;

use std::sync::{Arc, Mutex};
use wasmi::{Caller, Engine, Extern, Linker, Memory, Module, Store, TypedFunc};

/// A request a source asked for (the JSON the guest sends to `sk.http`).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct HttpRequest {
    #[serde(default)]
    pub method: String,
    pub url: String,
    #[serde(default)]
    pub headers: Vec<(String, String)>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub content_type: Option<String>,
    #[serde(default = "yes")]
    pub follow_redirects: bool,
    /// Answer with the body as base64 in `body_b64` (images, partial reads) instead of text in `body`.
    #[serde(default)]
    pub binary: bool,
    /// Read at most this many bytes of the body (a server that ignores `Range` must not send a whole image).
    #[serde(default)]
    pub max_bytes: Option<u64>,
}

fn yes() -> bool {
    true
}

/// What the host answers: `error` is set instead of a status when the request could not be made.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct HttpReply {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
    /// The body as base64, when the request asked for `binary`.
    #[serde(default)]
    pub body_b64: Option<String>,
    /// Final URL after redirects.
    pub url: String,
    pub error: Option<String>,
}

impl HttpReply {
    pub fn failed(error: impl Into<String>) -> Self {
        Self { error: Some(error.into()), ..Default::default() }
    }
}

/// HTTP as seen by sources. Implemented over the app's own client (Cloudflare, cookies, rate limits) on the device;
/// faked in tests.
pub trait Http: Send + Sync {
    /// Sends one request. A non-2xx answer is a reply with that status, not an error.
    fn request(&self, req: &HttpRequest) -> HttpReply;

    /// The value of cookie `name` held for `url`.
    fn cookie(&self, _url: &str, _name: &str) -> Option<String> {
        None
    }
}

/// Requests of one `http_batch` that run at the same time.
const BATCH_PARALLELISM: usize = 6;

/// Applies the private-network guard, then asks the transport.
fn guarded(http: &dyn Http, req: &HttpRequest) -> HttpReply {
    if is_blocked_url(&req.url) {
        return HttpReply::failed("sources cannot reach local or private network addresses");
    }
    http.request(req)
}

fn run_batch(http: &Arc<dyn Http>, reqs: &[HttpRequest]) -> Vec<HttpReply> {
    let mut out: Vec<HttpReply> = Vec::with_capacity(reqs.len());
    for chunk in reqs.chunks(BATCH_PARALLELISM) {
        std::thread::scope(|scope| {
            let handles: Vec<_> = chunk.iter().map(|r| scope.spawn(|| guarded(http.as_ref(), r))).collect();
            for h in handles {
                out.push(h.join().unwrap_or_else(|_| HttpReply::failed("request thread panicked")));
            }
        });
    }
    out
}

/// Default instructions one call may execute before it is cut off. Parsing a 700 KB page costs about 2 billion,
/// and a looping source is stopped after a few seconds on a phone.
const DEFAULT_FUEL: u64 = 6_000_000_000;
/// Largest request or response accepted in either direction.
const MAX_PAYLOAD: usize = 16 * 1024 * 1024;

struct HostState {
    http: Arc<dyn Http>,
    logs: Vec<String>,
}

pub struct WasmSource {
    fuel_per_call: u64,
    store: Store<HostState>,
    memory: Memory,
    alloc: TypedFunc<i32, i32>,
    free: TypedFunc<(i32, i32), ()>,
    call: TypedFunc<(i32, i32), i64>,
}

impl WasmSource {
    pub fn load(wasm: &[u8], http: Arc<dyn Http>) -> Result<Self, String> {
        let mut config = wasmi::Config::default();
        config.consume_fuel(true);
        let engine = Engine::new(&config);
        let module = Module::new(&engine, wasm).map_err(|e| format!("invalid module: {e}"))?;
        let mut store = Store::new(&engine, HostState { http, logs: Vec::new() });
        store.set_fuel(DEFAULT_FUEL).map_err(|e| e.to_string())?;

        let mut linker = <Linker<HostState>>::new(&engine);
        linker
            .func_wrap("sk", "log", |mut caller: Caller<'_, HostState>, ptr: i32, len: i32| {
                if let Some(msg) = read_string(&mut caller, ptr, len) {
                    caller.data_mut().logs.push(msg);
                }
            })
            .map_err(|e| e.to_string())?;
        // ABI v1: GET with headers, answers `{"status":..,"body":..}`.
        linker
            .func_wrap(
                "sk",
                "http_get",
                |mut caller: Caller<'_, HostState>, url_ptr: i32, url_len: i32, hdr_ptr: i32, hdr_len: i32| -> i64 {
                    let url = read_string(&mut caller, url_ptr, url_len).unwrap_or_default();
                    let headers: Vec<(String, String)> = read_string(&mut caller, hdr_ptr, hdr_len)
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap_or_default();
                    let http = caller.data().http.clone();
                    let req = HttpRequest { method: "GET".into(), url, headers, follow_redirects: true, ..Default::default() };
                    let reply = match guarded(http.as_ref(), &req) {
                        HttpReply { error: Some(error), .. } => serde_json::json!({ "status": 0, "error": error }),
                        r => serde_json::json!({ "status": r.status, "body": r.body }),
                    };
                    write_to_guest(&mut caller, reply.to_string().as_bytes()).unwrap_or(0)
                },
            )
            .map_err(|e| e.to_string())?;
        // ABI v2: one request, full control.
        linker
            .func_wrap("sk", "http", |mut caller: Caller<'_, HostState>, ptr: i32, len: i32| -> i64 {
                let http = caller.data().http.clone();
                let reply = match read_string(&mut caller, ptr, len).and_then(|s| serde_json::from_str::<HttpRequest>(&s).ok()) {
                    Some(req) => guarded(http.as_ref(), &req),
                    None => HttpReply::failed("malformed request"),
                };
                write_to_guest(&mut caller, serde_json::to_string(&reply).unwrap_or_default().as_bytes()).unwrap_or(0)
            })
            .map_err(|e| e.to_string())?;
        // ABI v2: several requests, answered in order after running in parallel.
        linker
            .func_wrap("sk", "http_batch", |mut caller: Caller<'_, HostState>, ptr: i32, len: i32| -> i64 {
                let http = caller.data().http.clone();
                let replies = match read_string(&mut caller, ptr, len).and_then(|s| serde_json::from_str::<Vec<HttpRequest>>(&s).ok()) {
                    Some(reqs) => run_batch(&http, &reqs),
                    None => vec![HttpReply::failed("malformed request")],
                };
                write_to_guest(&mut caller, serde_json::to_string(&replies).unwrap_or_default().as_bytes()).unwrap_or(0)
            })
            .map_err(|e| e.to_string())?;
        linker
            .func_wrap("sk", "cookie", |mut caller: Caller<'_, HostState>, url_ptr: i32, url_len: i32, name_ptr: i32, name_len: i32| -> i64 {
                let url = read_string(&mut caller, url_ptr, url_len).unwrap_or_default();
                let name = read_string(&mut caller, name_ptr, name_len).unwrap_or_default();
                let http = caller.data().http.clone();
                let value = if is_blocked_url(&url) { None } else { http.cookie(&url, &name) };
                write_to_guest(&mut caller, value.unwrap_or_default().as_bytes()).unwrap_or(0)
            })
            .map_err(|e| e.to_string())?;
        linker
            .func_wrap("sk", "now_ms", |_: Caller<'_, HostState>| -> i64 {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as i64)
                    .unwrap_or(0)
            })
            .map_err(|e| e.to_string())?;

        let instance = linker
            .instantiate_and_start(&mut store, &module)
            .map_err(|e| format!("cannot instantiate: {e}"))?;
        let memory = instance.get_memory(&store, "memory").ok_or("source exports no `memory`")?;
        let alloc = instance.get_typed_func(&store, "sk_alloc").map_err(|e| format!("sk_alloc: {e}"))?;
        let free = instance.get_typed_func(&store, "sk_free").map_err(|e| format!("sk_free: {e}"))?;
        let call = instance.get_typed_func(&store, "sk_call").map_err(|e| format!("sk_call: {e}"))?;
        Ok(Self { fuel_per_call: DEFAULT_FUEL, store, memory, alloc, free, call })
    }

    /// Overrides the instruction budget of each call.
    pub fn set_fuel_per_call(&mut self, fuel: u64) {
        self.fuel_per_call = fuel;
    }

    /// Sends one JSON request to the source and returns its JSON response.
    pub fn call(&mut self, request: &str) -> Result<String, String> {
        if request.len() > MAX_PAYLOAD {
            return Err("request too large".into());
        }
        self.store.set_fuel(self.fuel_per_call).map_err(|e| e.to_string())?;
        let len = request.len() as i32;
        let ptr = self.alloc.call(&mut self.store, len).map_err(|e| format!("sk_alloc failed: {e}"))?;
        self.memory.write(&mut self.store, ptr as usize, request.as_bytes()).map_err(|e| e.to_string())?;
        let packed = self.call.call(&mut self.store, (ptr, len)).map_err(|e| format!("source trapped: {e}"))?;
        let _ = self.free.call(&mut self.store, (ptr, len));
        let (out_ptr, out_len) = ((packed >> 32) as u32 as usize, packed as u32 as usize);
        if out_len > MAX_PAYLOAD {
            return Err("response too large".into());
        }
        let mut buf = vec![0u8; out_len];
        self.memory.read(&self.store, out_ptr, &mut buf).map_err(|e| e.to_string())?;
        let _ = self.free.call(&mut self.store, (out_ptr as i32, out_len as i32));
        String::from_utf8(buf).map_err(|e| e.to_string())
    }

    /// Instructions the last call executed (to size the budget and to compare sources).
    pub fn fuel_used(&self) -> u64 {
        self.fuel_per_call.saturating_sub(self.store.get_fuel().unwrap_or(0))
    }

    /// Messages the source wrote with `sk.log` since the last call to this method.
    pub fn take_logs(&mut self) -> Vec<String> {
        std::mem::take(&mut self.store.data_mut().logs)
    }
}

/// True for URLs a source must not reach: loopback, private, link-local and similar ranges, written as IP
/// literals, plus `localhost` and `.local`/`.internal` names. A source runs third-party code on the user's phone;
/// without this it could probe the home network (router admin pages, NAS, local servers).
///
/// ponytail: only the host as written is checked, not what a public name resolves to, so DNS rebinding is not
/// covered; resolve first and check the address in `sk-net` if that matters.
pub fn is_blocked_url(url: &str) -> bool {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    fn blocked_v4(ip: Ipv4Addr) -> bool {
        let o = ip.octets();
        ip.is_loopback()
            || ip.is_private()
            || ip.is_link_local()
            || ip.is_unspecified()
            || ip.is_broadcast()
            || o[0] == 0
            || (o[0] == 100 && (64..128).contains(&o[1])) // carrier-grade NAT
    }
    fn blocked_v6(ip: Ipv6Addr) -> bool {
        let s = ip.segments();
        ip.is_loopback()
            || ip.is_unspecified()
            || (s[0] & 0xfe00) == 0xfc00 // unique local
            || (s[0] & 0xffc0) == 0xfe80 // link-local
            || ip.to_ipv4_mapped().is_some_and(blocked_v4)
    }

    let rest = match url.split_once("://") {
        Some((scheme, rest)) if scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https") => rest,
        _ => return true, // not http(s): refuse
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    let host_port = authority.rsplit('@').next().unwrap_or("");
    let host = if let Some(v6) = host_port.strip_prefix('[') {
        v6.split(']').next().unwrap_or("")
    } else {
        host_port.split(':').next().unwrap_or("")
    };
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    if host.is_empty() {
        return true;
    }
    match host.parse::<IpAddr>() {
        Ok(IpAddr::V4(ip)) => blocked_v4(ip),
        Ok(IpAddr::V6(ip)) => blocked_v6(ip),
        Err(_) => {
            host == "localhost"
                || host.ends_with(".localhost")
                || host.ends_with(".local")
                || host.ends_with(".internal")
                || host.ends_with(".lan")
                // decimal/hex/octal IPv4 spellings such as http://2130706433/ or http://0x7f.1/
                || host.rsplit('.').next().is_some_and(|last| {
                    // a real TLD never starts with a digit, so a numeric last label means an IPv4 in disguise
                    last.starts_with(|c: char| c.is_ascii_digit()) && last.chars().all(|c| c.is_ascii_hexdigit() || c == 'x')
                })
        }
    }
}

/// The id an extension had: the first 8 bytes of MD5(`name.lowercase()/lang/version_id`) with the sign bit cleared.
/// A source converted from an extension with the same name, language and version id therefore keeps its library
/// entries, history and chapters.
pub fn extension_id(name: &str, lang: &str, version_id: u32) -> i64 {
    use md5::{Digest, Md5};
    let digest = Md5::digest(format!("{}/{lang}/{version_id}", name.to_lowercase()).as_bytes());
    let id = digest[..8].iter().fold(0u64, |acc, b| (acc << 8) | *b as u64);
    (id & (i64::MAX as u64)) as i64
}

/// A source shared between threads (the app calls from a pool).
pub type SharedSource = Arc<Mutex<WasmSource>>;

fn guest_memory(caller: &mut Caller<'_, HostState>) -> Option<Memory> {
    match caller.get_export("memory") {
        Some(Extern::Memory(m)) => Some(m),
        _ => None,
    }
}

fn read_string(caller: &mut Caller<'_, HostState>, ptr: i32, len: i32) -> Option<String> {
    if len < 0 || len as usize > MAX_PAYLOAD {
        return None;
    }
    let mem = guest_memory(caller)?;
    let mut buf = vec![0u8; len as usize];
    mem.read(&*caller, ptr as usize, &mut buf).ok()?;
    String::from_utf8(buf).ok()
}

/// Copies `bytes` into guest memory (through the guest's own allocator) and returns `(ptr << 32) | len`.
fn write_to_guest(caller: &mut Caller<'_, HostState>, bytes: &[u8]) -> Option<i64> {
    let alloc = caller.get_export("sk_alloc")?.into_func()?.typed::<i32, i32>(&*caller).ok()?;
    let ptr = alloc.call(&mut *caller, bytes.len() as i32).ok()?;
    guest_memory(caller)?.write(&mut *caller, ptr as usize, bytes).ok()?;
    Some(((ptr as u32 as i64) << 32) | bytes.len() as u32 as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeHttp;
    impl Http for FakeHttp {
        fn request(&self, req: &HttpRequest) -> HttpReply {
            HttpReply { status: 200, body: format!("{} {}|{}", req.method, req.url, req.headers.len()), url: req.url.clone(), ..Default::default() }
        }
    }

    /// A guest that echoes the request back, plus a variant that loops forever.
    const ECHO: &str = r#"(module
        (memory (export "memory") 1)
        (global $top (mut i32) (i32.const 1024))
        (func (export "sk_alloc") (param $n i32) (result i32)
            (local $p i32) (local.set $p (global.get $top))
            (global.set $top (i32.add (global.get $top) (local.get $n)))
            (local.get $p))
        (func (export "sk_free") (param i32 i32))
        (func (export "sk_call") (param $p i32) (param $n i32) (result i64)
            (i64.or (i64.shl (i64.extend_i32_u (local.get $p)) (i64.const 32)) (i64.extend_i32_u (local.get $n)))))"#;

    const LOOPING: &str = r#"(module
        (memory (export "memory") 1)
        (func (export "sk_alloc") (param i32) (result i32) (i32.const 0))
        (func (export "sk_free") (param i32 i32))
        (func (export "sk_call") (param i32 i32) (result i64) (loop $l (br $l)) (i64.const 0)))"#;

    fn load(wat: &str) -> Result<WasmSource, String> {
        WasmSource::load(&wat::parse_str(wat).unwrap(), Arc::new(FakeHttp))
    }

    #[test]
    fn a_request_round_trips_through_guest_memory() {
        let mut src = load(ECHO).unwrap();
        assert_eq!(src.call(r#"{"op":"info"}"#).unwrap(), r#"{"op":"info"}"#);
        assert_eq!(src.call("héllo").unwrap(), "héllo");
    }

    #[test]
    fn a_looping_source_is_cut_off_by_fuel() {
        let mut src = load(LOOPING).unwrap();
        src.set_fuel_per_call(1_000_000);
        let err = src.call("{}").unwrap_err();
        assert!(err.contains("trapped"), "{err}");
    }

    #[test]
    fn local_and_private_addresses_are_blocked_for_sources() {
        for url in [
            "http://localhost/x", "http://LOCALHOST:8080/", "http://127.0.0.1/", "http://127.1.2.3:9/", "https://10.0.0.5/a",
            "http://192.168.0.1/admin", "http://172.16.0.1/", "http://172.31.255.255/", "http://169.254.169.254/latest/meta-data",
            "http://0.0.0.0/", "http://100.64.0.1/", "http://[::1]/", "http://[fe80::1]/", "http://[fd00::1]/",
            "http://[::ffff:192.168.0.1]/", "http://user:pw@192.168.0.1/", "http://router.local/", "http://nas.lan/", "http://x.internal/",
            "http://2130706433/", "http://0x7f.0.0.1/", "http://printer.localhost./", "ftp://example.org/", "file:///etc/passwd", "http://",
        ] {
            assert!(is_blocked_url(url), "{url} should be blocked");
        }
        for url in [
            "https://api.mangadex.org/manga?limit=20", "http://example.com:8080/a", "https://uploads.mangadex.org/covers/x.jpg",
            "https://172.32.0.1/", "https://8.8.8.8/", "https://1a.de/x", "https://4chan.org/", "https://[2606:4700::1111]/", "https://user@example.org/",
        ] {
            assert!(!is_blocked_url(url), "{url} should be allowed");
        }
    }

    #[test]
    fn a_converted_source_gets_the_id_its_extension_had() {
        // Reference values computed with Python's hashlib.md5, the formula of `HttpSource.generateId`.
        assert_eq!(extension_id("Mangakakalot", "en", 1), 2_528_986_671_771_677_900);
        assert_eq!(extension_id("Asura Scans", "en", 1), 6_247_824_327_199_706_550);
        assert_eq!(extension_id("Webtoons.com", "en", 2), 8_510_171_328_472_873_347);
    }

    #[test]
    fn a_module_without_the_abi_is_refused() {
        let err = load("(module)").err().unwrap();
        assert!(err.contains("memory") || err.contains("sk_alloc"), "{err}");
    }
}


