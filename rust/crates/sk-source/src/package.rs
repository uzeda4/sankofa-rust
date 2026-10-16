//! Signed source packages (`.sksource`): a WebAssembly module plus a manifest, signed with Ed25519.
//!
//! Layout: `"SKSRC1\0"`, `u32le` manifest length, manifest JSON, then the module bytes to the end.
//! The signature covers the fields that matter (name, language, version and the module's SHA-256), so
//! swapping the module or relabeling the package invalidates it. Trust is decided separately: the app keeps
//! the fingerprints of publishers the user chose to trust, like the extension signing-key flow.

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const MAGIC: &[u8; 7] = b"SKSRC1\0";
/// A manifest is a few hundred bytes; refuse absurd values before allocating.
const MAX_MANIFEST: usize = 64 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub name: String,
    pub lang: String,
    pub version: u32,
    /// Display name of the publisher; informational, the fingerprint is what identifies them.
    pub publisher: String,
    /// Ed25519 public key, hex.
    pub public_key: String,
    /// SHA-256 of the module, hex.
    pub wasm_sha256: String,
    /// Ed25519 signature of [`signed_message`], hex.
    pub signature: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PackageError {
    Malformed(&'static str),
    HashMismatch,
    BadSignature,
}

impl std::fmt::Display for PackageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Malformed(why) => write!(f, "not a valid source package: {why}"),
            Self::HashMismatch => f.write_str("the module does not match the manifest (corrupted or tampered)"),
            Self::BadSignature => f.write_str("the signature is invalid (tampered, or signed by another key)"),
        }
    }
}

impl std::error::Error for PackageError {}

pub struct Package {
    pub manifest: Manifest,
    pub wasm: Vec<u8>,
}

/// Exactly what is signed. Fixed text format, so there is no JSON canonicalization to get wrong.
pub fn signed_message(m: &Manifest) -> String {
    format!(
        "sankofa-source-v1\nname={}\nlang={}\nversion={}\nsha256={}\n",
        m.name, m.lang, m.version, m.wasm_sha256
    )
}

/// SHA-256 of the raw public key, as 32 hex characters (first 16 bytes): short enough to compare by eye.
pub fn fingerprint(public_key_hex: &str) -> String {
    let key = from_hex(public_key_hex).unwrap_or_default();
    to_hex(&Sha256::digest(&key)[..16])
}

/// Builds and signs a package. Used by the `sk-pack` tool and by tests.
pub fn build(seed: &[u8; 32], name: &str, lang: &str, version: u32, publisher: &str, wasm: &[u8]) -> Vec<u8> {
    let key = SigningKey::from_bytes(seed);
    let mut manifest = Manifest {
        name: name.into(),
        lang: lang.into(),
        version,
        publisher: publisher.into(),
        public_key: to_hex(key.verifying_key().as_bytes()),
        wasm_sha256: to_hex(&Sha256::digest(wasm)),
        signature: String::new(),
    };
    manifest.signature = to_hex(&key.sign(signed_message(&manifest).as_bytes()).to_bytes());
    encode(&manifest, wasm)
}

fn encode(manifest: &Manifest, wasm: &[u8]) -> Vec<u8> {
    let json = serde_json::to_vec(manifest).expect("manifest serializes");
    let mut out = Vec::with_capacity(MAGIC.len() + 4 + json.len() + wasm.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&(json.len() as u32).to_le_bytes());
    out.extend_from_slice(&json);
    out.extend_from_slice(wasm);
    out
}

/// Parses a package and checks the module hash and the signature. It does **not** decide whether the
/// publisher is trusted; compare [`fingerprint`] with the user's trusted list for that.
pub fn decode_and_verify(bytes: &[u8]) -> Result<Package, PackageError> {
    let rest = bytes.strip_prefix(MAGIC.as_slice()).ok_or(PackageError::Malformed("missing header"))?;
    let (len, rest) = rest.split_first_chunk::<4>().ok_or(PackageError::Malformed("truncated"))?;
    let len = u32::from_le_bytes(*len) as usize;
    if len > MAX_MANIFEST || len > rest.len() {
        return Err(PackageError::Malformed("bad manifest length"));
    }
    let (json, wasm) = rest.split_at(len);
    let manifest: Manifest = serde_json::from_slice(json).map_err(|_| PackageError::Malformed("bad manifest"))?;

    if to_hex(&Sha256::digest(wasm)) != manifest.wasm_sha256.to_ascii_lowercase() {
        return Err(PackageError::HashMismatch);
    }
    let key: [u8; 32] = from_hex(&manifest.public_key)
        .and_then(|b| b.try_into().ok())
        .ok_or(PackageError::Malformed("bad public key"))?;
    let sig: [u8; 64] = from_hex(&manifest.signature)
        .and_then(|b| b.try_into().ok())
        .ok_or(PackageError::Malformed("bad signature encoding"))?;
    VerifyingKey::from_bytes(&key)
        .and_then(|k| k.verify_strict(signed_message(&manifest).as_bytes(), &Signature::from_bytes(&sig)))
        .map_err(|_| PackageError::BadSignature)?;
    Ok(Package { manifest, wasm: wasm.to_vec() })
}

pub fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn from_hex(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 || !s.is_ascii() {
        return None;
    }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEED: [u8; 32] = [7; 32];
    const OTHER: [u8; 32] = [9; 32];
    const MODULE: &[u8] = b"\0asm-pretend-module";

    fn package() -> Vec<u8> {
        build(&SEED, "Demo", "en", 3, "Someone", MODULE)
    }

    #[test]
    fn a_signed_package_round_trips() {
        let p = decode_and_verify(&package()).unwrap();
        assert_eq!((p.manifest.name.as_str(), p.manifest.version), ("Demo", 3));
        assert_eq!(p.wasm, MODULE);
        assert_eq!(fingerprint(&p.manifest.public_key).len(), 32);
    }

    #[test]
    fn a_swapped_module_is_caught_by_the_hash() {
        let mut bytes = package();
        *bytes.last_mut().unwrap() ^= 1;
        assert_eq!(decode_and_verify(&bytes).err(), Some(PackageError::HashMismatch));
    }

    #[test]
    fn a_relabeled_package_is_caught_by_the_signature() {
        let p = decode_and_verify(&package()).unwrap();
        let mut relabeled = p.manifest.clone();
        relabeled.name = "Evil".into();
        assert_eq!(decode_and_verify(&encode(&relabeled, MODULE)).err(), Some(PackageError::BadSignature));
        relabeled = p.manifest;
        relabeled.version = 4;
        assert_eq!(decode_and_verify(&encode(&relabeled, MODULE)).err(), Some(PackageError::BadSignature));
    }

    #[test]
    fn a_signature_from_another_key_is_rejected() {
        let a = decode_and_verify(&package()).unwrap().manifest;
        let b = decode_and_verify(&build(&OTHER, "Demo", "en", 3, "Someone", MODULE)).unwrap().manifest;
        assert_ne!(fingerprint(&a.public_key), fingerprint(&b.public_key));
        // Claim the first publisher's key, but carry the second one's signature.
        let mut forged = a;
        forged.signature = b.signature;
        assert_eq!(decode_and_verify(&encode(&forged, MODULE)).err(), Some(PackageError::BadSignature));
    }

    #[test]
    fn garbage_and_truncation_are_malformed_not_panics() {
        for bytes in [&b""[..], b"SKSRC1\0", b"SKSRC1\0\xff\xff\xff\xff", b"not a package at all"] {
            assert!(matches!(decode_and_verify(bytes), Err(PackageError::Malformed(_))), "{bytes:?}");
        }
        assert!(decode_and_verify(&package()[..20]).is_err());
    }
}
