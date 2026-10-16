//! `sk-pack`: creates keys and signed `.sksource` packages for Sankofa Rust sources.
//!
//! ```text
//! sk-pack keygen <key-file>
//! sk-pack pack --key <key-file> --wasm <module.wasm> --name <name> --lang <lang> --version <n> \
//!              --publisher <name> --out <file.sksource>
//! sk-pack inspect <file.sksource>
//! ```
//! The key file holds the 32-byte Ed25519 seed as hex. Keep it out of the repository: whoever has it can
//! publish sources your users trust.

use sk_source::package::{build, decode_and_verify, fingerprint, from_hex, to_hex};
use std::{collections::HashMap, fs, path::Path, process::ExitCode};

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(msg) => {
            println!("{msg}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<String>) -> Result<String, String> {
    match args.first().map(String::as_str) {
        Some("keygen") => keygen(args.get(1).ok_or("usage: sk-pack keygen <key-file>")?),
        Some("pack") => pack(&flags(&args[1..])?),
        Some("inspect") => inspect(args.get(1).ok_or("usage: sk-pack inspect <file.sksource>")?),
        _ => Err("usage: sk-pack keygen | pack | inspect (see the crate docs)".into()),
    }
}

fn flags(args: &[String]) -> Result<HashMap<&str, &str>, String> {
    args.chunks(2)
        .map(|c| match c {
            [k, v] if k.starts_with("--") => Ok((&k[2..], v.as_str())),
            _ => Err(format!("expected `--flag value`, got {c:?}")),
        })
        .collect()
}

fn keygen(path: &str) -> Result<String, String> {
    if Path::new(path).exists() {
        return Err(format!("{path} already exists; refusing to overwrite a key"));
    }
    let mut seed = [0u8; 32];
    getrandom::fill(&mut seed).map_err(|e| e.to_string())?;
    fs::write(path, to_hex(&seed)).map_err(|e| e.to_string())?;
    let pubkey = pubkey_of(&seed);
    Ok(format!("key written to {path}\npublic key  {pubkey}\nfingerprint {}", fingerprint(&pubkey)))
}

fn pubkey_of(seed: &[u8; 32]) -> String {
    // A package signed with this seed carries the public key; build an empty one and read it back.
    let pkg = decode_and_verify(&build(seed, "-", "-", 0, "-", b"")).expect("fresh package verifies");
    pkg.manifest.public_key
}

fn pack(f: &HashMap<&str, &str>) -> Result<String, String> {
    let get = |k: &str| f.get(k).copied().ok_or(format!("missing --{k}"));
    let seed: [u8; 32] = from_hex(fs::read_to_string(get("key")?).map_err(|e| e.to_string())?.trim())
        .and_then(|b| b.try_into().ok())
        .ok_or("the key file is not a 32-byte hex seed")?;
    let wasm = fs::read(get("wasm")?).map_err(|e| e.to_string())?;
    let bytes = build(
        &seed,
        get("name")?,
        get("lang")?,
        get("version")?.parse().map_err(|_| "--version must be a number")?,
        get("publisher")?,
        &wasm,
    );
    fs::write(get("out")?, &bytes).map_err(|e| e.to_string())?;
    Ok(format!("wrote {} ({} bytes)\nfingerprint {}", get("out")?, bytes.len(), fingerprint(&pubkey_of(&seed))))
}

fn inspect(path: &str) -> Result<String, String> {
    let pkg = decode_and_verify(&fs::read(path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let m = &pkg.manifest;
    Ok(format!(
        "{} v{} ({})\npublisher   {}\nfingerprint {}\nmodule      {} bytes, sha256 {}\nsignature   valid",
        m.name,
        m.version,
        m.lang,
        m.publisher,
        fingerprint(&m.public_key),
        pkg.wasm.len(),
        m.wasm_sha256
    ))
}
