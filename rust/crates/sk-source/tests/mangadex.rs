//! Runs the real `sk-source-mangadex` WebAssembly module in the host against the live MangaDex API.
//! Needs the module built first and network access:
//!
//! ```text
//! cargo build -p sk-source-mangadex --release --target wasm32-unknown-unknown
//! SK_TEST_WASM=target/wasm32-unknown-unknown/release/sk_source_mangadex.wasm cargo test -p sk-source --test mangadex -- --nocapture
//! ```
//! The PC has no C compiler for `sk-net`, so HTTP goes through `curl` here; the app uses `sk-net`.

use serde_json::Value;
use sk_source::{Http, WasmSource};
use std::process::Command;
use std::sync::Arc;

struct Curl;
impl Http for Curl {
    fn get(&self, url: &str, headers: &[(String, String)]) -> Result<(u16, String), String> {
        let mut cmd = Command::new("curl");
        cmd.args(["-sS", "-L", "--max-time", "30", "-A", "sankofa-test", "-w", "\n%{http_code}"]);
        for (k, v) in headers {
            cmd.args(["-H", &format!("{k}: {v}")]);
        }
        // `[]` in the query is literal for MangaDex; stop curl from treating it as a glob.
        let out = cmd.arg("-g").arg(url).output().map_err(|e| e.to_string())?;
        let text = String::from_utf8_lossy(&out.stdout).into_owned();
        let (body, status) = text.rsplit_once('\n').ok_or("empty response")?;
        Ok((status.trim().parse().map_err(|_| format!("bad status {status}"))?, body.to_string()))
    }
}

fn load() -> Option<WasmSource> {
    let path = std::env::var("SK_TEST_WASM").ok()?;
    Some(WasmSource::load(&std::fs::read(path).unwrap(), Arc::new(Curl)).unwrap())
}

fn call(src: &mut WasmSource, request: &str) -> Value {
    let v: Value = serde_json::from_str(&src.call(request).unwrap()).unwrap();
    assert!(v.get("error").is_none(), "source error: {v}");
    v
}

#[test]
fn the_rust_source_browses_reads_and_pages_a_real_title() {
    let Some(mut src) = load() else { return };

    let info = call(&mut src, r#"{"op":"info"}"#);
    assert_eq!(info["info"]["lang"], "en");

    let popular = call(&mut src, r#"{"op":"popular","page":1}"#);
    let mangas = popular["page"]["mangas"].as_array().unwrap();
    println!("popular: {} titles, first {:?}", mangas.len(), mangas[0]["title"]);
    assert_eq!(mangas.len(), 20);
    assert!(mangas[0]["thumbnail_url"].as_str().unwrap().starts_with("https://uploads.mangadex.org/covers/"));
    assert_eq!(popular["page"]["has_next"], true);

    let search = call(&mut src, r#"{"op":"search","page":1,"query":"one piece"}"#);
    println!("search: first {:?}", search["page"]["mangas"][0]["title"]);
    assert!(!search["page"]["mangas"].as_array().unwrap().is_empty());

    let url = mangas[0]["url"].as_str().unwrap();
    let details = call(&mut src, &format!(r#"{{"op":"details","url":"{url}"}}"#));
    println!("details: {:?}, status {}, author {:?}, artist {:?}", details["manga"]["title"], details["manga"]["status"], details["manga"]["author"], details["manga"]["artist"]);

    // The first popular title may have only external chapters; walk until one has readable pages.
    for m in mangas {
        let url = m["url"].as_str().unwrap();
        let chapters = call(&mut src, &format!(r#"{{"op":"chapters","url":"{url}"}}"#));
        let list = chapters["chapters"].as_array().unwrap();
        if let Some(ch) = list.first() {
            println!("{}: {} chapters, newest {:?}", m["title"], list.len(), ch["name"]);
            let pages = call(&mut src, &format!(r#"{{"op":"pages","url":"{}"}}"#, ch["url"].as_str().unwrap()));
            let pages = pages["pages"].as_array().unwrap();
            println!("pages: {}, first {:?}", pages.len(), pages[0]["image_url"]);
            assert!(!pages.is_empty());
            assert!(pages[0]["image_url"].as_str().unwrap().contains("/data/"));
            return;
        }
    }
    panic!("no popular title had readable chapters");
}

