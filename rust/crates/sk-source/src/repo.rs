//! Signed source repositories (`index.skrepo`): a list of `.sksource` packages the app can offer to install.
//!
//! Layout: `"SKREPO1\0"`, `u32le` body length, body JSON, then the 64-byte Ed25519 signature. The signature
//! covers `"sankofa-repo-v1\n"` followed by the exact body bytes (no JSON canonicalization involved). The
//! repository's public key is inside the body and identifies it; the app asks the user to confirm its
//! fingerprint when adding the repository, and refuses an index whose `serial` is lower than the last one it
//! saw (so an old, vulnerable list cannot be replayed).
//!
//! The index only says what exists and where. Every package is still verified on its own (hash from the index
//! and the publisher signature) before it is installed.

use crate::package::{from_hex, to_hex};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};

const MAGIC: &[u8; 8] = b"SKREPO1\0";
const CONTEXT: &[u8] = b"sankofa-repo-v1\n";
const MAX_BODY: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub name: String,
    pub lang: String,
    pub version: u32,
    /// Absolute URL, or relative to the index URL.
    pub package: String,
    /// SHA-256 of the `.sksource` file, hex.
    pub sha256: String,
    pub size: u64,
    /// Absolute URL, or relative to the index URL, of the source's icon (a PNG). Older indexes have none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Index {
    pub name: String,
    /// Ed25519 public key of the repository, hex.
    pub public_key: String,
    /// Grows with every publication; used to refuse replays of older indexes.
    pub serial: u64,
    pub sources: Vec<Entry>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RepoError {
    Malformed(&'static str),
    BadSignature,
}

impl std::fmt::Display for RepoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Malformed(why) => write!(f, "not a valid repository index: {why}"),
            Self::BadSignature => f.write_str("the repository index signature is invalid"),
        }
    }
}

impl std::error::Error for RepoError {}

/// Builds and signs an index. `index.public_key` is overwritten with the key derived from `seed`.
pub fn build(seed: &[u8; 32], mut index: Index) -> Vec<u8> {
    let key = SigningKey::from_bytes(seed);
    index.public_key = to_hex(key.verifying_key().as_bytes());
    let body = serde_json::to_vec(&index).expect("index serializes");
    let signature = key.sign(&[CONTEXT, &body].concat()).to_bytes();
    let mut out = Vec::with_capacity(MAGIC.len() + 4 + body.len() + 64);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&(body.len() as u32).to_le_bytes());
    out.extend_from_slice(&body);
    out.extend_from_slice(&signature);
    out
}

pub fn decode_and_verify(bytes: &[u8]) -> Result<Index, RepoError> {
    let rest = bytes.strip_prefix(MAGIC.as_slice()).ok_or(RepoError::Malformed("missing header"))?;
    let (len, rest) = rest.split_first_chunk::<4>().ok_or(RepoError::Malformed("truncated"))?;
    let len = u32::from_le_bytes(*len) as usize;
    if len > MAX_BODY || rest.len() != len + 64 {
        return Err(RepoError::Malformed("bad length"));
    }
    let (body, sig) = rest.split_at(len);
    let index: Index = serde_json::from_slice(body).map_err(|_| RepoError::Malformed("bad body"))?;
    let key: [u8; 32] = from_hex(&index.public_key)
        .and_then(|b| b.try_into().ok())
        .ok_or(RepoError::Malformed("bad public key"))?;
    let sig: [u8; 64] = sig.try_into().map_err(|_| RepoError::Malformed("bad signature"))?;
    VerifyingKey::from_bytes(&key)
        .and_then(|k| k.verify_strict(&[CONTEXT, body].concat(), &Signature::from_bytes(&sig)))
        .map_err(|_| RepoError::BadSignature)?;
    Ok(index)
}

/// Resolves `entry.package` against the URL the index was downloaded from.
pub fn resolve_url(index_url: &str, package: &str) -> String {
    if package.starts_with("http://") || package.starts_with("https://") {
        return package.to_string();
    }
    match index_url.rsplit_once('/') {
        Some((base, _)) => format!("{base}/{}", package.trim_start_matches('/')),
        None => package.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEED: [u8; 32] = [3; 32];

    fn index(serial: u64) -> Index {
        Index {
            name: "Test".into(),
            public_key: String::new(),
            serial,
            sources: vec![Entry {
                name: "Demo".into(),
                lang: "en".into(),
                version: 2,
                package: "demo.sksource".into(),
                sha256: "ab".repeat(32),
                size: 10,
                icon: Some("icons/demo.png".into()),
            }],
        }
    }

    #[test]
    fn a_signed_index_round_trips() {
        let i = decode_and_verify(&build(&SEED, index(5))).unwrap();
        assert_eq!((i.serial, i.sources.len(), i.sources[0].version), (5, 1, 2));
        assert_eq!(i.public_key.len(), 64);
    }

    #[test]
    fn any_changed_byte_breaks_the_signature_or_the_format() {
        let bytes = build(&SEED, index(5));
        for at in [10, bytes.len() / 2, bytes.len() - 70, bytes.len() - 1] {
            let mut t = bytes.clone();
            t[at] ^= 1;
            assert!(decode_and_verify(&t).is_err(), "flipping byte {at} was not caught");
        }
    }

    #[test]
    fn a_body_signed_by_someone_else_is_rejected() {
        let a = build(&SEED, index(1));
        let b = build(&[4; 32], index(1));
        // Keep the first body (and its public key) but attach the other key's signature.
        let split = a.len() - 64;
        let forged = [&a[..split], &b[b.len() - 64..]].concat();
        assert_eq!(decode_and_verify(&forged), Err(RepoError::BadSignature));
    }

    #[test]
    fn garbage_is_malformed_not_a_panic() {
        for bytes in [&b""[..], b"SKREPO1\0", b"SKREPO1\0\xff\xff\xff\xff", b"nope"] {
            assert!(matches!(decode_and_verify(bytes), Err(RepoError::Malformed(_))));
        }
    }

    #[test]
    fn package_urls_resolve_against_the_index_url() {
        assert_eq!(resolve_url("https://h/x/index.skrepo", "a.sksource"), "https://h/x/a.sksource");
        assert_eq!(resolve_url("https://h/x/index.skrepo", "/a.sksource"), "https://h/x/a.sksource");
        assert_eq!(resolve_url("https://h/index.skrepo", "https://other/a.sksource"), "https://other/a.sksource");
    }
}
