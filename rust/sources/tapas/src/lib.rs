//! Tapas, written in Rust. Port of the Keiyoushi extension `en.tapastic`. The popular and latest lists come from the
//! site's cosmos API (JSON), search and details are HTML pages, the episode list is a paged JSON endpoint and the pages
//! of a chapter are the images of the reader (plus, optionally, the author's note drawn as a last page).

use serde::Deserialize;
use sk_source_sdk::*;

const BASE: &str = "https://tapas.io";
const API: &str = "https://story-api.tapas.io";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:105.0) Gecko/20100101 Firefox/105.0";

const SHOW_LOCKED: &str = "lockedChapterVisibilityPref";
const SHOW_SCHEDULED: &str = "scheduledChapterVisibilityPref";
const SHOW_NOTES: &str = "showAuthorsNotes";

// ------------------------------------------------------------------ the API's shapes

#[derive(Deserialize)]
struct Wrapper<T> {
    data: T,
    meta: Option<Meta>,
}

#[derive(Deserialize)]
struct Meta {
    pagination: Pagination,
}

#[derive(Deserialize)]
struct Pagination {
    #[serde(default = "yes")]
    last: bool,
}

fn yes() -> bool {
    true
}

#[derive(Deserialize)]
struct Items {
    items: Vec<Series>,
}

#[derive(Deserialize)]
struct Series {
    #[serde(rename = "seriesId")]
    series_id: i64,
    title: String,
    #[serde(default)]
    description: String,
    #[serde(rename = "assetProperty")]
    asset: Asset,
}

#[derive(Deserialize)]
struct Asset {
    #[serde(rename = "bookCoverImage", default)]
    cover: std::collections::BTreeMap<String, String>,
}

#[derive(Deserialize)]
struct Episodes {
    pagination: EpisodePagination,
    episodes: Vec<Episode>,
}

#[derive(Deserialize)]
struct EpisodePagination {
    #[serde(default)]
    has_next: bool,
}

#[derive(Deserialize)]
struct Episode {
    id: i64,
    title: String,
    publish_date: String,
    unlocked: bool,
    free: bool,
    scene: f32,
    scheduled: bool,
}

fn manga_of(s: Series) -> Manga {
    Manga {
        url: format!("/series/{}", s.series_id),
        title: s.title,
        description: Some(s.description).filter(|d| !d.is_empty()),
        thumbnail_url: s.asset.cover.values().next().map(|u| format!("{u}.png")),
        ..Default::default()
    }
}

/// A list answer: the titles, and whether another page follows (`meta.pagination.last` is false).
fn list(url: &str) -> Result<MangaPage> {
    let w: Wrapper<Items> = get(url).send()?.json()?;
    let has_next = !w.meta.map(|m| m.pagination.last).unwrap_or(true);
    Ok(MangaPage { mangas: w.data.items.into_iter().map(manga_of).collect(), has_next })
}

// ------------------------------------------------------------------ the source

#[derive(Default)]
pub struct Tapas;

impl Source for Tapas {
    fn info(&self) -> Info {
        Info {
            name: "Tapas".into(),
            lang: "en".into(),
            base_url: BASE.into(),
            supports_latest: true,
            custom_urls: true,
            // an age gate the site asks every browser
            cookies: vec![CookieDef::new("tapas.io", "birthDate", "1990-01-01"), CookieDef::new("tapas.io", "adjustedBirthDate", "1990-01-01")],
            headers: vec![("Referer".into(), "https://m.tapas.io".into()), ("User-Agent".into(), USER_AGENT.into())],
            prefs: vec![
                PrefDef::switch(
                    SHOW_LOCKED,
                    "Show paywalled chapters",
                    "Tapas requires login/payment for some chapters. Enable to always show paywalled chapters. Hiding chapters with paid access will also hide scheduled chapters.",
                    true,
                ),
                PrefDef::switch(SHOW_SCHEDULED, "Show scheduled chapters", "Scheduled chapters can be hidden from the chapter list by enabling this option.", true),
                PrefDef::switch(SHOW_NOTES, "Show author's notes", "Enable to see the author's notes at the end of chapters (if they're there).", true),
            ],
            ..Default::default()
        }
    }

    fn popular(&self, page: u32) -> Result<MangaPage> {
        list(&format!("{API}/cosmos/api/v1/landing/ranking?category_type=COMIC&subtab_id=17&size=25&page={}", page.saturating_sub(1)))
    }

    fn latest(&self, page: u32) -> Result<MangaPage> {
        list(&format!(
            "{API}/cosmos/api/v1/landing/genre?category_type=COMIC&sort_option=NEWEST_EPISODE&subtab_id=17&pageSize=25&page={}",
            page.saturating_sub(1)
        ))
    }

    fn search(&self, page: u32, query: &str, _filters: &Filters) -> Result<MangaPage> {
        let mut url = Url::parse(&format!("{BASE}/search")).map_err(|e| e.to_string())?;
        url.query_pairs_mut().append_pair("pageNumber", &page.to_string()).append_pair("q", query).append_pair("t", "COMICS");
        let doc = get(url.as_str()).send()?.html();
        let mangas = doc
            .select(".search-item-wrap")
            .iter()
            .filter_map(|el| {
                let link = el.select_first(".item__thumb a, .title-section .title a")?;
                let title = el.select_first(".item__thumb img").map(|i| i.attr("alt")).filter(|t| !t.is_empty()).unwrap_or_else(|| el.select(".title-section .title a").text());
                Some(Manga {
                    url: format!("/series/{}", link.attr("data-series-id")),
                    title,
                    thumbnail_url: Some(el.select(".item__thumb img, .thumb-wrap img").attr("src")).filter(|u| !u.is_empty()),
                    description: el.select_first(".desc.force.mbm").map(|d| d.text()),
                    ..Default::default()
                })
            })
            .collect();
        Ok(MangaPage { mangas, has_next: doc.select_first("a[class*=paging__button--next]").is_some() })
    }

    fn details(&self, manga: &Manga) -> Result<Manga> {
        let doc = get(&format!("{BASE}{}/info", manga.url)).send()?.html();
        let title = doc.select_first(".info__right .title").ok_or("the page has no title (the site changed?)")?.text();
        let mut description = doc.select_first(".description__body").map(|d| d.text()).unwrap_or_default();
        if let Some(kind) = doc.select_first(r#".stats > a[href^="/static-landing/genre?category="]"#) {
            description.push_str(&format!("\n\nType: {}", kind.text()));
        }
        if let Some(colophon) = doc.select_first(".colophon") {
            let text = colophon.whole_text();
            let text = text.trim_start();
            // the colophon starts with the title (and "(Comic)" / "(Novel)"): drop that line
            let rest = match text.split_once('\n') {
                Some((first, rest)) if first.to_lowercase().starts_with(&title.to_lowercase()) => rest.trim_start(),
                _ => text,
            };
            description.push_str(&format!("\n\n{}", rest.trim()));
        }
        let mut genres: Vec<String> = Vec::new();
        for g in doc.select(".genre-btn").each_text() {
            if !genres.contains(&g) {
                genres.push(g);
            }
        }
        let status = doc
            .select_first(".schedule-ico:has(.sp-ico-updated-line-pwt) + .schedule-label")
            .map(|s| s.text().to_lowercase())
            .map(|t| if t.contains("updates") { 1 } else if t.contains("completed") { 2 } else { 0 })
            .unwrap_or(0);
        Ok(Manga {
            url: manga.url.clone(),
            title,
            thumbnail_url: doc.select_first(".thumb.js-thumbnail img").map(|i| i.abs_url("src")).filter(|u| !u.is_empty()),
            description: Some(description).filter(|d| !d.trim().is_empty()),
            genre: genres,
            author: Some(doc.select(".creator-section .name").each_text().join(", ")).filter(|a| !a.is_empty()),
            status,
            ..Default::default()
        })
    }

    fn chapters(&self, manga: &Manga) -> Result<Vec<Chapter>> {
        let (show_locked, show_scheduled) = (pref_bool(SHOW_LOCKED, true), pref_bool(SHOW_SCHEDULED, true));
        let mut out = Vec::new();
        for page in 1.. {
            let url = format!("{BASE}{}/episodes?page={page}&sort=NEWEST&since={}&large=true&last_access=0&=", manga.url, now_ms());
            let w: Wrapper<Episodes> = get(&url).send()?.json()?;
            out.extend(w.data.episodes.into_iter().filter(|e| (show_locked || e.unlocked || e.free) && (show_scheduled || !e.scheduled)).map(|e| Chapter {
                url: format!("/episode/{}", e.id),
                name: if e.unlocked || e.free { e.title } else { format!("🔒 {} ", e.title) },
                chapter_number: e.scene,
                scanlator: None,
                date_upload: parse_iso(&e.publish_date).unwrap_or(0),
                memo: None,
            }));
            if !w.data.pagination.has_next {
                break;
            }
        }
        Ok(out)
    }

    fn pages(&self, chapter: &Chapter) -> Result<Vec<Page>> {
        let doc = get(&format!("{BASE}{}", chapter.url)).send()?.html();
        // the "Style" button is on novels even when the chapter is locked
        if doc.select_first(r#".toolbar a[data-type="style"]"#).is_some() {
            return Err("This is not a comic, but a novel chapter".into());
        }
        let mut pages: Vec<Page> = doc
            .select("img.content__img")
            .iter()
            .enumerate()
            .map(|(i, img)| Page::new(i as u32, if img.has_attr("data-src") { img.abs_url("data-src") } else { img.abs_url("src") }))
            .collect();
        if pages.is_empty() {
            return Err("Chapter locked".into());
        }
        if pref_bool(SHOW_NOTES, true) {
            let story = doc.select("p.js-episode-story").text();
            if !story.trim().is_empty() {
                let creator = doc.select_first("a.name.js-fb-tracking").map(|c| c.text()).unwrap_or_default();
                let mut page = Page::new(pages.len() as u32, format!("{BASE}/author-note"));
                page.ops = vec![ImageOp::Text { title: format!("Author's Notes from {creator}"), body: story.trim().to_string() }];
                pages.push(page);
            }
        }
        Ok(pages)
    }

    fn manga_url(&self, manga: &Manga) -> Option<String> {
        Some(format!("{BASE}{}/info", manga.url))
    }
}

export_source!(Tapas);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_series_becomes_a_title_with_its_cover() {
        let s: Series = serde_json::from_str(r#"{"seriesId":7,"title":"T","description":"D","assetProperty":{"bookCoverImage":{"a":"https://c.test/x"}}}"#).unwrap();
        let m = manga_of(s);
        assert_eq!((m.url.as_str(), m.thumbnail_url.as_deref()), ("/series/7", Some("https://c.test/x.png")));
    }

    #[test]
    fn the_last_page_flag_ends_the_list() {
        let w: Wrapper<Items> = serde_json::from_str(r#"{"data":{"items":[]},"meta":{"pagination":{"last":true}}}"#).unwrap();
        assert!(w.meta.map(|m| m.pagination.last).unwrap_or(true));
    }
}
