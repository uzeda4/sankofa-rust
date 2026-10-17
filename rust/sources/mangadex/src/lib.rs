//! MangaDex as a Sankofa source written in Rust: the reference for what a Rust source looks like.
//! Uses the public API (https://api.mangadex.org); safe and suggestive content only, English chapters.

use serde_json::Value;
use sk_source_sdk::{
    export_source, http_get_json, url_encode, Chapter, Filters, Info, Manga, MangaPage, Page, RateLimit, Result, Source,
};

const API: &str = "https://api.mangadex.org";
const COVERS: &str = "https://uploads.mangadex.org/covers";
const PAGE_SIZE: u32 = 20;
const RATINGS: &str = "&contentRating[]=safe&contentRating[]=suggestive";
const HEADERS: &[(&str, &str)] = &[("Accept", "application/json")];

#[derive(Default)]
pub struct MangaDex;

impl Source for MangaDex {
    fn info(&self) -> Info {
        Info {
            name: "MangaDex (Rust)".into(),
            lang: "en".into(),
            base_url: "https://mangadex.org".into(),
            supports_latest: true,
            rate_limit: Some(RateLimit { permits: 4, period_ms: 1000 }),
            ..Default::default()
        }
    }

    fn popular(&self, page: u32) -> Result<MangaPage> {
        list(page, "&order[followedCount]=desc", "")
    }

    fn latest(&self, page: u32) -> Result<MangaPage> {
        list(page, "&order[latestUploadedChapter]=desc", "")
    }

    fn search(&self, page: u32, query: &str, _filters: &Filters) -> Result<MangaPage> {
        list(page, "&order[relevance]=desc", &format!("&title={}", url_encode(query)))
    }

    fn details(&self, manga: &Manga) -> Result<Manga> {
        let id = id_of(&manga.url)?;
        let json: Value =
            http_get_json(&format!("{API}/manga/{id}?includes[]=cover_art&includes[]=author&includes[]=artist"), HEADERS)?;
        Ok(to_manga(&json["data"]))
    }

    fn chapters(&self, manga: &Manga) -> Result<Vec<Chapter>> {
        let id = id_of(&manga.url)?;
        // ponytail: first 500 English chapters; page through `offset` for longer series.
        let json: Value = http_get_json(
            &format!(
                "{API}/manga/{id}/feed?translatedLanguage[]=en&limit=500&order[chapter]=desc&includes[]=scanlation_group{RATINGS}"
            ),
            HEADERS,
        )?;
        Ok(json["data"]
            .as_array()
            .into_iter()
            .flatten()
            // Chapters hosted elsewhere have no pages on MangaDex.
            .filter(|c| c["attributes"]["externalUrl"].is_null() && c["attributes"]["pages"].as_u64() != Some(0))
            .map(|c| {
                let a = &c["attributes"];
                let number = a["chapter"].as_str().and_then(|n| n.parse::<f32>().ok()).unwrap_or(0.0);
                let title = a["title"].as_str().filter(|t| !t.is_empty());
                Chapter {
                    url: format!("/chapter/{}", c["id"].as_str().unwrap_or_default()),
                    name: match (a["chapter"].as_str(), title) {
                        (Some(n), Some(t)) => format!("Chapter {n} - {t}"),
                        (Some(n), None) => format!("Chapter {n}"),
                        (None, Some(t)) => t.to_string(),
                        (None, None) => "Oneshot".into(),
                    },
                    chapter_number: number,
                    scanlator: c["relationships"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .find(|r| r["type"] == "scanlation_group")
                        .and_then(|r| r["attributes"]["name"].as_str())
                        .map(str::to_string),
                    date_upload: a["publishAt"].as_str().map(iso_to_millis).unwrap_or(0),
                    memo: None,
                }
            })
            .collect())
    }

    fn pages(&self, chapter: &Chapter) -> Result<Vec<Page>> {
        let id = chapter.url.trim_start_matches("/chapter/");
        let json: Value = http_get_json(&format!("{API}/at-home/server/{id}"), HEADERS)?;
        let base = json["baseUrl"].as_str().ok_or("no baseUrl")?;
        let hash = json["chapter"]["hash"].as_str().ok_or("no chapter hash")?;
        Ok(json["chapter"]["data"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .enumerate()
            .map(|(i, file)| Page::new(i as u32, format!("{base}/data/{hash}/{file}")))
            .collect())
    }
}

fn list(page: u32, order: &str, extra: &str) -> Result<MangaPage> {
    let offset = page.saturating_sub(1) * PAGE_SIZE;
    let json: Value = http_get_json(
        &format!("{API}/manga?limit={PAGE_SIZE}&offset={offset}&includes[]=cover_art{RATINGS}{order}{extra}"),
        HEADERS,
    )?;
    let mangas: Vec<Manga> = json["data"].as_array().into_iter().flatten().map(to_manga).collect();
    let total = json["total"].as_u64().unwrap_or(0);
    Ok(MangaPage { has_next: (offset as u64 + mangas.len() as u64) < total, mangas })
}

fn id_of(url: &str) -> Result<&str> {
    url.strip_prefix("/manga/").filter(|s| !s.is_empty()).ok_or_else(|| format!("not a MangaDex title: {url}"))
}

fn to_manga(m: &Value) -> Manga {
    let id = m["id"].as_str().unwrap_or_default();
    let a = &m["attributes"];
    let rel = |kind: &str| {
        m["relationships"].as_array().into_iter().flatten().find(|r| r["type"] == kind).map(|r| &r["attributes"])
    };
    let localized = |v: &Value| v["en"].as_str().or_else(|| v.as_object()?.values().find_map(Value::as_str)).map(str::to_string);
    Manga {
        url: format!("/manga/{id}"),
        title: localized(&a["title"]).unwrap_or_default(),
        thumbnail_url: rel("cover_art")
            .and_then(|c| c["fileName"].as_str())
            .map(|f| format!("{COVERS}/{id}/{f}.512.jpg")),
        author: rel("author").and_then(|x| x["name"].as_str()).map(str::to_string),
        artist: rel("artist").and_then(|x| x["name"].as_str()).map(str::to_string),
        description: localized(&a["description"]),
        genre: a["tags"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|t| localized(&t["attributes"]["name"]))
            .collect(),
        status: match a["status"].as_str() {
            Some("ongoing") => 1,
            Some("completed") => 2,
            Some("cancelled") => 5,
            Some("hiatus") => 6,
            _ => 0,
        },
        ..Default::default()
    }
}

/// `2024-03-05T10:20:30+00:00` to epoch milliseconds (UTC; a non-zero offset is applied).
fn iso_to_millis(s: &str) -> i64 {
    let num = |r: std::ops::Range<usize>| s.get(r).and_then(|p| p.parse::<i64>().ok());
    let (Some(y), Some(mo), Some(d), Some(h), Some(mi), Some(sec)) =
        (num(0..4), num(5..7), num(8..10), num(11..13), num(14..16), num(17..19))
    else {
        return 0;
    };
    // Days from civil date (Howard Hinnant's algorithm).
    let (y, m) = if mo <= 2 { (y - 1, mo + 9) } else { (y, mo - 3) };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * m + 2) / 5 + d - 1;
    let days = era * 146097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719468;
    let offset = match s.get(19..) {
        Some(t) if t.len() >= 6 && (t.starts_with('+') || t.starts_with('-')) => {
            let sign = if t.starts_with('-') { -1 } else { 1 };
            let oh: i64 = t[1..3].parse().unwrap_or(0);
            let om: i64 = t[4..6].parse().unwrap_or(0);
            sign * (oh * 3600 + om * 60)
        }
        _ => 0,
    };
    (days * 86400 + h * 3600 + mi * 60 + sec - offset) * 1000
}

export_source!(MangaDex);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_dates_become_epoch_millis() {
        assert_eq!(iso_to_millis("1970-01-01T00:00:00+00:00"), 0);
        assert_eq!(iso_to_millis("2000-03-01T00:00:00+00:00"), 951_868_800_000);
        assert_eq!(iso_to_millis("2000-03-01T02:00:00+02:00"), 951_868_800_000);
        assert_eq!(iso_to_millis("garbage"), 0);
    }

    #[test]
    fn a_title_is_mapped_from_the_api_shape() {
        let v: Value = serde_json::json!({
            "id": "abc", "attributes": {"title": {"ja": "x", "en": "Hello"}, "status": "ongoing",
              "tags": [{"attributes": {"name": {"en": "Action"}}}]},
            "relationships": [{"type": "cover_art", "attributes": {"fileName": "c.jpg"}},
                              {"type": "author", "attributes": {"name": "Someone"}}]
        });
        let m = to_manga(&v);
        assert_eq!(m.url, "/manga/abc");
        assert_eq!(m.title, "Hello");
        assert_eq!(m.thumbnail_url.as_deref(), Some("https://uploads.mangadex.org/covers/abc/c.jpg.512.jpg"));
        assert_eq!((m.author.as_deref(), m.status, m.genre.as_slice()), (Some("Someone"), 1, ["Action".to_string()].as_slice()));
    }
}
