use sk_source::{Http, HttpReply, HttpRequest, WasmSource};
use std::sync::Arc;
struct File(String);
impl Http for File {
    fn request(&self, _: &HttpRequest) -> HttpReply {
        match std::fs::read_to_string(&self.0) {
            Ok(body) => HttpReply { status: 200, body, ..Default::default() },
            Err(e) => HttpReply::failed(e.to_string()),
        }
    }
}
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let wasm = std::fs::read(&args[1]).unwrap();
    // `live` as the second argument runs against the real sites through curl (needs `--features testing`)
    #[cfg(feature = "testing")]
    let http: Arc<dyn Http> = if args[2] == "live" { Arc::new(sk_source::testing::Curl::default()) } else { Arc::new(File(args[2].clone())) };
    #[cfg(not(feature = "testing"))]
    let http: Arc<dyn Http> = Arc::new(File(args[2].clone()));
    let mut s = WasmSource::load(&wasm, http).unwrap();
    s.set_fuel_per_call(u64::MAX / 2);
    let t = std::time::Instant::now();
    // a context (language, settings) can be sent with every call: the fourth argument is the request, the fifth the `ctx`
    let op = args.get(3).map(String::as_str).unwrap_or(r#"{"op":"popular","page":1}"#);
    let r = s.call(op).unwrap();
    println!("{} ms; {}", t.elapsed().as_millis(), &r[..r.len().min(300)]);
    println!("fuel used: {}", s.fuel_used());
}
