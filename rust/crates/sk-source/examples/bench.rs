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
    let mut s = WasmSource::load(&wasm, Arc::new(File(args[2].clone()))).unwrap();
    s.set_fuel_per_call(u64::MAX / 2);
    let t = std::time::Instant::now();
    let r = s.call(r#"{"op":"popular","page":1}"#).unwrap();
    println!("{} ms; {r}", t.elapsed().as_millis());
    println!("fuel used: {}", s.fuel_used());
}
