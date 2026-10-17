//! Image sizes from the first bytes of a file, so a source can plan edits (merging strips) without downloading
//! whole pages: ask for `range(0, 2047)` and pass the bytes here.

/// `(width, height)` of a PNG, JPEG, WebP or GIF, or `None` when the bytes are too short or not one of those.
pub fn image_size(b: &[u8]) -> Option<(u32, u32)> {
    if b.starts_with(&[0x89, b'P', b'N', b'G']) {
        return Some((be32(b, 16)?, be32(b, 20)?));
    }
    if b.starts_with(b"GIF8") {
        return Some((le16(b, 6)? as u32, le16(b, 8)? as u32));
    }
    if b.starts_with(&[0xFF, 0xD8]) {
        return jpeg_size(b);
    }
    if b.len() >= 16 && &b[0..4] == b"RIFF" && &b[8..12] == b"WEBP" {
        return webp_size(b);
    }
    None
}

fn le16(b: &[u8], i: usize) -> Option<u16> {
    Some(u16::from_le_bytes([*b.get(i)?, *b.get(i + 1)?]))
}

fn be16(b: &[u8], i: usize) -> Option<u16> {
    Some(u16::from_be_bytes([*b.get(i)?, *b.get(i + 1)?]))
}

fn be32(b: &[u8], i: usize) -> Option<u32> {
    Some(u32::from_be_bytes([*b.get(i)?, *b.get(i + 1)?, *b.get(i + 2)?, *b.get(i + 3)?]))
}

fn le24(b: &[u8], i: usize) -> Option<u32> {
    Some(*b.get(i)? as u32 | (*b.get(i + 1)? as u32) << 8 | (*b.get(i + 2)? as u32) << 16)
}

/// Walks the markers until a start-of-frame one.
fn jpeg_size(b: &[u8]) -> Option<(u32, u32)> {
    let mut i = 2;
    while i + 9 < b.len() {
        if b[i] != 0xFF {
            i += 1;
            continue;
        }
        let marker = b[i + 1];
        if marker == 0xFF {
            i += 1;
            continue;
        }
        // SOF0..SOF15 except DHT (C4), JPG (C8) and DAC (CC) carry the size.
        if (0xC0..=0xCF).contains(&marker) && !matches!(marker, 0xC4 | 0xC8 | 0xCC) {
            return Some((be16(b, i + 7)? as u32, be16(b, i + 5)? as u32));
        }
        i += 2 + be16(b, i + 2)? as usize;
    }
    None
}

fn webp_size(b: &[u8]) -> Option<(u32, u32)> {
    match b.get(12..16)? {
        // lossy
        b"VP8 " => Some(((le16(b, 26)? & 0x3FFF) as u32, (le16(b, 28)? & 0x3FFF) as u32)),
        // lossless: 14 bits each, minus-one coded
        b"VP8L" => {
            let bits = u32::from_le_bytes([*b.get(21)?, *b.get(22)?, *b.get(23)?, *b.get(24)?]);
            Some(((bits & 0x3FFF) + 1, ((bits >> 14) & 0x3FFF) + 1))
        }
        // extended: 24 bits each, minus-one coded
        b"VP8X" => Some((le24(b, 24)? + 1, le24(b, 27)? + 1)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_are_read_from_headers() {
        let mut png = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 13, b'I', b'H', b'D', b'R'];
        png.extend(800u32.to_be_bytes());
        png.extend(1200u32.to_be_bytes());
        assert_eq!(image_size(&png), Some((800, 1200)));

        // SOI, APP0 (16 bytes), SOF0 with height 1000 and width 720
        let mut jpg = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
        jpg.extend([0u8; 14]);
        jpg.extend([0xFF, 0xC0, 0x00, 0x11, 8, 0x03, 0xE8, 0x02, 0xD0, 3]);
        assert_eq!(image_size(&jpg), Some((720, 1000)));

        let mut vp8x = b"RIFF\0\0\0\0WEBPVP8X".to_vec();
        vp8x.extend([10, 0, 0, 0, 0, 0, 0, 0]); // chunk size + flags
        vp8x.extend([0x1F, 0x03, 0x00]); // width - 1 = 799
        vp8x.extend([0xFF, 0x04, 0x00]); // height - 1 = 1279
        assert_eq!(image_size(&vp8x), Some((800, 1280)));

        let mut vp8l = b"RIFF\0\0\0\0WEBPVP8L".to_vec();
        vp8l.extend([0, 0, 0, 0, 0x2F]);
        let bits: u32 = 639 | (1599 << 14);
        vp8l.extend(bits.to_le_bytes());
        assert_eq!(image_size(&vp8l), Some((640, 1600)));

        let mut vp8 = b"RIFF\0\0\0\0WEBPVP8 ".to_vec();
        vp8.extend([0; 10]); // chunk size, frame tag, start code
        vp8.extend(900u16.to_le_bytes());
        vp8.extend(1400u16.to_le_bytes());
        assert_eq!(image_size(&vp8), Some((900, 1400)));

        assert_eq!(image_size(b"GIF89a\x10\x00\x20\x00"), Some((16, 32)));
        assert_eq!(image_size(b"nope"), None);
        assert_eq!(image_size(&[0xFF, 0xD8]), None);
    }
}
