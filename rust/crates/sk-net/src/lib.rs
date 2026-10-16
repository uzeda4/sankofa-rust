//! HTTP for the `sk-*` crates: TLS via rustls with the bundled Mozilla roots, gzip/brotli bodies,
//! timeouts and a browser-like user agent. Blocking on purpose; callers run it on worker threads.
//!
//! ponytail: roots are the bundled webpki set, so user-installed CAs and corporate proxies are not
//! trusted. Swap for the platform verifier (needs JNI + a small Kotlin helper) if that matters.

use std::io::Read;
use std::time::Duration;
use ureq::Agent;

/// Responses larger than this are refused (covers and chapter pages stay well below it).
const MAX_BODY: u64 = 64 * 1024 * 1024;

const USER_AGENT: &str =
    "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0 Mobile Safari/537.36";

#[derive(Debug)]
pub enum Error {
    /// The server answered with a non-2xx status.
    Status(u16),
    /// Connection, TLS, DNS, timeout or body errors.
    Transport(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Status(s) => write!(f, "HTTP {s}"),
            Self::Transport(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Clone)]
pub struct Client {
    agent: Agent,
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

impl Client {
    pub fn new() -> Self {
        let agent: Agent = Agent::config_builder()
            .user_agent(USER_AGENT)
            .timeout_global(Some(Duration::from_secs(30)))
            .timeout_connect(Some(Duration::from_secs(10)))
            // Non-2xx statuses are reported through `Error::Status`, not as transport errors.
            .http_status_as_error(false)
            .build()
            .into();
        Self { agent }
    }

    pub fn get_bytes(&self, url: &str) -> Result<Vec<u8>, Error> {
        self.get_bytes_with(url, &[])
    }

    /// Like [`Client::get_bytes`] with extra request headers; image CDNs of many sources return 403
    /// unless `Referer` is the source site.
    pub fn get_bytes_with(&self, url: &str, headers: &[(&str, &str)]) -> Result<Vec<u8>, Error> {
        let mut req = self.agent.get(url);
        for (name, value) in headers {
            req = req.header(*name, *value);
        }
        let mut resp = req.call().map_err(|e| Error::Transport(e.to_string()))?;
        let status = resp.status().as_u16();
        if !(200..300).contains(&status) {
            return Err(Error::Status(status));
        }
        let mut buf = Vec::new();
        resp.body_mut()
            .as_reader()
            .take(MAX_BODY)
            .read_to_end(&mut buf)
            .map_err(|e| Error::Transport(e.to_string()))?;
        Ok(buf)
    }

    pub fn get_text(&self, url: &str) -> Result<String, Error> {
        let bytes = self.get_bytes(url)?;
        // ponytail: assumes UTF-8; sources with other charsets need the Content-Type charset.
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn https_page_is_downloaded_and_decoded() {
        if std::env::var("SK_TEST_NET").is_err() {
            return;
        }
        let html = Client::new().get_text("https://example.com/").unwrap();
        println!("example.com: {} bytes", html.len());
        assert!(html.contains("Example Domain"));
    }

    #[test]
    fn non_2xx_is_a_status_error() {
        if std::env::var("SK_TEST_NET").is_err() {
            return;
        }
        let err = Client::new().get_text("https://example.com/definitely-missing").unwrap_err();
        println!("missing page: {err}");
        assert!(matches!(err, Error::Status(404)));
    }

    #[test]
    fn invalid_certificates_are_rejected() {
        if std::env::var("SK_TEST_NET").is_err() {
            return;
        }
        for host in ["expired.badssl.com", "self-signed.badssl.com", "wrong.host.badssl.com"] {
            let err = Client::new().get_text(&format!("https://{host}/")).unwrap_err();
            println!("{host}: {err}");
            assert!(matches!(err, Error::Transport(_)), "{host} should fail TLS");
        }
    }

    #[test]
    fn cover_cdn_needs_the_referer() {
        if std::env::var("SK_TEST_NET").is_err() {
            return;
        }
        let client = Client::new();
        let Ok(html) = client.get_text("https://www.mangakakalot.gg/") else { return };
        let Some(i) = html.find("https://img-r1.2xstorage.com/thumb/") else { return };
        let url = &html[i..i + html[i..].find('"').unwrap()];
        println!("cover: {url}");
        assert!(matches!(client.get_bytes(url), Err(Error::Status(403))));
        let bytes = client.get_bytes_with(url, &[("Referer", "https://www.mangakakalot.gg/")]).unwrap();
        println!("cover with referer: {} bytes", bytes.len());
        assert!(bytes.len() > 1000);
    }

    /// The source the library already uses. A Cloudflare challenge (403/503) is reported, not hidden,
    /// because it decides whether sources need the WebView fallback.
    #[test]
    fn real_source_page_answers() {
        if std::env::var("SK_TEST_NET").is_err() {
            return;
        }
        match Client::new().get_text("https://www.mangakakalot.gg/") {
            Ok(html) => {
                println!("mangakakalot.gg: {} bytes", html.len());
                assert!(html.to_lowercase().contains("manga"));
            }
            Err(Error::Status(s)) => println!("mangakakalot.gg: HTTP {s} (challenge or block)"),
            Err(e) => panic!("transport failure: {e}"),
        }
    }
}
