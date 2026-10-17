//! Keyoapp (V2): server-rendered manga sites whose browse and search run through `/?_ajax=...` endpoints that answer
//! `{html, has_more}` and whose chapter list comes from `/?_chapters_html=1&series_slug=...`. A site is a type that
//! implements [`Theme`]; `export_source!(Site<MySite>)` makes it a source.
//!
//! Port of `KeyoappV2` and `KeyoappBase` from the `keyoapp` multisrc of the Kotlin extensions. The older Keyoapp
//! layout (`Keyoapp`, which needs a JavaScript engine for its images) is not ported: no installed site uses it.

use fancy_regex::Regex;
use serde::Deserialize;
use sk_source_sdk::html::{Document, El};
use sk_source_sdk::*;
use std::marker::PhantomData;

const SHOW_PAID_PREF: &str = "pref_show_paid_chap";

pub trait Theme: 'static {
    const NAME: &'static str;
    const LANG: &'static str = "en";
    const BASE_URL: &'static str;
    const VERSION_ID: u32 = 1;
    /// `java.time` pattern of absolute chapter dates; relative ones ("2 days ago") are always understood.
    const DATE_PATTERN: &'static str = "MMM d, yyyy";
}

pub struct Site<T>(PhantomData<T>);

impl<T> Default for Site<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

const TYPES: [(&str, &str); 5] = [("All", ""), ("Manhwa", "manhwa"), ("Manhua", "manhua"), ("Mangatoon", "mangatoon"), ("Manga", "manga")];
const STATUSES: [(&str, &str); 3] = [("All", ""), ("Ongoing", "ongoing"), ("Completed", "completed")];
const SORTS: [(&str, &str); 7] = [
    ("Latest", "latest"),
    ("Trending", "trending"),
    ("Popular", "popular"),
    ("Most Viewed", "views"),
    ("Top Rated", "rating"),
    ("A-Z", "az"),
    ("Z-A", "za"),
];

fn base<T: Theme>() -> String {
    base_url(T::BASE_URL)
}

/// `<base>/?k=v&...` for the site's ajax endpoints.
fn endpoint<T: Theme>(params: &[(&str, &str)]) -> Result<String> {
    let mut url = Url::parse(&base::<T>()).map_err(|e| format!("bad base address: {e}"))?;
    url.set_path("/");
    url.query_pairs_mut().extend_pairs(params);
    Ok(url.to_string())
}

impl<T: Theme> Source for Site<T> {
    fn info(&self) -> Info {
        Info {
            name: T::NAME.into(),
            lang: T::LANG.into(),
            base_url: T::BASE_URL.into(),
            version_id: T::VERSION_ID,
            supports_latest: true,
            supports_filter_fetching: true,
            prefs: vec![PrefDef::switch(
                SHOW_PAID_PREF,
                "Display paid chapters",
                "Paid chapters are listed (with a lock). Off: only free chapters are shown.",
                false,
            )],
            ..Default::default()
        }
    }

    fn popular(&self, page: u32) -> Result<MangaPage> {
        browse::<T>(page, None, Some("popular"), &Filters::default())
    }

    fn latest(&self, page: u32) -> Result<MangaPage> {
        let url = endpoint::<T>(&[("_ajax", "recent-chapters"), ("page", &page.to_string())])?;
        let doc = get(&url).send()?.html();
        let mangas = doc
            .select("article.ru-card")
            .iter()
            .filter_map(|card| {
                let link = card.select_first("h2.ru-title > a")?;
                Some(Manga {
                    url: path_of(&link.abs_url("href")),
                    title: link.text(),
                    thumbnail_url: card.select_first("img.ru-cover-img").and_then(|i| non_blank(i.attr("data-src"))),
                    ..Default::default()
                })
            })
            .collect();
        let has_next = doc.select_first("button[aria-label=\"Next recently updated page\"]:not([disabled])").is_some();
        Ok(MangaPage { mangas, has_next })
    }

    fn search(&self, page: u32, query: &str, filters: &Filters) -> Result<MangaPage> {
        browse::<T>(page, Some(query), None, filters)
    }

    fn filters(&self, data: Option<&Value>) -> Vec<FilterDef> {
        let tags: Vec<String> = data.and_then(|d| serde_json::from_value(d.clone()).ok()).unwrap_or_default();
        let mut f = vec![
            FilterDef::select("type", "Type", &TYPES, 0),
            FilterDef::select("status", "Status", &STATUSES, 0),
            FilterDef::select("sort", "Sort by", &SORTS, 0),
        ];
        if !tags.is_empty() {
            let items: Vec<(String, String)> = tags.iter().map(|t| (ucfirst(t), t.clone())).collect();
            f.push(FilterDef::checkboxes("tags", "Genres", &items));
        }
        f
    }

    fn filter_data(&self) -> Result<Option<Value>> {
        let doc = get(&endpoint::<T>(&[("browse", "1")])?).send()?.html();
        let tags: Vec<String> = doc
            .select("#fsel-genres a[role=option][href]")
            .iter()
            .filter_map(|a| {
                let u = Url::parse(&a.abs_url("href")).ok()?;
                u.query_pairs().find(|(k, _)| k == "tag[]").map(|(_, v)| v.into_owned())
            })
            .collect();
        if tags.is_empty() {
            return Err("Genre tags not found".into());
        }
        Ok(Some(json!(tags)))
    }

    fn update(&self, manga: &Manga, fetch_details: bool, fetch_chapters: bool) -> Result<Update> {
        let slug = manga.url.trim_start_matches('/').to_string();
        let mut requests = Vec::new();
        if fetch_details {
            requests.push(get(&series_url::<T>(&slug)));
        }
        if fetch_chapters {
            requests.push(get(&endpoint::<T>(&[("_chapters_html", "1"), ("series_slug", &slug)])?));
        }
        let mut answers = send_all(requests).into_iter();
        let mut update = Update::default();
        if fetch_details {
            let doc = answers.next().ok_or("no answer")??.html();
            if doc.select_first("[data-series-uid]").is_none() {
                return Err("Series not found".into());
            }
            update.manga = Some(parse_details::<T>(&doc, &slug));
        }
        if fetch_chapters {
            update.chapters = Some(parse_chapters::<T>(answers.next().ok_or("no answer")??)?);
        }
        Ok(update)
    }

    fn details(&self, manga: &Manga) -> Result<Manga> {
        self.update(manga, true, false)?.manga.ok_or_else(|| "no details".into())
    }

    fn chapters(&self, manga: &Manga) -> Result<Vec<Chapter>> {
        self.update(manga, false, true)?.chapters.ok_or_else(|| "no chapters".into())
    }

    fn by_url(&self, url: &str) -> Result<Option<Manga>> {
        let u = Url::parse(url).map_err(|e| e.to_string())?;
        let base = Url::parse(&base::<T>()).map_err(|e| e.to_string())?;
        let segments: Vec<&str> = u.path_segments().map(|s| s.filter(|p| !p.is_empty()).collect()).unwrap_or_default();
        if u.host_str() != base.host_str() || segments.len() != 1 {
            return Ok(None);
        }
        let doc = get(url).send()?.html();
        if let Some(series) = doc.select_first("[data-series-uid]") {
            let slug = series.attr("data-series-slug");
            return Ok(Some(parse_details::<T>(&doc, &slug)));
        }
        // A chapter page names its series.
        let Some(slug) = doc.select_first("#sidebar-ch-list[data-series-slug]").map(|e| e.attr("data-series-slug")) else { return Ok(None) };
        let series = get(&series_url::<T>(&slug)).send()?.html();
        Ok(Some(parse_details::<T>(&series, &slug)))
    }

    fn pages(&self, chapter: &Chapter) -> Result<Vec<Page>> {
        let doc = get(&absolute(&base::<T>(), &chapter.url)).send()?.html();
        Ok(doc
            .select("#strip-reader img.reader-page")
            .iter()
            .enumerate()
            .map(|(i, img)| Page::new(i as u32, img.abs_url("src")))
            .collect())
    }
}

fn series_url<T: Theme>(slug: &str) -> String {
    format!("{}/{slug}", base::<T>())
}

fn non_blank(s: String) -> Option<String> {
    (!s.trim().is_empty()).then_some(s)
}

fn ucfirst(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}

// ------------------------------------------------------------------ lists

#[derive(Deserialize)]
struct SearchResponse {
    html: String,
    #[serde(default)]
    has_more: bool,
}

fn browse<T: Theme>(page: u32, query: Option<&str>, sort: Option<&str>, filters: &Filters) -> Result<MangaPage> {
    let offset = ((page - 1) * 20).to_string();
    let mut params: Vec<(String, String)> = vec![("_ajax".into(), "search".into())];
    if let Some(q) = query.map(str::trim).filter(|q| !q.is_empty()) {
        params.push(("q".into(), q.into()));
    }
    if let Some(s) = sort {
        params.push(("sort".into(), s.into()));
    }
    params.push(("offset".into(), offset));
    // Selects first, then the genre boxes; a select left on its first option sends nothing.
    for (key, options) in [("type", &TYPES[..]), ("status", &STATUSES[..]), ("sort", &SORTS[..])] {
        if filters.index(key) != 0 {
            params.push((key.into(), filters.select_value(key, options).into()));
        }
    }
    for tag in filters.included("tags") {
        params.push(("tag[]".into(), tag.into()));
    }

    let pairs: Vec<(&str, &str)> = params.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    let data: SearchResponse = get(&endpoint::<T>(&pairs)?).send()?.json()?;
    let doc = Document::parse(&data.html, &base::<T>());
    let mangas = doc
        .select("a.group[href]")
        .iter()
        .filter(|a| !is_novel(a))
        .filter_map(|a| {
            Some(Manga {
                url: path_of(&a.abs_url("href")),
                title: a.select_first("h2")?.text(),
                thumbnail_url: a.select_first("img").and_then(|i| non_blank(i.attr("data-src"))),
                ..Default::default()
            })
        })
        .collect();
    Ok(MangaPage { mangas, has_next: data.has_more })
}

fn is_novel(el: &El) -> bool {
    !el.select(r#"[data-type="novel" i], span:matchesOwn((?i)^novel$)"#).is_empty()
}

// ------------------------------------------------------------------ details and chapters

fn parse_details<T: Theme>(doc: &Document, slug: &str) -> Manga {
    let meta = |label: &str| {
        doc.select_first(&format!("span.text-xs:containsOwn({label})"))
            .and_then(|s| s.parent())
            .and_then(|p| p.next_sibling())
            .and_then(|n| n.text_or_none())
    };
    let genres = doc.select("a[href*=\"?browse&tag\"]").each_text().join(", ");
    Manga {
        url: format!("/{slug}"),
        title: doc.select_first("h1").map(|h| h.text()).unwrap_or_default(),
        description: doc.select_first("#series-desc").map(|d| d.whole_text().trim().to_string()),
        thumbnail_url: doc.select_first("meta[property=og:image]").and_then(|m| non_blank(m.attr("content"))),
        genre: genres.split(", ").filter(|g| !g.is_empty()).map(str::to_string).collect(),
        author: meta("Author"),
        artist: meta("Artist"),
        status: parse_status(doc.select_first("div.text-xs:containsOwn(Status) + div span.text-sm").map(|e| e.text())),
        initialized: true,
        ..Default::default()
    }
}

fn parse_status(text: Option<String>) -> u8 {
    match text.map(|t| t.to_lowercase()).as_deref() {
        Some("ongoing") => 1,
        Some("completed") => 2,
        Some("dropped") => 5,
        Some("paused") => 6,
        _ => 0,
    }
}

#[derive(Deserialize)]
struct ChaptersResponse {
    rows_html: String,
}

fn parse_chapters<T: Theme>(resp: Response) -> Result<Vec<Chapter>> {
    let data: ChaptersResponse = resp.json()?;
    let doc = Document::parse(&data.rows_html, &base::<T>());
    let show_paid = pref_bool(SHOW_PAID_PREF, false);
    Ok(doc
        .select("a.chapter-row[href]")
        .iter()
        .filter(|row| show_paid || row.attr("data-ch-locked") != "1")
        .map(|row| {
            let label = row.attr("data-ch-label");
            Chapter {
                url: path_of(&row.abs_url("href")),
                name: if row.attr("data-ch-locked") == "1" { format!("🔒 {label}") } else { label },
                date_upload: row.select_first(".ch-date-row span:last-child").map(|d| parse_chapter_date::<T>(&d.text())).unwrap_or(0),
                ..Default::default()
            }
        })
        .collect())
}

/// "3 days ago" or an absolute date; 0 when it is neither.
fn parse_chapter_date<T: Theme>(s: &str) -> i64 {
    if s.contains("ago") {
        return relative(s, now_ms());
    }
    parse_date(s, T::DATE_PATTERN, "en").unwrap_or(0)
}

/// `(a|one|N) unit ago`, counted back from `now_ms` (seconds and milliseconds dropped, like the extension).
fn relative(s: &str, now_ms: i64) -> i64 {
    let Ok(Some(c)) = Regex::new(r"(a|one|\d+)\s*(\w+)\s+ago").unwrap().captures(s) else { return 0 };
    let amount = match &c[1] {
        "a" | "one" => 1,
        n => match n.parse::<i64>() {
            Ok(n) => n,
            Err(_) => return 0,
        },
    };
    let unit = match &c[2] {
        "m" | "min" | "mins" | "minute" | "minutes" => Unit::Minute,
        "h" | "hr" | "hrs" | "hour" | "hours" => Unit::Hour,
        "d" | "day" | "days" => Unit::Day,
        "w" | "wk" | "wks" | "week" | "weeks" => Unit::Week,
        "mo" | "mos" | "month" | "months" => Unit::Month,
        "y" | "yr" | "yrs" | "year" | "years" => Unit::Year,
        _ => return now_ms - now_ms.rem_euclid(60_000),
    };
    minus(now_ms - now_ms.rem_euclid(60_000), amount, unit)
}

#[cfg(test)]
mod tests;
