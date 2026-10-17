//! Mangakakalot, written in Rust. A MangaBox site: everything comes from `sk-theme-mangabox`; this file only says
//! what is different here. Port of the Kotlin extension `en.mangakakalot` (same source id, so a library carries over).

use sk_source_sdk::{export_source, Manga, RateLimit};
use sk_theme_mangabox::{Site, Theme};

pub struct Mangakakalot;

impl Theme for Mangakakalot {
    const NAME: &'static str = "Mangakakalot";
    const BASE_URL: &'static str = "https://www.mangakakalot.gg";
    // The extension's id is md5("mangakakalot/en/0"), not version 1.
    const VERSION_ID: u32 = 0;
    const MIRRORS: &'static [&'static str] = &["https://www.mangakakalot.gg", "https://www.mangakakalove.com"];
    const RATE_LIMIT: Option<RateLimit> = Some(RateLimit { permits: 2, period_ms: 1000 });

    /// Titles listed with an id (`ab123`) are addressed by their name instead.
    fn compute_slug(manga: &Manga) -> String {
        let slug = manga.url.rsplit('/').next().unwrap_or_default().to_string();
        if is_id(&slug) && !manga.title.is_empty() {
            title_to_slug(&manga.title)
        } else {
            slug
        }
    }
}

/// Two lowercase letters and digits.
fn is_id(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() > 2 && b[..2].iter().all(u8::is_ascii_lowercase) && b[2..].iter().all(u8::is_ascii_digit)
}

fn title_to_slug(title: &str) -> String {
    let kept: String = title
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c.is_whitespace() || *c == '-')
        .collect();
    let mut out = String::new();
    let mut last_sep = false;
    for c in kept.trim().chars() {
        if c.is_whitespace() || c == '-' {
            if !last_sep {
                out.push('-');
            }
            last_sep = true;
        } else {
            out.push(c);
            last_sep = false;
        }
    }
    out.trim_matches('-').to_string()
}

export_source!(Site<Mangakakalot>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_id_is_the_one_the_extension_had_on_the_device() {
        assert_eq!(sk_source::extension_id(Mangakakalot::NAME, Mangakakalot::LANG, Mangakakalot::VERSION_ID), 160_963_523_964_560_052);
    }

    #[test]
    fn id_like_slugs_become_title_slugs() {
        let m = |url: &str, title: &str| Manga { url: url.into(), title: title.into(), ..Default::default() };
        assert_eq!(Mangakakalot::compute_slug(&m("/manga/ab123", "Solo Leveling: Ragnarok!")), "solo-leveling-ragnarok");
        assert_eq!(Mangakakalot::compute_slug(&m("/manga/one-piece", "One Piece")), "one-piece");
        assert_eq!(Mangakakalot::compute_slug(&m("/manga/ab123", "")), "ab123", "no title yet: keep the id");
        assert!(is_id("ab123") && !is_id("abc123") && !is_id("ab") && !is_id("a1"));
    }
}
