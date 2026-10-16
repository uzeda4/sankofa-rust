//! Host for sources written in Rust (or anything else that compiles to WebAssembly).
//!
//! A source is a `.wasm` module that exports a tiny JSON-over-memory ABI (see `docs/redesign/RUST-SOURCES.md`):
//!
//! * `sk_alloc(len) -> ptr`, `sk_free(ptr, len)`: guest allocator, used by the host to pass bytes in.
//! * `sk_call(ptr, len) -> i64`: takes a JSON request, returns `(ptr << 32) | len` of a JSON response that
//!   lives in guest memory (the host frees it with `sk_free`).
//!
//! The host provides `sk.http_get(url, headers) -> response` and `sk.log(msg)`; sources never open sockets.
//! Execution is fuel-metered, so a looping source cannot hang the app.

pub mod package;

use std::sync::{Arc, Mutex};
use wasmi::{Caller, Engine, Extern, Linker, Memory, Module, Store, TypedFunc};

/// HTTP as seen by sources. Implemented over `sk-net` by the app; faked in tests.
pub trait Http: Send + Sync {
    /// Returns `(status, body)` for a GET. `headers` are `(name, value)` pairs.
    fn get(&self, url: &str, headers: &[(String, String)]) -> Result<(u16, String), String>;
}

/// Default instructions one call may execute before it is cut off (roughly a few seconds on a phone).
const DEFAULT_FUEL: u64 = 400_000_000;
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
                    let reply = match http.get(&url, &headers) {
                        Ok((status, body)) => serde_json::json!({ "status": status, "body": body }),
                        Err(error) => serde_json::json!({ "status": 0, "error": error }),
                    };
                    write_to_guest(&mut caller, reply.to_string().as_bytes()).unwrap_or(0)
                },
            )
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

    /// Messages the source wrote with `sk.log` since the last call to this method.
    pub fn take_logs(&mut self) -> Vec<String> {
        std::mem::take(&mut self.store.data_mut().logs)
    }
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
        fn get(&self, url: &str, headers: &[(String, String)]) -> Result<(u16, String), String> {
            Ok((200, format!("{url}|{}", headers.len())))
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
    fn a_module_without_the_abi_is_refused() {
        let err = load("(module)").err().unwrap();
        assert!(err.contains("memory") || err.contains("sk_alloc"), "{err}");
    }
}

