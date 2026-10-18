//! HentaiFox, written in Rust. Port of the Keiyoushi extension `all.hentaifox` (GalleryAdults): lists are
//! `/language/<name>/popular/pag/<n>/` pages, a title is one gallery with a single chapter, and its pictures come from
//! the `$.parseJSON('{"1":"j,w,h",...}')` script of the gallery page. One copy per language (en, ja, zh, ko, all).
//! Not ported: tag/category filters, the favorites list and the "short title" setting.

use fancy_regex::Regex;
use sk_source_sdk::html::{Document, El};
use sk_source_sdk::*;

const BASE: &str = "https://hentaifox.com";

/// The site's name for the language of this copy; empty (no language) for `all`.
fn language() -> &'static str {
    match lang().as_str() {
        "en" => "english",
        "ja" => "japanese",
        "zh" => "chinese",
        "ko" => "korean",
        _ => "",
    }
}

/// The code the site puts in `data-languages` for the language (1 = English).
fn language_code() -> Option<&'static str> {
    match lang().as_str() {
        "en" => Some("1"),
        "ja" => Some("5"),
        "zh" => Some("6"),
        "ko" => Some("11"),
        _ => None,
    }
}

/// `/pag/<n>/` after a path, `&page=<n>` after a query, and the home page's own `page/2/`.
fn add_page(url: &str, page: u32) -> String {
    if url == format!("{BASE}/") && page == 2 {
        format!("{url}page/{page}/")
    } else if url.contains('?') {
        format!("{url}&page={page}")
    } else {
        format!("{}/pag/{page}/", url.trim_end_matches('/'))
    }
}

fn thumb_of(el: &El) -> Option<String> {
    ["data-cfsrc", "data-src", "data-lazy-src"].iter().map(|a| el.abs_url(a)).find(|u| !u.is_empty()).or_else(|| Some(el.abs_url("src")).filter(|u| !u.is_empty()))
}

fn path_of(abs: &str) -> String {
    match Url::parse(abs) {
        Ok(u) => u.path().to_string(),
        Err(_) => abs.to_string(),
    }
}

fn list(doc: &Document, only_language: bool) -> MangaPage {
    let code = language_code();
    let mangas = doc
        .select("div.thumb")
        .iter()
        .filter(|el| {
            // a result in another language is left out (a list with no language data at all is kept as it is)
            let langs = el.attr("data-languages");
            !only_language || code.is_none() || langs.trim().is_empty() || langs.split(' ').any(|l| Some(l) == code)
        })
        .filter_map(|el| {
            let link = el.select_first(".inner_thumb a")?;
            Some(Manga {
                url: path_of(&link.abs_url("href")),
                title: el.select_first(".caption").map(|c| c.text())?,
                thumbnail_url: el.select_first(".inner_thumb img").and_then(|i| thumb_of(&i)),
                ..Default::default()
            })
        })
        .collect();
    MangaPage { mangas, has_next: doc.select_first(".pagination li.active + li:not(.disabled), a[rel='next']").is_some() }
}

fn fetch(url: &str) -> Result<Document> {
    Ok(get(url).send()?.html())
}

/// `school-girl` becomes `school+girl`: a word after a special character is a separate term.
fn query_string(query: &str) -> String {
    let special = Regex::new(r#"[^a-zA-Z0-9"]+(?=[a-zA-Z0-9"])"#).unwrap();
    [query, language()]
        .iter()
        .filter(|s| !s.trim().is_empty())
        .map(|s| special.replace_all(s.trim(), "+").to_string())
        .collect::<Vec<_>>()
        .join("+")
}

/// The sorted names of every link under `ul.<tag>` (each with its translation in `.split_tag`).
fn info(doc: &Document, tag: &str) -> String {
    let mut names: Vec<String> = Vec::new();
    for a in doc.select(&format!("ul.{} a", tag.to_lowercase())).iter() {
        names.push(a.own_text().trim().to_string());
        names.push(a.select(".split_tag").text().trim_start_matches("| ").trim().to_string());
    }
    names.retain(|n| !n.is_empty());
    names.sort();
    names.join(", ")
}

fn input(doc: &Document, id: &str) -> String {
    doc.select(&format!("input[id={id}]")).attr("value")
}

/// "3 weeks ago" -> milliseconds; 0 when it is not in that form.
fn posted(text: &str) -> i64 {
    let t = text.trim().trim_start_matches("Posted:").trim().to_lowercase();
    let mut words = t.split_whitespace();
    let (Some(n), Some(unit)) = (words.next().and_then(|n| n.parse::<i64>().ok()), words.next()) else { return 0 };
    let unit = match unit.trim_end_matches('s') {
        "second" => Unit::Second,
        "minute" => Unit::Minute,
        "hour" => Unit::Hour,
        "day" => Unit::Day,
        "week" => Unit::Week,
        "month" => Unit::Month,
        "year" => Unit::Year,
        _ => return 0,
    };
    minus(now_ms(), n, unit)
}

fn details_of(doc: &Document, url: &str) -> Result<Manga> {
    let top = doc.select_first(".gallery_top").ok_or("the page has no gallery (the site changed?)")?;
    let mut parts: Vec<String> = ["Parodies", "Characters", "Groups", "Languages", "Categories", "Category"]
        .iter()
        .filter_map(|tag| Some(info(doc, tag)).filter(|v| !v.is_empty()).map(|v| format!("**{tag}**: {v}")))
        .collect();
    let pages = input(doc, "load_pages");
    if !pages.is_empty() {
        parts.push(format!("**Pages**: {pages}"));
    }
    if let Some(alt) = top.select_first("h1 + h2, .subtitle").map(|a| a.own_text()).filter(|a| !a.trim().is_empty()) {
        parts.push(format!("**Alternative title**: {}", alt.trim()));
    }
    let artists = info(doc, "Artists");
    Ok(Manga {
        url: url.to_string(),
        title: top.select_first("h1").map(|h| h.text()).unwrap_or_default(),
        thumbnail_url: top.select_first(".cover img").and_then(|i| thumb_of(&i)),
        genre: info(doc, "Tags").split(", ").filter(|g| !g.is_empty()).map(str::to_string).collect(),
        author: Some(if artists.is_empty() { info(doc, "Groups") } else { artists }).filter(|a| !a.is_empty()),
        description: Some(parts.join("\n\n")).filter(|d| !d.is_empty()),
        status: 2,
        ..Default::default()
    })
}

fn chapter_of(doc: &Document, url: &str) -> Vec<Chapter> {
    let groups = info(doc, "Groups");
    vec![Chapter {
        url: url.to_string(),
        name: "Chapter".into(),
        chapter_number: -1.0,
        scanlator: Some(groups).filter(|g| !g.is_empty()),
        date_upload: doc.select_first(".pages:contains(Posted:)").map(|p| posted(&p.own_text())).unwrap_or(0),
        memo: None,
    }]
}

#[derive(Default)]
pub struct HentaiFox;

impl Source for HentaiFox {
    fn info(&self) -> Info {
        Info { name: "HentaiFox".into(), lang: lang(), base_url: BASE.into(), supports_latest: !language().is_empty(), ..Default::default() }
    }

    fn popular(&self, page: u32) -> Result<MangaPage> {
        let mut url = format!("{BASE}/");
        if !language().is_empty() {
            url.push_str(&format!("language/{}/popular", language()));
        }
        Ok(list(&fetch(&add_page(&url, page))?, false))
    }

    fn latest(&self, page: u32) -> Result<MangaPage> {
        let mut url = format!("{BASE}/");
        if !language().is_empty() {
            url.push_str(&format!("language/{}", language()));
        }
        Ok(list(&fetch(&add_page(&url, page))?, false))
    }

    fn search(&self, page: u32, query: &str, _filters: &Filters) -> Result<MangaPage> {
        // a number is a gallery's own id
        if !query.trim().is_empty() && query.trim().chars().all(|c| c.is_ascii_digit()) {
            let url = format!("/gallery/{}/", query.trim());
            let doc = fetch(&format!("{BASE}{url}"))?;
            return Ok(MangaPage { mangas: vec![details_of(&doc, &url)?], has_next: false });
        }
        let url = format!("{BASE}/search/?q={}&sort=popular", query_string(query));
        Ok(list(&fetch(&add_page(&url, page))?, true))
    }

    fn details(&self, manga: &Manga) -> Result<Manga> {
        details_of(&fetch(&format!("{BASE}{}", manga.url))?, &manga.url)
    }

    fn chapters(&self, manga: &Manga) -> Result<Vec<Chapter>> {
        Ok(chapter_of(&fetch(&format!("{BASE}{}", manga.url))?, &manga.url))
    }

    fn update(&self, manga: &Manga, fetch_details: bool, fetch_chapters: bool) -> Result<Update> {
        // a gallery is one page: details and its single chapter come from the same answer
        let doc = fetch(&format!("{BASE}{}", manga.url))?;
        Ok(Update {
            manga: if fetch_details { Some(details_of(&doc, &manga.url)?) } else { None },
            chapters: fetch_chapters.then(|| chapter_of(&doc, &manga.url)),
        })
    }

    fn pages(&self, chapter: &Chapter) -> Result<Vec<Page>> {
        let doc = fetch(&format!("{BASE}{}", chapter.url))?;
        let html = doc.html();
        let json = html.split_once("$.parseJSON('").and_then(|(_, rest)| rest.split_once("');")).map(|(j, _)| j.trim().to_string());
        let (dir, id, gid) = (input(&doc, "load_dir"), input(&doc, "load_id"), input(&doc, "gallery_id"));
        let server = doc.select_first(".cover img").and_then(|i| thumb_of(&i)).and_then(|u| Url::parse(&u).ok()).and_then(|u| u.host_str().map(str::to_string)).unwrap_or_default();
        let json: Value = serde_json::from_str(&json.ok_or("the gallery has no page list (the site changed?)")?).map_err(|e| format!("bad page list: {e}"))?;
        let mut pages: Vec<(u32, String)> = json
            .as_object()
            .ok_or("bad page list")?
            .iter()
            .filter_map(|(k, v)| {
                let n: u32 = k.parse().ok()?;
                let ext = match v.as_str()?.split(',').next()? {
                    "p" => "png",
                    "b" => "bmp",
                    "g" => "gif",
                    "w" => "webp",
                    _ => "jpg",
                };
                Some((n, format!("https://{server}/{dir}/{id}/{n}.{ext}")))
            })
            .collect();
        pages.sort_by_key(|p| p.0);
        Ok(pages
            .into_iter()
            .map(|(n, image)| Page { index: n, url: format!("{BASE}/g/{gid}/{n}/"), image_url: image, headers: Vec::new(), ops: Vec::new(), mirrors: Vec::new() })
            .collect())
    }
}

export_source!(HentaiFox);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_numbers_follow_the_sites_three_forms() {
        assert_eq!(add_page("https://hentaifox.com/language/english/popular", 3), "https://hentaifox.com/language/english/popular/pag/3/");
        assert_eq!(add_page("https://hentaifox.com/search/?q=a", 2), "https://hentaifox.com/search/?q=a&page=2");
        assert_eq!(add_page("https://hentaifox.com/", 2), "https://hentaifox.com/page/2/");
    }

    #[test]
    fn a_word_after_a_special_character_becomes_its_own_term() {
        set_context("en", &[]);
        assert_eq!(query_string("school-girl"), "school+girl+english");
    }
}
