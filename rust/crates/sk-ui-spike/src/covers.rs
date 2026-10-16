//! Cover discovery for the spike: pulls cover image URLs out of a source's HTML listing.

/// Unique absolute http(s) image URLs (`jpg`, `jpeg`, `png`, `webp`) found in `src`/`data-src`
/// attributes, in page order, up to `limit`.
///
/// ponytail: attribute scan, not an HTML parser; good enough for a spike, the real source
/// contract returns structured data instead.
pub fn extract_image_urls(html: &str, limit: usize) -> Vec<String> {
    let mut urls: Vec<String> = Vec::new();
    for attr in ["src=\"", "data-src=\""] {
        let mut rest = html;
        while let Some(i) = rest.find(attr) {
            rest = &rest[i + attr.len()..];
            let Some(end) = rest.find('"') else { break };
            let url = &rest[..end];
            let lower = url.to_ascii_lowercase();
            let is_image = ["jpg", "jpeg", "png", "webp"].iter().any(|e| lower.ends_with(&format!(".{e}")));
            if is_image && lower.starts_with("http") && !urls.iter().any(|u| u == url) {
                urls.push(url.to_string());
            }
        }
    }
    urls.truncate(limit);
    urls
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_unique_absolute_images_and_skips_the_rest() {
        let html = r#"<img src="/logo.svg"><img src="https://a/x.webp" loading="lazy">
            <img src="https://a/x.webp"><img data-src="http://b/y.JPG"><img src="https://c/z.gif">"#;
        assert_eq!(extract_image_urls(html, 10), ["https://a/x.webp", "http://b/y.JPG"]);
        assert_eq!(extract_image_urls(html, 1), ["https://a/x.webp"]);
    }
}
