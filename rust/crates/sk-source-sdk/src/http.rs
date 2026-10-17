//! Network access. A source never opens sockets: requests go to the host, which sends them through the app's own
//! HTTP client (cookies, Cloudflare handling, rate limits, user agent) and refuses local and private addresses.

use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, String>;

/// What the host is asked to send.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
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
    /// Ask for the body as bytes (`Response::bytes`); text answers go through `body`.
    #[serde(default)]
    pub binary: bool,
    #[serde(default)]
    pub max_bytes: Option<u64>,
}

fn yes() -> bool {
    true
}

/// What the host answers: `error` is set for transport failures (no status).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HttpResponse {
    #[serde(default)]
    pub status: u16,
    #[serde(default)]
    pub headers: Vec<(String, String)>,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub body_b64: Option<String>,
    /// The URL finally answered, after redirects.
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub error: Option<String>,
}

/// A request being built; `send` runs it.
#[derive(Debug, Clone)]
pub struct Request {
    req: HttpRequest,
    ensure_success: bool,
}

pub fn get(url: &str) -> Request {
    Request::new("GET", url)
}

pub fn post(url: &str) -> Request {
    Request::new("POST", url)
}

impl Request {
    fn new(method: &str, url: &str) -> Self {
        Self {
            req: HttpRequest { method: method.into(), url: url.into(), headers: Vec::new(), body: None, content_type: None, follow_redirects: true, binary: false, max_bytes: None },
            ensure_success: true,
        }
    }

    /// Sets a header, replacing an earlier one of the same name.
    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.req.headers.retain(|(n, _)| !n.eq_ignore_ascii_case(name));
        self.req.headers.push((name.into(), value.into()));
        self
    }

    pub fn headers(mut self, headers: &[(&str, &str)]) -> Self {
        for (n, v) in headers {
            self = self.header(n, v);
        }
        self
    }

    /// `application/x-www-form-urlencoded` body.
    pub fn form(mut self, fields: &[(&str, &str)]) -> Self {
        let body = fields.iter().map(|(k, v)| format!("{}={}", crate::url_encode(k), crate::url_encode(v))).collect::<Vec<_>>().join("&");
        self.req.body = Some(body);
        self.req.content_type = Some("application/x-www-form-urlencoded".into());
        self
    }

    pub fn json<T: Serialize>(mut self, value: &T) -> Self {
        self.req.body = Some(serde_json::to_string(value).unwrap_or_default());
        self.req.content_type = Some("application/json".into());
        self
    }

    pub fn body(mut self, text: &str, content_type: &str) -> Self {
        self.req.body = Some(text.into());
        self.req.content_type = Some(content_type.into());
        self
    }

    /// Keep non-2xx answers instead of turning them into an error (to read a 404 page, or an API's error body).
    pub fn allow_errors(mut self) -> Self {
        self.ensure_success = false;
        self
    }

    pub fn no_redirects(mut self) -> Self {
        self.req.follow_redirects = false;
        self
    }

    /// Read the answer as bytes (`Response::bytes`), for images.
    pub fn binary(mut self) -> Self {
        self.req.binary = true;
        self
    }

    /// Only bytes `start..=end` (the sizes of an image sit in its first few bytes). Implies [`Request::binary`].
    pub fn range(mut self, start: u64, end: u64) -> Self {
        self.req.max_bytes = Some(end + 1);
        self.binary().header("Range", &format!("bytes={start}-{end}"))
    }

    pub fn send(self) -> Result<Response> {
        let ensure = self.ensure_success;
        let url = self.req.url.clone();
        let reply = transport(std::slice::from_ref(&self.req)).pop().ok_or("host returned no response")?;
        finish(reply, ensure, &url)
    }
}

fn finish(reply: HttpResponse, ensure: bool, url: &str) -> Result<Response> {
    if let Some(e) = reply.error {
        return Err(format!("{url}: {e}"));
    }
    if ensure && !(200..300).contains(&reply.status) {
        return Err(format!("HTTP {} for {url}", reply.status));
    }
    Ok(Response {
        status: reply.status,
        headers: reply.headers,
        body: reply.body,
        body_b64: reply.body_b64,
        url: if reply.url.is_empty() { url.into() } else { reply.url },
    })
}

/// Sends several requests at once (the host runs them in parallel) and returns the answers in the same order.
pub fn send_all(requests: Vec<Request>) -> Vec<Result<Response>> {
    let reqs: Vec<HttpRequest> = requests.iter().map(|r| r.req.clone()).collect();
    let replies = transport(&reqs);
    requests
        .iter()
        .zip(replies)
        .map(|(r, reply)| finish(reply, r.ensure_success, &r.req.url))
        .collect()
}

#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
    body_b64: Option<String>,
    /// Final URL after redirects; relative links on the page resolve against it.
    pub url: String,
}

impl Response {
    pub fn text(&self) -> &str {
        &self.body
    }

    pub fn json<T: for<'de> Deserialize<'de>>(&self) -> Result<T> {
        serde_json::from_str(&self.body).map_err(|e| format!("bad JSON from {}: {e}", self.url))
    }

    /// The body as bytes: what `binary()` asked for, or the text's UTF-8 bytes otherwise.
    pub fn bytes(&self) -> Vec<u8> {
        match &self.body_b64 {
            Some(b) => {
                use base64::Engine;
                base64::engine::general_purpose::STANDARD.decode(b).unwrap_or_default()
            }
            None => self.body.clone().into_bytes(),
        }
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find(|(n, _)| n.eq_ignore_ascii_case(name)).map(|(_, v)| v.as_str())
    }

    /// The body as an HTML document whose relative links resolve against the final URL.
    #[cfg(feature = "html")]
    pub fn html(&self) -> sk_html::Document {
        sk_html::Document::parse(&self.body, &self.url)
    }
}

// ------------------------------------------------------------------ transport

#[cfg(target_arch = "wasm32")]
mod host {
    #[link(wasm_import_module = "sk")]
    extern "C" {
        pub fn http(ptr: *const u8, len: i32) -> i64;
        pub fn http_batch(ptr: *const u8, len: i32) -> i64;
        pub fn now_ms() -> i64;
        pub fn cookie(url_ptr: *const u8, url_len: i32, name_ptr: *const u8, name_len: i32) -> i64;
        pub fn log(ptr: *const u8, len: i32);
    }
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

fn failed(msg: &str, n: usize) -> Vec<HttpResponse> {
    (0..n).map(|_| HttpResponse { error: Some(msg.into()), ..Default::default() }).collect()
}

#[cfg(target_arch = "wasm32")]
fn transport(reqs: &[HttpRequest]) -> Vec<HttpResponse> {
    let call = |batch: bool| -> Option<Vec<u8>> {
        let json = if batch { serde_json::to_string(reqs) } else { serde_json::to_string(&reqs[0]) }.ok()?;
        let packed = unsafe {
            if batch {
                host::http_batch(json.as_ptr(), json.len() as i32)
            } else {
                host::http(json.as_ptr(), json.len() as i32)
            }
        };
        unsafe { take_guest_bytes(packed) }
    };
    if reqs.is_empty() {
        return Vec::new();
    }
    if reqs.len() == 1 {
        return match call(false).and_then(|b| serde_json::from_slice::<HttpResponse>(&b).ok()) {
            Some(r) => vec![r],
            None => failed("the host returned no usable response", 1),
        };
    }
    match call(true).and_then(|b| serde_json::from_slice::<Vec<HttpResponse>>(&b).ok()) {
        Some(r) if r.len() == reqs.len() => r,
        _ => failed("the host returned no usable response", reqs.len()),
    }
}

/// Test hook for the PC: answers requests from fixtures. Only the last one set is active.
#[cfg(not(target_arch = "wasm32"))]
pub type MockFn = Box<dyn Fn(&HttpRequest) -> std::result::Result<HttpResponse, String>>;

#[cfg(not(target_arch = "wasm32"))]
thread_local! {
    static MOCK: std::cell::RefCell<Option<MockFn>> = const { std::cell::RefCell::new(None) };
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_mock_http(f: impl Fn(&HttpRequest) -> std::result::Result<HttpResponse, String> + 'static) {
    MOCK.with(|m| *m.borrow_mut() = Some(Box::new(f)));
}

#[cfg(not(target_arch = "wasm32"))]
fn transport(reqs: &[HttpRequest]) -> Vec<HttpResponse> {
    MOCK.with(|m| match m.borrow().as_ref() {
        None => failed("no network outside the Sankofa runtime (use set_mock_http in tests)", reqs.len()),
        Some(f) => reqs
            .iter()
            .map(|r| f(r).unwrap_or_else(|e| HttpResponse { error: Some(e), ..Default::default() }))
            .collect(),
    })
}

/// The value of the cookie `name` the app holds for `url` (a login the user did in the site's WebView), if any.
pub fn cookie(url: &str, name: &str) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        let packed = unsafe { host::cookie(url.as_ptr(), url.len() as i32, name.as_ptr(), name.len() as i32) };
        let bytes = unsafe { take_guest_bytes(packed) }?;
        String::from_utf8(bytes).ok().filter(|v| !v.is_empty())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (url, name);
        None
    }
}

/// Milliseconds since the Unix epoch (WebAssembly has no clock of its own).
pub fn now_ms() -> i64 {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        host::now_ms()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
    }
}

/// Writes a line to the host log.
pub fn log(message: &str) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        host::log(message.as_ptr(), message.len() as i32);
    }
    #[cfg(not(target_arch = "wasm32"))]
    eprintln!("[source] {message}");
}

/// Compatibility with sources written for ABI v1.
pub fn http_get(url: &str, headers: &[(&str, &str)]) -> Result<HttpResponse> {
    let r = get(url).headers(headers).send()?;
    Ok(HttpResponse { status: r.status, headers: r.headers, body: r.body, body_b64: r.body_b64, url: r.url, error: None })
}

pub fn http_get_json<T: for<'de> Deserialize<'de>>(url: &str, headers: &[(&str, &str)]) -> Result<T> {
    get(url).headers(headers).send()?.json()
}
