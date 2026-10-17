//! Helpers for running a source's WebAssembly module on the PC. The PC has no C compiler for `sk-net`, so live
//! tests go through the `curl` binary; the app uses its own OkHttp client.

use crate::{Http, HttpReply, HttpRequest};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct Curl {
    pub user_agent: String,
}

impl Default for Curl {
    fn default() -> Self {
        Self {
            user_agent: "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0 Mobile Safari/537.36".into(),
        }
    }
}

impl Http for Curl {
    fn request(&self, req: &HttpRequest) -> HttpReply {
        let id = format!("{}-{}", std::process::id(), COUNTER.fetch_add(1, Ordering::Relaxed));
        let dir = std::env::temp_dir();
        let (head, body) = (dir.join(format!("sk-curl-{id}.h")), dir.join(format!("sk-curl-{id}.b")));
        let mut cmd = Command::new("curl");
        cmd.args(["-sS", "-g", "--max-time", "40", "-A", &self.user_agent, "-X", if req.method.is_empty() { "GET" } else { &req.method }]);
        if req.follow_redirects {
            cmd.arg("-L");
        }
        cmd.args(["-D", &head.to_string_lossy(), "-o", &body.to_string_lossy(), "-w", "%{http_code} %{url_effective}"]);
        for (k, v) in &req.headers {
            cmd.args(["-H", &format!("{k}: {v}")]);
        }
        if let Some(ct) = &req.content_type {
            cmd.args(["-H", &format!("Content-Type: {ct}")]);
        }
        if let Some(b) = &req.body {
            cmd.args(["--data-binary", b]);
        }
        let out = match cmd.arg(&req.url).output() {
            Ok(o) => o,
            Err(e) => return HttpReply::failed(e.to_string()),
        };
        let info = String::from_utf8_lossy(&out.stdout).into_owned();
        let headers_raw = std::fs::read_to_string(&head).unwrap_or_default();
        let bytes = std::fs::read(&body).unwrap_or_default();
        let _ = (std::fs::remove_file(&head), std::fs::remove_file(&body));
        if !out.status.success() {
            return HttpReply::failed(String::from_utf8_lossy(&out.stderr).trim().to_string());
        }
        let (code, url) = info.split_once(' ').unwrap_or(("0", ""));
        // With -L the header file holds every hop; keep the last block.
        let last = headers_raw.rsplit("\r\n\r\n").find(|b| b.starts_with("HTTP/")).unwrap_or("");
        let headers = last
            .lines()
            .skip(1)
            .filter_map(|l| l.split_once(':'))
            .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
            .collect();
        let (body, body_b64) = if req.binary {
            use base64::Engine;
            (String::new(), Some(base64::engine::general_purpose::STANDARD.encode(&bytes)))
        } else {
            (String::from_utf8_lossy(&bytes).into_owned(), None)
        };
        HttpReply { status: code.parse().unwrap_or(0), headers, body, body_b64, url: url.to_string(), error: None }
    }
}
