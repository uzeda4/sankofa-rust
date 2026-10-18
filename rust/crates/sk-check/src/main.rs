//! Smoke test of a built source module against the real site: popular, details, chapters, pages.
//!   sk-check <module.wasm> [lang]
//! Prints one JSON line. The network is the system `curl` (the host build has no C compiler for TLS), so a site behind
//! Cloudflare answers 403/503 here and is reported as such; on the phone the app's own client solves that.

use serde_json::{json, Value};
use sk_source::{Http, HttpReply, HttpRequest, WasmSource};
use std::process::Command;
use std::sync::Arc;

struct Curl;

impl Http for Curl {
    fn request(&self, req: &HttpRequest) -> HttpReply {
        // one folder per request: the host runs the requests of an update in parallel
        static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!("sk-check-{}-{}", std::process::id(), N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)));
        let _ = std::fs::create_dir_all(&dir);
        let (head, body) = (dir.join("h"), dir.join("b"));
        let mut c = Command::new("curl");
        c.args(["-g", "-s", "-S", "--max-time", "40", "-A", "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0 Mobile Safari/537.36"]);
        if req.follow_redirects {
            c.arg("-L");
        }
        c.args(["-D", head.to_str().unwrap(), "-o", body.to_str().unwrap(), "-w", "%{http_code}\n%{url_effective}"]);
        if !req.method.is_empty() && !req.method.eq_ignore_ascii_case("GET") {
            c.args(["-X", &req.method]);
        }
        for (k, v) in &req.headers {
            c.args(["-H", &format!("{k}: {v}")]);
        }
        if let Some(ct) = &req.content_type {
            c.args(["-H", &format!("Content-Type: {ct}")]);
        }
        if let Some(b) = &req.body {
            c.args(["--data-raw", b]);
        }
        c.arg(&req.url);
        let out = match c.output() {
            Ok(o) => o,
            Err(e) => return HttpReply::failed(format!("curl: {e}")),
        };
        if !out.status.success() {
            return HttpReply::failed(format!("curl: {}", String::from_utf8_lossy(&out.stderr).trim()));
        }
        let text = String::from_utf8_lossy(&out.stdout).into_owned();
        let mut it = text.lines();
        let status: u16 = it.next().and_then(|s| s.trim().parse().ok()).unwrap_or(0);
        let url = it.next().unwrap_or_default().to_string();
        let bytes = std::fs::read(&body).unwrap_or_default();
        let headers = std::fs::read_to_string(&head)
            .unwrap_or_default()
            .lines()
            .filter_map(|l| l.split_once(':').map(|(k, v)| (k.trim().to_string(), v.trim().to_string())))
            .collect();
        let _ = std::fs::remove_dir_all(&dir);
        HttpReply { status, headers, body: String::from_utf8_lossy(&bytes).into_owned(), url, ..Default::default() }
    }
}

fn call(s: &mut WasmSource, req: Value) -> Result<Value, String> {
    let v: Value = serde_json::from_str(&s.call(&req.to_string())?).map_err(|e| e.to_string())?;
    match v.get("error").and_then(Value::as_str) {
        Some(e) => Err(e.to_string()),
        None => Ok(v),
    }
}


/// The picks the app sends when it searches with `source.getFilterList()` untouched (as a library migration does):
/// every filter at its default, keyed the way `RustFilters.state` does.
fn default_state(defs: &[Value]) -> Value {
    let mut o = serde_json::Map::new();
    for d in defs {
        let Some(key) = d["key"].as_str() else { continue };
        let v = match d["type"].as_str() {
            Some("select") => json!(d["default"].as_i64().unwrap_or(0)),
            Some("checkbox") => json!(d["default"].as_bool().unwrap_or(false)),
            Some("tristate") => json!(0),
            Some("text") => json!(d["default"].as_str().unwrap_or_default()),
            Some("group") => default_state(d["items"].as_array().map(Vec::as_slice).unwrap_or_default()),
            Some("sort") => json!({ "index": d["default_index"].as_i64().unwrap_or(0), "ascending": d["default_ascending"].as_bool().unwrap_or(false) }),
            _ => continue,
        };
        o.insert(key.to_string(), v);
    }
    Value::Object(o)
}

fn verdict(error: &str) -> &'static str {
    let e = error.to_lowercase();
    if e.contains("http 403") || e.contains("http 503") || e.contains("just a moment") || e.contains("cloudflare") || e.contains("http 429") {
        "cloudflare"
    } else if e.contains("resolve host") || e.contains("couldn't connect") || e.contains("timed out") || e.contains("ssl") || e.contains("http 404") || e.contains("http 5") {
        "dead"
    } else {
        "broken"
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let wasm = std::fs::read(&args[1]).expect("module");
    // a published package: check the module inside it (what the phone downloaded), signature included
    let wasm = if args[1].ends_with(".sksource") { sk_source::package::decode_and_verify(&wasm).expect("package").wasm } else { wasm };
    let lang = args.get(2).cloned().unwrap_or_else(|| "en".into());
    // SK_MIRROR=https://other.site tests a source against one of its mirrors (the "Domain" setting of the app).
    let ctx = match std::env::var("SK_MIRROR") {
        Ok(m) => json!({ "lang": lang, "prefs": { "mirror": m } }),
        Err(_) => json!({ "lang": lang }),
    };
    let mut s = WasmSource::load(&wasm, Arc::new(Curl)).expect("load");
    let mut out = json!({ "module": args[1] });
    let result: Result<(), String> = (|| {
        let info = call(&mut s, json!({ "op": "info", "ctx": ctx }))?;
        out["name"] = info["info"]["name"].clone();
        if std::env::var("SK_INFO").is_ok() {
            out["info"] = info["info"].clone();
        }
        out["base"] = info["info"]["base_url"].clone();
        // SK_QUERY=title checks the first search result instead of the first popular one
        // SK_LEGACY_URL=<address> SK_LEGACY_TITLE=<title>: a saved title is opened as the app would (details and chapters)
        if let (Ok(url), Ok(title)) = (std::env::var("SK_LEGACY_URL"), std::env::var("SK_LEGACY_TITLE")) {
            let manga = json!({ "url": url, "title": title });
            let upd = call(&mut s, json!({ "op": "update", "manga": manga, "fetch_details": true, "fetch_chapters": true, "ctx": ctx }))?;
            out["healed_memo"] = upd["update"]["manga"]["memo"].clone();
            out["healed_title"] = upd["update"]["manga"]["title"].clone();
            out["healed_chapters"] = json!(upd["update"]["chapters"].as_array().map(Vec::len).unwrap_or(0));
            return Ok(());
        }
        let page = match std::env::var("SK_QUERY") {
            Ok(q) => call(&mut s, json!({ "op": "search", "page": 1, "query": q, "ctx": ctx }))?,
            // SK_OP=latest and SK_PAGE=n list another page (a migration meets titles of every popularity)
            Err(_) => call(
                &mut s,
                json!({ "op": std::env::var("SK_OP").unwrap_or_else(|_| "popular".into()), "page": std::env::var("SK_PAGE").ok().and_then(|p| p.parse::<u32>().ok()).unwrap_or(1), "ctx": ctx }),
            )?,
        };
        let mangas = page["page"]["mangas"].as_array().cloned().unwrap_or_default();
        out["popular"] = json!(mangas.len());
        out["titles"] = json!(mangas.iter().take(5).map(|m| m["title"].clone()).collect::<Vec<_>>());
        // SK_SELFSEARCH=1: does searching a listed title's own name bring that title back? (what a library migration does)
        if std::env::var("SK_SELFSEARCH").is_ok() {
            let defs = call(&mut s, json!({ "op": "filters", "ctx": ctx }))?;
            out["filters"] = json!(defs["filters"].as_array().map(Vec::len).unwrap_or(0));
            let state = default_state(defs["filters"].as_array().map(Vec::as_slice).unwrap_or_default());
            let (mut found, mut misses) = (0, Vec::new());
            for m in &mangas {
                let title = m["title"].as_str().unwrap_or_default();
                let hits = call(&mut s, json!({ "op": "search", "page": 1, "query": title.trim(), "filters": state, "ctx": ctx }))?;
                let hits = hits["page"]["mangas"].as_array().cloned().unwrap_or_default();
                match hits.iter().position(|h| h["url"] == m["url"]) {
                    Some(_) => found += 1,
                    None => misses.push(json!({ "title": title, "results": hits.len(), "first": hits.first().map(|h| h["title"].clone()) })),
                }
            }
            out["selfsearch_found"] = json!(found);
            out["selfsearch_misses"] = json!(misses);
        }
        let first = mangas.first().ok_or("popular is empty")?.clone();
        out["first"] = first["title"].clone();
        let upd = call(&mut s, json!({ "op": "update", "manga": first, "fetch_details": true, "fetch_chapters": true, "ctx": ctx }))?;
        let chapters = upd["update"]["chapters"].as_array().cloned().unwrap_or_default();
        out["chapters"] = json!(chapters.len());
        // the newest chapter may be locked behind a paywall: try the oldest too
        let first = chapters.first().ok_or("no chapters")?.clone();
        let last = chapters.last().cloned().unwrap_or_else(|| first.clone());
        let pages = match call(&mut s, json!({ "op": "pages", "chapter": first, "ctx": ctx })) {
            Ok(p) => p,
            Err(_) => call(&mut s, json!({ "op": "pages", "chapter": last, "ctx": ctx }))?,
        };
        let n = pages["pages"].as_array().map(Vec::len).unwrap_or(0);
        out["pages"] = json!(n);
        out["image"] = pages["pages"][0]["image_url"].clone();
        if n == 0 {
            return Err("no pages".into());
        }
        let s2 = call(&mut s, json!({ "op": "search", "page": 1, "query": "love", "ctx": ctx }));
        out["search"] = json!(s2.map(|v| v["page"]["mangas"].as_array().map(Vec::len).unwrap_or(0)).unwrap_or(0));
        Ok(())
    })();
    match result {
        Ok(()) => out["status"] = json!("ok"),
        Err(e) => {
            out["status"] = json!(verdict(&e));
            out["error"] = json!(e.chars().take(160).collect::<String>());
        }
    }
    println!("{out}");
}
