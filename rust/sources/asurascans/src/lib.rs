//! Asura Scans, written in Rust. Port of the Kotlin extension `en.asurascans`: a JSON API for lists, pages rendered by
//! Astro whose data sits in `props` attributes (details, chapter list, pages), chapter pictures that can come cut into
//! tiles and shuffled, and premium chapters that need the reader's login.

use fancy_regex::Regex;
use serde::Deserialize;
use sk_source_sdk::html::Document;
use sk_source_sdk::*;

const API: &str = "https://api.asurascans.com/api";
const BASE: &str = "https://asurascans.com";
const PER_PAGE: u32 = 20;
const HIDE_PREMIUM: &str = "pref_hide_premium_chapters";

const SORTS: [(&str, &str); 5] = [("Latest Update", "latest"), ("Popular", "popular"), ("Rating", "rating"), ("A-Z", "title"), ("Newest", "update")];
const STATUSES: [(&str, &str); 6] = [
    ("All", ""),
    ("Ongoing", "ongoing"),
    ("Completed", "completed"),
    ("Hiatus", "hiatus"),
    ("Dropped", "dropped"),
    ("Axed", "axed"),
];
const TYPES: [(&str, &str); 4] = [("All", ""), ("Manhwa", "manhwa"), ("Manhua", "manhua"), ("Mangatoon", "manga")];

#[derive(Default)]
pub struct AsuraScans;

impl Source for AsuraScans {
    fn info(&self) -> Info {
        Info {
            name: "Asura Scans".into(),
            lang: "en".into(),
            base_url: BASE.into(),
            supports_latest: true,
            supports_filter_fetching: true,
            supports_related: true,
            custom_urls: true,
            // Everything but the covers: the site and its API, not the picture servers.
            rate_limit: Some(RateLimit { permits: 2, period_ms: 2000, host: "asurascans.com,api.asurascans.com".into(), skip_path: String::new() }),
            prefs: vec![PrefDef::switch(HIDE_PREMIUM, "Hide premium chapters", "Hides the chapters that require a subscription to view", true)],
            ..Default::default()
        }
    }

    fn popular(&self, page: u32) -> Result<MangaPage> {
        list(page, "", &[("sort", "popular"), ("order", "desc")])
    }

    fn latest(&self, page: u32) -> Result<MangaPage> {
        list(page, "", &[("sort", "latest"), ("order", "desc")])
    }

    fn search(&self, page: u32, query: &str, filters: &Filters) -> Result<MangaPage> {
        let mut params: Vec<(String, String)> = Vec::new();
        let (index, ascending) = filters.sort("sort").unwrap_or((0, false));
        params.push(("sort".into(), SORTS.get(index).map_or("latest", |s| s.1).into()));
        params.push(("order".into(), if ascending { "asc" } else { "desc" }.into()));
        for (key, options) in [("status", &STATUSES[..]), ("type", &TYPES[..])] {
            let v = filters.select_value(key, options);
            if !v.is_empty() {
                params.push((key.into(), v.into()));
            }
        }
        let genres = filters.included("genres");
        if !genres.is_empty() {
            params.push(("genres".into(), genres.join(",")));
        }
        // Creators: one list that holds the artists, then the authors.
        let data: Option<FiltersData> = filter_data().and_then(|d| serde_json::from_value(d).ok());
        let picked = filters.index("creator");
        if let (Some(data), true) = (data, picked > 0) {
            if picked <= data.artists.len() {
                params.push(("artist".into(), data.artists[picked - 1].clone()));
            } else if let Some(author) = data.authors.get(picked - 1 - data.artists.len()) {
                params.push(("author".into(), author.clone()));
            }
        }
        let min = filters.text("min_chapters");
        if !min.is_empty() {
            params.push(("min_chapters".into(), min.into()));
        }
        let pairs: Vec<(&str, &str)> = params.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        list(page, query, &pairs)
    }

    fn filters(&self, data: Option<&Value>) -> Vec<FilterDef> {
        let mut f = vec![
            FilterDef::sort("sort", "Sort By", &SORTS, 0, false),
            FilterDef::select("status", "Status", &STATUSES, 0),
            FilterDef::select("type", "Type", &TYPES, 0),
        ];
        if let Some(data) = data.and_then(|d| serde_json::from_value::<FiltersData>(d.clone()).ok()) {
            let genres: Vec<(String, String)> = data.genres.iter().map(|g| (g.name.clone(), g.slug.clone())).collect();
            f.push(FilterDef::checkboxes("genres", "Genres", &genres));
            let mut names = vec![String::new()];
            names.extend(data.artists.iter().cloned());
            names.extend(data.authors.iter().cloned());
            f.push(FilterDef::Select { key: "creator".into(), name: "Creators".into(), options: names, default: 0 });
        }
        f.push(FilterDef::text("min_chapters", "Min Chapters"));
        f
    }

    fn filter_data(&self) -> Result<Option<Value>> {
        let mut answers = send_all(vec![get(&format!("{BASE}/browse")), get(&format!("{API}/creators"))]).into_iter();
        let browse = answers.next().ok_or("no answer")??.html();
        let creators: Data<Creators> = answers.next().ok_or("no answer")??.json()?;
        let genres: AvailableGenres = astro_prop(&browse, &["availableGenres"])?;
        let creators = creators.data.ok_or("no creators")?;
        let data = FiltersData { genres: genres.genres, authors: creators.authors, artists: creators.artists };
        Ok(Some(serde_json::to_value(data).map_err(|e| e.to_string())?))
    }

    fn by_url(&self, url: &str) -> Result<Option<Manga>> {
        let u = Url::parse(url).map_err(|e| e.to_string())?;
        let seg: Vec<&str> = u.path_segments().map(|s| s.collect()).unwrap_or_default();
        let same_host = u.host_str() == Url::parse(BASE).ok().and_then(|b| b.host_str().map(str::to_string)).as_deref();
        if !same_host || seg.first() != Some(&"comics") {
            return Ok(None);
        }
        let Some(slug) = seg.get(1) else { return Ok(None) };
        let stub = Manga { url: format!("/series/{slug}"), memo: Some(json!({ "slug": slug })), ..Default::default() };
        self.update(&stub, true, false)?.manga.map(Some).ok_or_else(|| "no details".into())
    }

    fn manga_url(&self, manga: &Manga) -> Option<String> {
        Some(format!("{BASE}/comics/{}", random_slug(manga)))
    }

    fn chapter_url(&self, chapter: &Chapter) -> Option<String> {
        let slug = chapter.memo.as_ref()?["mangaSlug"].as_str()?.to_string();
        let number = substring_after(&chapter.url, "/chapter/");
        Some(format!("{BASE}/comics/{slug}/chapter/{number}"))
    }

    fn update(&self, manga: &Manga, _fetch_details: bool, _fetch_chapters: bool) -> Result<Update> {
        let url = self.manga_url(manga).unwrap_or_default();
        let mut resp = get(&url).allow_errors().send()?;
        // The random suffix changes; without it the site redirects to the current one.
        if !(200..300).contains(&resp.status) {
            resp = get(&random_suffix().replace(&url, "").into_owned()).send()?;
        }
        let doc = resp.html();

        let details: DetailsDto = astro_prop(&doc, &["title", "description"])?;
        let ids: UrlDto = astro_prop(&doc, &["publicUrl"])?;
        let slug = ids.public_url.rsplit('/').next().unwrap_or_default().to_string();
        let mut m = details.to_manga();
        m.url = format!("/series/{}", ids.series_slug);
        m.memo = Some(json!({ "slug": slug }));

        let hide = pref_bool(HIDE_PREMIUM, true);
        let list: ChapterListDto = astro_prop(&doc, &["chapters"])?;
        let now = now_ms();
        let chapters = list
            .chapters
            .unwrap_or_default()
            .into_iter()
            .filter(|c| !(hide && c.is_locked(now)))
            .map(|c| c.to_chapter(&slug, &ids.series_slug, now))
            .collect();
        Ok(Update { manga: Some(m), chapters: Some(chapters) })
    }

    fn details(&self, manga: &Manga) -> Result<Manga> {
        self.update(manga, true, false)?.manga.ok_or_else(|| "no details".into())
    }

    fn chapters(&self, manga: &Manga) -> Result<Vec<Chapter>> {
        self.update(manga, false, true)?.chapters.ok_or_else(|| "no chapters".into())
    }

    fn related(&self, manga: &Manga) -> Result<Vec<Manga>> {
        let doc = get(&self.manga_url(manga).unwrap_or_default()).send()?.html();
        Ok(doc
            .select("h2:contains(recommended) + div a[href*=/comics/]")
            .iter()
            .filter_map(|a| {
                let slug = Url::parse(&a.abs_url("href")).ok()?.path_segments()?.nth(1)?.to_string();
                Some(Manga {
                    url: format!("/series/{}", random_suffix().replace(&slug, "")),
                    memo: Some(json!({ "slug": slug })),
                    title: a.select_first(".mt-2 span")?.own_text(),
                    thumbnail_url: a.select_first("img").map(|i| i.abs_url("src")),
                    ..Default::default()
                })
            })
            .collect())
    }

    fn pages(&self, chapter: &Chapter) -> Result<Vec<Page>> {
        let url = self.chapter_url(chapter).ok_or("Refresh Chapter List")?;
        let resp = get(&url).send()?;
        let doc = resp.html();
        let mut pages: Vec<PageDto> = astro_prop::<PageListDto>(&doc, &["pages"]).map(|p| p.pages).unwrap_or_default();

        if pages.is_empty() {
            // Premium chapters: the API wants the reader's login and a token the page carries.
            let Some(access_token) = cookie(BASE, "access_token") else { return Ok(Vec::new()) };
            let page_token = doc
                .select_first("script:containsData(pageToken)")
                .and_then(|s| {
                    let data = s.data();
                    Regex::new(r#"pageToken\*=\*"([^"]+)""#).ok()?.captures(data.as_str()).ok().flatten().map(|c| c[1].to_string())
                })
                .unwrap_or_else(|| "asura-reader-2026".into());
            let final_url = Url::parse(&resp.url).map_err(|e| e.to_string())?;
            let seg: Vec<&str> = final_url.path_segments().map(|s| s.collect()).unwrap_or_default();
            let (Some(series), Some(number)) = (seg.get(1), seg.get(3)) else { return Ok(Vec::new()) };
            pages = get(&format!("{API}/series/{series}/chapters/{number}"))
                .header("Authorization", &format!("Bearer {access_token}"))
                .header("X-Page-Token", &page_token)
                .send()
                .and_then(|r| r.json::<PremiumPageList>())
                .map(|p| p.data.chapter.pages)
                .unwrap_or_default();
        }

        Ok(pages
            .into_iter()
            .enumerate()
            .map(|(i, p)| {
                let mut page = Page::new(i as u32, p.url);
                if let Some(order) = p.tiles.filter(|t| !t.is_empty()) {
                    page.ops = vec![ImageOp::Shuffle { cols: p.tile_cols.unwrap_or(4), rows: p.tile_rows.unwrap_or(5), order }];
                }
                page
            })
            .collect())
    }
}

fn list(page: u32, query: &str, params: &[(&str, &str)]) -> Result<MangaPage> {
    let mut url = format!("{API}/series?offset={}&limit={PER_PAGE}", page.saturating_sub(1) * PER_PAGE);
    if !query.trim().is_empty() {
        url.push_str(&format!("&search={}", url_encode(query)));
    }
    for (k, v) in params {
        url.push_str(&format!("&{k}={}", url_encode(v)));
    }
    let result: Data<Vec<MangaDto>> = get(&url).send()?.json()?;
    Ok(MangaPage {
        mangas: result.data.unwrap_or_default().into_iter().map(|m| m.to_manga()).collect(),
        has_next: result.meta.is_some_and(|m| m.has_more),
    })
}

/// The slug the site currently uses in a title's address (it carries a random suffix that changes).
fn random_slug(manga: &Manga) -> String {
    if let Some(s) = manga.memo.as_ref().and_then(|m| m["slug"].as_str()) {
        return s.to_string();
    }
    let u = &manga.url;
    let part = |marker: &str| substring_after(u, marker).split('/').next().unwrap_or_default().to_string();
    if u.contains("/series/") {
        part("/series/")
    } else if u.contains("/comics/") {
        part("/comics/")
    } else if u.contains("/manga/") {
        // the oldest form: `/manga/<number>-<slug>`
        Regex::new(r"^/manga/(\d+-)?([^/]+)/?$")
            .ok()
            .and_then(|r| r.captures(u.as_str()).ok().flatten().map(|c| c[2].to_string()))
            .unwrap_or_else(|| part("/manga/"))
    } else {
        u.trim_matches('/').to_string()
    }
}

fn random_suffix() -> Regex {
    Regex::new(r"-[a-z0-9]{8}$").expect("a valid pattern")
}

// ------------------------------------------------------------------ Astro props

/// The data of an Astro island whose `props` attribute mentions every one of `keys`, with Astro's `[type, value]`
/// wrappers removed.
fn astro_prop<T: for<'de> Deserialize<'de>>(doc: &Document, keys: &[&str]) -> Result<T> {
    let selector: String = keys.iter().map(|k| format!("[props*={k}]")).collect();
    let props = doc.select_first(&selector).map(|e| e.attr("props")).ok_or_else(|| format!("Unable to find prop with {keys:?}"))?;
    let json: Value = serde_json::from_str(&props).map_err(|e| format!("bad props: {e}"))?;
    serde_json::from_value(unwrap_astro(json)).map_err(|e| format!("unexpected props for {keys:?}: {e}"))
}

/// Astro serialises `5` as `[0, 5]`, `"x"` as `[0, "x"]` and a list as `[1, [..]]`; `[0]` and `[]` are nothing.
fn unwrap_astro(v: Value) -> Value {
    match v {
        Value::Array(mut a) => {
            let tagged = a.first().is_some_and(|t| t.is_i64() || t.is_u64());
            match a.len() {
                0 => Value::Null,
                1 if tagged => Value::Null,
                2 if tagged => unwrap_astro(a.swap_remove(1)),
                _ => Value::Array(a.into_iter().map(unwrap_astro).collect()),
            }
        }
        Value::Object(o) => Value::Object(o.into_iter().map(|(k, v)| (k, unwrap_astro(v))).collect()),
        other => other,
    }
}

// ------------------------------------------------------------------ data

#[derive(Deserialize)]
struct Data<T> {
    data: Option<T>,
    meta: Option<Meta>,
}

#[derive(Deserialize)]
struct Meta {
    #[serde(default)]
    has_more: bool,
}

#[derive(Deserialize)]
struct MangaDto {
    public_url: String,
    slug: String,
    title: String,
    #[serde(alias = "coverUrl")]
    cover: String,
}

impl MangaDto {
    fn to_manga(self) -> Manga {
        Manga {
            title: self.title,
            thumbnail_url: Some(self.cover),
            // the old address form, so titles already in a library stay the same
            url: format!("/series/{}", self.slug),
            memo: Some(json!({ "slug": self.public_url.rsplit('/').next().unwrap_or_default() })),
            ..Default::default()
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DetailsDto {
    title: String,
    cover_url: String,
    author: Option<String>,
    artist: Option<String>,
    description: Option<String>,
    rating: Option<f64>,
    bookmark_count: Option<u64>,
    #[serde(rename = "type")]
    kind: Option<String>,
    popularity_rank: Option<u64>,
    alternative_titles: Option<String>,
    genres: Option<Vec<GenreDto>>,
    status: Option<String>,
}

impl DetailsDto {
    fn to_manga(&self) -> Manga {
        let mut genre = Vec::new();
        if let Some(k) = &self.kind {
            let mut c = k.chars();
            genre.extend(c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()));
        }
        genre.extend(self.genres.iter().flatten().map(|g| g.name.clone()));
        Manga {
            title: self.title.clone(),
            thumbnail_url: Some(self.cover_url.clone()),
            author: self.author.clone(),
            artist: self.artist.clone(),
            description: Some(self.description_text()),
            genre,
            status: match self.status.as_deref().map(str::to_lowercase).as_deref() {
                Some("ongoing") => 1,
                Some("completed") => 2,
                Some("hiatus") => 6,
                Some("dropped" | "axed") => 5,
                _ => 0,
            },
            initialized: true,
            ..Default::default()
        }
    }

    fn description_text(&self) -> String {
        let mut out = String::new();
        let mut meta = Vec::new();
        if let Some(r) = self.popularity_rank {
            meta.push(format!("Rank: #{r}"));
        }
        if let Some(r) = self.rating {
            meta.push(format!("Rating: {r:.2}"));
        }
        if let Some(n) = self.bookmark_count {
            meta.push(format!(
                "Bookmarks: {}",
                if n >= 1_000_000 {
                    format!("{:.1}M", n as f64 / 1_000_000.0)
                } else if n >= 1_000 {
                    format!("{:.1}K", n as f64 / 1_000.0)
                } else {
                    n.to_string()
                }
            ));
        }
        out.push_str(&meta.join(" • "));
        if let Some(text) = self.description.as_deref().map(plain_text).filter(|t| !t.is_empty()) {
            if !out.is_empty() {
                out.push_str("\n\n");
            }
            out.push_str(&text);
        }
        let alts: Vec<String> = self
            .alternative_titles
            .as_deref()
            .map(|a| if a.contains('•') { a.split('•').map(str::to_string).collect::<Vec<_>>() } else { a.split(',').map(str::to_string).collect() })
            .unwrap_or_default()
            .iter()
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect();
        if !alts.is_empty() {
            if !out.is_empty() {
                out.push_str("\n\n");
            }
            out.push_str("Alternative Titles:\n");
            out.push_str(&alts.iter().map(|t| format!("- {t}")).collect::<Vec<_>>().join("\n"));
        }
        out
    }
}

/// `Jsoup.parseBodyFragment(html).text()`.
fn plain_text(html: &str) -> String {
    Document::parse(html, "").root().text()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UrlDto {
    public_url: String,
    series_slug: String,
}

#[derive(Deserialize, serde::Serialize, Clone)]
struct GenreDto {
    name: String,
    slug: String,
}

#[derive(Deserialize)]
struct AvailableGenres {
    #[serde(rename = "availableGenres")]
    genres: Vec<GenreDto>,
}

#[derive(Deserialize)]
struct Creators {
    artists: Vec<String>,
    authors: Vec<String>,
}

#[derive(Deserialize, serde::Serialize)]
struct FiltersData {
    genres: Vec<GenreDto>,
    authors: Vec<String>,
    artists: Vec<String>,
}

#[derive(Deserialize)]
struct ChapterListDto {
    chapters: Option<Vec<ChapterDto>>,
}

#[derive(Deserialize)]
struct ChapterDto {
    number: f32,
    title: Option<String>,
    #[serde(default)]
    created_at: String,
    #[serde(default, alias = "is_locked")]
    is_premium: bool,
    early_access_until: Option<String>,
    series_slug: Option<String>,
}

impl ChapterDto {
    /// Premium, or still in early access (readable only by subscribers until then).
    fn is_locked(&self, now: i64) -> bool {
        self.is_premium || self.early_access_until.as_deref().and_then(parse_iso).is_some_and(|until| until > now)
    }

    fn to_chapter(&self, random_manga_slug: &str, manga_slug: &str, now: i64) -> Chapter {
        let number = self.number.to_string();
        let mut name = String::new();
        if self.is_locked(now) {
            name.push_str("🔒 ");
        }
        name.push_str(&format!("Chapter {number}"));
        if let Some(t) = &self.title {
            name.push_str(&format!(" - {t}"));
        }
        Chapter {
            url: format!("/series/{}/chapter/{number}", self.series_slug.as_deref().unwrap_or(manga_slug)),
            name,
            chapter_number: self.number,
            date_upload: parse_iso(&self.created_at).unwrap_or(0),
            memo: Some(json!({ "mangaSlug": random_manga_slug })),
            ..Default::default()
        }
    }
}

#[derive(Deserialize)]
struct PageListDto {
    #[serde(default)]
    pages: Vec<PageDto>,
}

#[derive(Deserialize)]
struct PageDto {
    url: String,
    tiles: Option<Vec<u32>>,
    tile_cols: Option<u32>,
    tile_rows: Option<u32>,
}

#[derive(Deserialize)]
struct PremiumPageList {
    data: PremiumChapter,
}

#[derive(Deserialize)]
struct PremiumChapter {
    chapter: PageListDto,
}

export_source!(AsuraScans);

#[cfg(test)]
mod tests;
