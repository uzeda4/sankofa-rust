//! The request signature the site's API wants (`vrf=`): three byte-substitution stages, each with a 256-byte table, a
//! key and a start value, then URL-safe base64 without padding. Port of `VrfSigner.kt`.

use base64::Engine;

const TABLE_1: &str = "yINlmUNho8VYJT+ibTIP+9ESiULpVEtMOoD6U6lRE0R/xwXo/Xp9NrUgC4cw/Lmo33vUyjUE40kUoEWIr/fxfNNcq2s79ShQ5NhNrFnJ4hXPwOu/SuXzIbuTQKGFvfm08E9jvCfqAtoDqvQq3dVWPQFmJjgvkISBeXY3BgANR+yVnjGbcxZ47d6kLNfZPIayTq3/YGySb1KuVZodWp/WGNAO5pfMcpaK53Hhs0allBszaMaxuouOwdxbwgxIw6YunSsXjI05Yi0j9j4eHKfSXR8Ifo/Od+8iamRfCXTyvm7NGRGYdcQ0ywcK/u6RXhrbcCm4t2eCtrDgQVecJGkQ+A==";
const KEY_1: &str = "0Ec58JOY3uBzJK9m3zqIOpdlF7UFiax9DmA=";
const TABLE_2: &str = "IUFltCxD3Oc2cwCgkJffthaOg9cgPUb0LgW6H/VtfcF0kc5F25t+aWj6JH9VOhOaY0rAFdUxlDnl5BLNvwEJvQtP5qcw7vdb/K+chnbwnspSHT8mz5lqwz41TezG0hkO06FTjJZhsyNuFLDpD2ZZxQj/QIRcF90zpmQ7Byu483WsQqUE0C342HL+JXngRB6fRzxRyVTaKu83h7UYTJ0QMt6ixFh6S3F8gqkKwrGTL3jHNBsD45UnifK8+RGtishQV2K3rujLKEkiZxpr2dYcudFW4oFsDKhad3CLBvuyTqsCo4B7mL5IKQ1vXo/MOOvq1I1d8ar9X6Ttu5KF4fZgiA==";
const KEY_2: &str = "AAdjb1iPY8CiDmq9H34tKTBF8a3oDQ==";
const TABLE_3: &str = "NQHlu1/wVO5EmkwQymF810qqY2xG1k2obcas4Z9mCsPEIFl9pRIjFxbJ7ybMHbBckT5Ton85E0FOeHezbh/mjlEYpmpnlXOS8dgrqeq2KfxImTh1YK9y0PeMNhzA1OQzSY9brYOJq/l2QnE/hwOeZIhPixVSKIUlDb5vLcH6RWKxkIEMuP0bDwIqQ71AJJaEaMJL7A6YtyIwoRT+L5v4aZzodN/0+3nOGsfblFjgxSfPzVDjNFeNl5P26+kEC/8AHgdrpAbt3hHz3HrRN1Y6e+JHgF7ncFWnoF0y3THL1S71WgWGCa6KtSzTCCG58n68nTyj2T3Sshk7utqCtMi/ZQ==";
const KEY_3: &str = "DELOJgPsVaCcblDtTGMdHzM=";

/// `(table, key, iv)` of each stage, in order.
fn stages() -> Vec<(Vec<u8>, Vec<u8>, u8)> {
    let b64 = |s: &str| base64::engine::general_purpose::STANDARD.decode(s).expect("a valid built-in table");
    vec![
        (b64(TABLE_1), b64(KEY_1), 0x5A),
        (b64(TABLE_2), b64(KEY_2), 0x35),
        (b64(TABLE_3), b64(KEY_3), 0xBA),
    ]
}

fn stage(data: &[u8], table: &[u8], key: &[u8], iv: u8) -> Vec<u8> {
    let mut prev = iv;
    data.iter()
        .enumerate()
        .map(|(i, b)| {
            prev = table[(b ^ key[i % key.len()] ^ prev) as usize];
            prev
        })
        .collect()
}

/// The signature of `path` (the request path without `/api`, with the sorted query).
pub fn sign(path: &str) -> String {
    let mut data = path.as_bytes().to_vec();
    for (table, key, iv) in stages() {
        data = stage(&data, &table, &key, iv);
    }
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tables_decode_to_256_bytes_and_signing_is_deterministic() {
        for (t, k, _) in stages() {
            assert_eq!(t.len(), 256);
            assert!(!k.is_empty());
        }
        let a = sign("/titles?limit=50&page=1");
        assert_eq!(a, sign("/titles?limit=50&page=1"));
        assert_ne!(a, sign("/titles?limit=50&page=2"));
        assert!(!a.contains(['+', '/', '=']), "{a}");
    }
}
