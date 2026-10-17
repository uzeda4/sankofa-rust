//! MangaFire, written in Rust. Port of the Kotlin extension `all.mangafire` (one source per language: en, es, es-419,
//! fr, ja, pt, pt-BR). The site is a JSON API whose every request carries a signature of its own sorted query (`vrf`,
//! see [`vrf`]); chapters are listed per language, or as volumes.
//!
//! Not ported: the "shape-selecting captcha" solver (it runs OpenCV in a WebView). When the site asks for one, the source
//! says so; after solving it by hand in the WebView the app's cookie lets the requests through.

mod vrf;

use fancy_regex::Regex;
use serde::Deserialize;
use sk_source_sdk::html::Document;
use sk_source_sdk::*;
use std::cell::RefCell;
use std::collections::HashMap;

const BASE: &str = "https://mangafire.to";
const CONTENT_RATING_PREF: &str = "pref_content_rating";
const SHOW_AS_VOLUMES: &str = "show_as_volumes";
const MERGE_CHAPTERS: &str = "merge_chapters";
const PREFER_OFFICIAL: &str = "prefer_official";
const CONTENT_RATINGS: [&str; 4] = ["safe", "suggestive", "erotica", "pornographic"];

const TYPES: [(&str, &str); 4] = [("Manga", "manga"), ("Manhwa", "manhwa"), ("Manhua", "manhua"), ("Other", "other")];
const STATUSES: [(&str, &str); 5] = [
    ("Releasing", "releasing"),
    ("Finished", "finished"),
    ("On Hiatus", "on_hiatus"),
    ("Discontinued", "discontinued"),
    ("Not Yet Released", "not_yet_released"),
];
const SORTS: [(&str, &str); 12] = [
    ("Latest update", "chapter_updated_at:desc"),
    ("Best match", "relevance:desc"),
    ("Recently added", "created_at:desc"),
    ("Title (A–Z)", "title:asc"),
    ("Title (Z–A)", "title:desc"),
    ("Year (newest)", "year:desc"),
    ("Year (oldest)", "year:asc"),
    ("Highest rated", "score:desc"),
    ("Most viewed · 7 days", "views_7d:desc"),
    ("Most viewed · 30 days", "views_30d:desc"),
    ("Most viewed · all time", "views_total:desc"),
    ("Most followed", "follows_total:desc"),
];
const GENRE_MODES: [(&str, &str); 2] = [("AND", "and"), ("OR", "or")];

const GENRES: [(&str, &str); 52] = [
    ("Action", "1"), ("Adult", "268929"), ("Adventure", "78"), ("Avant Garde", "3"), ("Boys Love", "4"), ("Comedy", "5"),
    ("Crime", "268921"), ("Demons", "77"), ("Drama", "6"), ("Ecchi", "7"), ("Fantasy", "79"), ("Girls Love", "9"),
    ("Gourmet", "10"), ("Harem", "11"), ("Hentai", "268930"), ("Historical", "268922"), ("Horror", "530"), ("Isekai", "13"),
    ("Iyashikei", "531"), ("Josei", "15"), ("Kids", "532"), ("Magic", "539"), ("Magical Girls", "268923"),
    ("Mahou Shoujo", "533"), ("Martial Arts", "534"), ("Mature", "268931"), ("Mecha", "19"), ("Medical", "268924"),
    ("Military", "535"), ("Music", "21"), ("Mystery", "22"), ("Parody", "23"), ("Philosophical", "268925"),
    ("Psychological", "536"), ("Reverse Harem", "25"), ("Romance", "26"), ("School", "73"), ("Sci-Fi", "28"),
    ("Seinen", "537"), ("Shoujo", "30"), ("Shounen", "31"), ("Slice of Life", "538"), ("Smut", "268932"), ("Space", "33"),
    ("Sports", "34"), ("Super Power", "75"), ("Superhero", "268926"), ("Supernatural", "76"), ("Suspense", "37"),
    ("Thriller", "38"), ("Tragedy", "268927"), ("Vampire", "39"),
];

const THEMES: [(&str, &str); 38] = [
    ("Aliens", "268933"), ("Animals", "268934"), ("Cooking", "268935"), ("Crossdressing", "268936"),
    ("Delinquents", "268937"), ("Demons", "268938"), ("Genderswap", "268939"), ("Ghosts", "268940"), ("Gyaru", "268941"),
    ("Harem", "268942"), ("Incest", "268943"), ("Loli", "268944"), ("Mafia", "268945"), ("Magic", "268946"),
    ("Martial Arts", "268947"), ("Military", "268948"), ("Monster Girls", "268949"), ("Monsters", "268950"),
    ("Music", "268951"), ("Ninja", "268952"), ("Office Workers", "268953"), ("Police", "268954"),
    ("Post-Apocalyptic", "268955"), ("Reincarnation", "268956"), ("Reverse Harem", "268957"), ("Samurai", "268958"),
    ("School Life", "268959"), ("Shota", "268960"), ("Supernatural", "268961"), ("Survival", "268962"),
    ("Time Travel", "268963"), ("Traditional Games", "268964"), ("Vampires", "268965"), ("Video Games", "268966"),
    ("Villainess", "268967"), ("Virtual Reality", "268968"), ("Zombies", "268969"), ("Wuxia", "268928"),
];

thread_local! {
    /// Author names already looked up (the site's id for a name), so a search page does not repeat it.
    static AUTHOR_IDS: RefCell<HashMap<String, Option<String>>> = RefCell::new(HashMap::new());
}

/// `es-419` and `pt-BR` are spelled `es-la` and `pt-br` in the API.
fn lang_code() -> String {
    match lang().as_str() {
        "es-419" => "es-la".into(),
        "pt-BR" => "pt-br".into(),
        l => l.into(),
    }
}

#[derive(Default)]
pub struct MangaFire;

impl Source for MangaFire {
    fn info(&self) -> Info {
        let ratings: Vec<(&str, &str)> = CONTENT_RATINGS.iter().map(|r| (title_case(r), *r)).collect();
        Info {
            name: "MangaFire".into(),
            lang: lang(),
            base_url: BASE.into(),
            supports_latest: true,
            rate_limit: Some(RateLimit { permits: 2, period_ms: 1000, host: String::new() }),
            prefs: vec![
                PrefDef::multi_list(CONTENT_RATING_PREF, "Content Rating (All by default)", &ratings, &[]),
                PrefDef::switch(SHOW_AS_VOLUMES, "Prefer Volume Release", "Requires Chapter List Refresh to Apply", false),
                PrefDef::switch(MERGE_CHAPTERS, "Merge duplicate chapters", "Requires Chapter List Refresh to Apply", false),
                PrefDef::switch(PREFER_OFFICIAL, "Prefer official chapters", "Only when merging duplicates", true),
            ],
            ..Default::default()
        }
    }

    fn popular(&self, page: u32) -> Result<MangaPage> {
        titles(&[("order[views_30d]", "desc".into()), ("page", page.to_string()), ("limit", "50".into())], &content_rating_params())
    }

    fn latest(&self, page: u32) -> Result<MangaPage> {
        titles(&[("order[chapter_updated_at]", "desc".into()), ("page", page.to_string()), ("limit", "50".into())], &content_rating_params())
    }

    fn search(&self, page: u32, query: &str, filters: &Filters) -> Result<MangaPage> {
        let mut params: Vec<(String, String)> = Vec::new();
        let author = filters.text("author");
        if !author.is_empty() {
            match author_id(author)? {
                Some(id) => params.push(("authors[]".into(), id)),
                None => return Ok(MangaPage::default()),
            }
        }
        if !query.trim().is_empty() {
            params.push(("keyword".into(), query.trim().into()));
        }
        params.push(("page".into(), page.to_string()));
        params.push(("limit".into(), "50".into()));

        for r in filters.included("content_rating") {
            params.push(("content_rating[]".into(), r.into()));
        }
        for t in filters.included("types") {
            params.push(("types[]".into(), t.into()));
        }
        let mode = filters.select_value("genre_mode", &GENRE_MODES);
        params.push(("genres_mode".into(), mode.into()));
        params.push(("theme_mode".into(), mode.into()));
        for g in filters.included("genres") {
            params.push(("genres_in[]".into(), g.into()));
        }
        for g in filters.excluded("genres") {
            params.push(("genres_ex[]".into(), g.into()));
        }
        for t in filters.included("themes") {
            params.push(("theme_ids[]".into(), t.into()));
        }
        for s in filters.included("statuses") {
            params.push(("statuses[]".into(), s.into()));
        }
        if let Some(n) = filters.text("min_chapters").parse::<i64>().ok().filter(|n| *n > 0) {
            params.push(("min_chap".into(), n.to_string()));
        }
        for key in ["year_from", "year_to"] {
            if let Ok(y) = filters.text(key).parse::<i64>() {
                params.push((key.into(), y.to_string()));
            }
        }
        let sort = filters.select_value("sort", &SORTS);
        let (field, dir) = sort.split_once(':').unwrap_or(("chapter_updated_at", "desc"));
        params.push((format!("order[{field}]"), dir.into()));
        titles(&params.iter().map(|(k, v)| (k.as_str(), v.clone())).collect::<Vec<_>>(), &[])
    }

    fn filters(&self, _data: Option<&Value>) -> Vec<FilterDef> {
        let chosen = pref_list(CONTENT_RATING_PREF);
        let boxes = |key: &str, name: &str, items: &[(&str, &str)], on: &dyn Fn(&str) -> bool| FilterDef::Group {
            key: key.into(),
            name: name.into(),
            items: items.iter().map(|(label, v)| FilterDef::Checkbox { key: v.to_string(), name: label.to_string(), default: on(v) }).collect(),
        };
        let ratings: Vec<(&str, &str)> = CONTENT_RATINGS.iter().map(|r| (title_case(r), *r)).collect();
        let genres: Vec<(String, String)> = GENRES.iter().map(|(l, v)| (l.to_string(), v.to_string())).collect();
        vec![
            boxes("content_rating", "Content Rating", &ratings, &|v| chosen.iter().any(|c| c == v)),
            FilterDef::Separator,
            boxes("types", "Type", &TYPES, &|_| false),
            FilterDef::Separator,
            FilterDef::select("genre_mode", "Genre and theme match mode", &GENRE_MODES, 0),
            FilterDef::tristates("genres", "Genres", &genres),
            FilterDef::Separator,
            boxes("themes", "Themes", &THEMES, &|_| false),
            FilterDef::Separator,
            boxes("statuses", "Status", &STATUSES, &|_| false),
            FilterDef::Separator,
            FilterDef::text("author", "Author / Artist"),
            FilterDef::text("year_from", "Release year (From)"),
            FilterDef::text("year_to", "Release year (To)"),
            FilterDef::text("min_chapters", "Minimum chapters"),
            FilterDef::Separator,
            FilterDef::select("sort", "Sort by", &SORTS, 1),
        ]
    }

    fn by_url(&self, url: &str) -> Result<Option<Manga>> {
        let u = Url::parse(url).map_err(|e| e.to_string())?;
        let seg: Vec<&str> = u.path_segments().map(|s| s.collect()).unwrap_or_default();
        let same_host = u.host_str() == Url::parse(BASE).ok().and_then(|b| b.host_str().map(str::to_string)).as_deref();
        if !same_host || seg.len() < 2 {
            return Ok(None);
        }
        fetch_details(&hid(seg[1])).map(Some)
    }

    fn update(&self, manga: &Manga, fetch_details_: bool, fetch_chapters: bool) -> Result<Update> {
        let id = hid(&manga.url);
        let details = if fetch_details_ { Some(fetch_details(&id)?) } else { None };
        let chapters = if fetch_chapters { Some(chapters_of(manga, &id)?) } else { None };
        Ok(Update { manga: details, chapters })
    }

    fn details(&self, manga: &Manga) -> Result<Manga> {
        fetch_details(&hid(&manga.url))
    }

    fn chapters(&self, manga: &Manga) -> Result<Vec<Chapter>> {
        chapters_of(manga, &hid(&manga.url))
    }

    fn pages(&self, chapter: &Chapter) -> Result<Vec<Page>> {
        let url = Url::parse(&absolute(BASE, &chapter.url)).map_err(|e| e.to_string())?;
        let seg: Vec<&str> = url.path_segments().map(|s| s.collect()).unwrap_or_default();
        let last = seg.last().copied().unwrap_or_default();
        let path = if seg.contains(&"volume") {
            format!("/api/volumes/{last}")
        } else {
            let id: i64 = last.split('-').next().and_then(|n| n.parse().ok()).ok_or("Refresh manga")?;
            format!("/api/chapters/{id}")
        };
        let data: PagesResponse = api(&path, &[])?.json()?;
        Ok(data.data.pages.into_iter().enumerate().map(|(i, p)| Page::new(i as u32, p.url)).collect())
    }
}

fn title_case(s: &str) -> &'static str {
    match s {
        "safe" => "Safe",
        "suggestive" => "Suggestive",
        "erotica" => "Erotica",
        _ => "Pornographic",
    }
}

fn content_rating_params() -> Vec<(&'static str, String)> {
    pref_list(CONTENT_RATING_PREF).into_iter().filter(|r| CONTENT_RATINGS.contains(&r.as_str())).map(|r| ("content_rating[]", r)).collect()
}

/// The id of a title in an address: `/title/ab12.xyz` and `ab12-slug` both give the part the API knows.
fn hid(url: &str) -> String {
    let last = url.trim_end_matches('/').rsplit('/').next().unwrap_or_default();
    if let Some((_, after)) = last.rsplit_once('.') {
        after.to_string()
    } else if let Some((before, _)) = last.split_once('-') {
        before.to_string()
    } else {
        last.to_string()
    }
}

// ------------------------------------------------------------------ requests

/// `<base><path>?<params>&vrf=<signature>`, with the parameters sorted by name as the signature expects.
fn signed_url(path: &str, params: &[(&str, String)]) -> String {
    let mut sorted: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
    sorted.sort_by(|a, b| a.0.cmp(b.0));

    let mut signed = path.strip_prefix("/api").unwrap_or(path).to_string();
    if !sorted.is_empty() {
        signed.push('?');
        let (mut last_key, mut index) = ("", 0);
        let parts: Vec<String> = sorted
            .iter()
            .map(|(key, value)| {
                // keys that end in `[]` get their position: `genres_in[0]`, `genres_in[1]`
                let key = if key.ends_with("[]") {
                    if last_key != *key {
                        index = 0;
                    }
                    last_key = key;
                    let k = key.replace("[]", &format!("[{index}]"));
                    index += 1;
                    k
                } else {
                    key.to_string()
                };
                format!("{key}={value}")
            })
            .collect();
        signed.push_str(&parts.join("&"));
    }

    let mut url = format!("{BASE}{path}?");
    for (k, v) in &sorted {
        url.push_str(&format!("{}={}&", url_encode(k), url_encode(v)));
    }
    url.push_str(&format!("vrf={}", vrf::sign(&signed)));
    url
}

fn api(path: &str, params: &[(&str, String)]) -> Result<Response> {
    let resp = get(&signed_url(path, params)).header("Accept", "application/json").allow_errors().send()?;
    if (200..300).contains(&resp.status) {
        return Ok(resp);
    }
    if resp.status == 403 && resp.body.contains("captcha_required") {
        return Err("Shape-selecting captcha detected. Open in WebView to solve it manually, then try again.".into());
    }
    Err(format!("HTTP {} for {path}", resp.status))
}

fn titles(params: &[(&str, String)], extra: &[(&str, String)]) -> Result<MangaPage> {
    let all: Vec<(&str, String)> = params.iter().chain(extra).cloned().collect();
    let data: ApiResponse<MangaDto> = api("/api/titles", &all)?.json()?;
    Ok(MangaPage { mangas: data.items.into_iter().map(|m| m.to_manga()).collect(), has_next: data.meta.is_some_and(|m| m.has_next) })
}

fn author_id(name: &str) -> Result<Option<String>> {
    if let Some(hit) = AUTHOR_IDS.with(|c| c.borrow().get(name).cloned()) {
        return Ok(hit);
    }
    let tags: TagResponse = api("/api/tags", &[("keyword", name.to_string())])?.json()?;
    let id = tags.data.into_iter().find(|t| t.kind == "author" || t.kind == "artist").map(|t| t.id.to_string());
    AUTHOR_IDS.with(|c| {
        let mut c = c.borrow_mut();
        if c.len() >= 20 {
            c.clear();
        }
        c.insert(name.to_string(), id.clone());
    });
    Ok(id)
}

fn fetch_details(id: &str) -> Result<Manga> {
    let data: DetailsResponse = api(&format!("/api/titles/{id}"), &[])?.json()?;
    Ok(data.data.to_manga())
}

fn chapters_of(manga: &Manga, id: &str) -> Result<Vec<Chapter>> {
    let code = lang_code();
    let mut volumes_view = pref_bool(SHOW_AS_VOLUMES, false);
    let mut chapters: Vec<Chapter> = Vec::new();

    if volumes_view {
        let data: ApiResponse<VolumeDto> = api(&format!("/api/titles/{id}/volumes"), &[])?.json()?;
        if data.items.is_empty() {
            volumes_view = false; // no volumes: fall back to chapters
        } else {
            chapters.extend(data.items.into_iter().filter(|v| v.language == code).map(|v| v.to_chapter(&manga.url)));
        }
    }
    if !volumes_view {
        chapters.extend(all_chapters(id, &code)?.into_iter().map(|c| c.to_chapter(&manga.url, &code)));
        if pref_bool(MERGE_CHAPTERS, false) {
            let prefer_official = pref_bool(PREFER_OFFICIAL, true);
            let is_official = |c: &Chapter| c.scanlator.as_deref().unwrap_or_default().to_lowercase() == "official";
            // official first (or last), then one chapter per number, newest number first
            chapters.sort_by_key(|c| if prefer_official { !is_official(c) } else { is_official(c) });
            let mut seen: Vec<u32> = Vec::new();
            chapters.retain(|c| {
                let bits = c.chapter_number.to_bits();
                !seen.contains(&bits) && {
                    seen.push(bits);
                    true
                }
            });
            chapters.sort_by(|a, b| b.chapter_number.partial_cmp(&a.chapter_number).unwrap_or(std::cmp::Ordering::Equal));
        }
    }
    Ok(chapters)
}

/// Every chapter in the language: the first page says how many pages there are; the rest are fetched together.
fn all_chapters(id: &str, code: &str) -> Result<Vec<ChapterDto>> {
    let params = |page: u32| -> Vec<(&'static str, String)> {
        vec![("language", code.to_string()), ("sort", "number".into()), ("order", "desc".into()), ("page", page.to_string()), ("limit", "200".into())]
    };
    let path = format!("/api/titles/{id}/chapters");
    let first: ApiResponse<ChapterDto> = api(&path, &params(1))?.json()?;
    let last_page = first.meta.as_ref().map_or(1, |m| m.last_page);
    let mut all = first.items;
    if last_page > 1 {
        let requests = (2..=last_page).map(|p| get(&signed_url(&path, &params(p))).header("Accept", "application/json")).collect();
        for answer in send_all(requests) {
            all.extend(answer?.json::<ApiResponse<ChapterDto>>()?.items);
        }
    }
    Ok(all)
}

// ------------------------------------------------------------------ data

#[derive(Deserialize)]
struct ApiResponse<T> {
    #[serde(default = "Vec::new")]
    items: Vec<T>,
    meta: Option<Meta>,
}

#[derive(Deserialize)]
struct Meta {
    #[serde(rename = "lastPage", default = "one")]
    last_page: u32,
    #[serde(rename = "hasNext", default)]
    has_next: bool,
}

fn one() -> u32 {
    1
}

#[derive(Deserialize)]
struct TagResponse {
    #[serde(default)]
    data: Vec<TagDto>,
}

#[derive(Deserialize)]
struct TagDto {
    id: i64,
    #[serde(rename = "type")]
    kind: String,
}

#[derive(Deserialize)]
struct Poster {
    small: Option<String>,
    medium: Option<String>,
    large: Option<String>,
}

impl Poster {
    fn best(self) -> Option<String> {
        self.large.or(self.medium).or(self.small)
    }
}

#[derive(Deserialize)]
struct MangaDto {
    hid: String,
    slug: Option<String>,
    title: String,
    poster: Option<Poster>,
}

fn title_url(hid: &str, slug: &Option<String>) -> String {
    format!("/title/{hid}{}", slug.as_ref().map(|s| format!("-{s}")).unwrap_or_default())
}

impl MangaDto {
    fn to_manga(self) -> Manga {
        Manga { url: title_url(&self.hid, &self.slug), title: self.title, thumbnail_url: self.poster.and_then(Poster::best), ..Default::default() }
    }
}

#[derive(Deserialize)]
struct DetailsResponse {
    data: DetailsDto,
}

#[derive(Deserialize)]
struct Entity {
    title: String,
}

#[derive(Deserialize)]
struct DetailsDto {
    hid: String,
    slug: Option<String>,
    title: String,
    #[serde(rename = "type")]
    kind: Option<String>,
    status: Option<String>,
    poster: Option<Poster>,
    #[serde(rename = "synopsisHtml")]
    synopsis_html: Option<String>,
    authors: Option<Vec<Entity>>,
    artists: Option<Vec<Entity>>,
    genres: Option<Vec<Entity>>,
    themes: Option<Vec<Entity>>,
}

impl DetailsDto {
    fn to_manga(self) -> Manga {
        let join = |v: &Option<Vec<Entity>>| v.as_ref().map(|v| v.iter().map(|e| e.title.clone()).collect::<Vec<_>>().join(", "));
        let mut genre = Vec::new();
        if let Some(k) = &self.kind {
            let mut c = k.chars();
            genre.extend(c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()));
        }
        for list in [&self.genres, &self.themes] {
            genre.extend(list.iter().flatten().map(|e| e.title.clone()));
        }
        Manga {
            url: title_url(&self.hid, &self.slug),
            title: self.title.clone(),
            thumbnail_url: self.poster.and_then(Poster::best),
            author: join(&self.authors),
            artist: join(&self.artists),
            description: self.synopsis_html.as_deref().map(|h| Document::parse(h, "").root().text()),
            genre,
            status: match self.status.as_deref().map(str::to_lowercase).as_deref() {
                Some("releasing") => 1,
                Some("finished") => 2,
                Some("on_hiatus") => 6,
                Some("discontinued") => 5,
                _ => 0,
            },
            initialized: true,
            ..Default::default()
        }
    }
}

#[derive(Deserialize)]
struct VolumeDto {
    id: i64,
    number: i64,
    name: Option<String>,
    #[serde(rename = "chapterCount")]
    chapter_count: i64,
    language: String,
}

impl VolumeDto {
    fn to_chapter(&self, manga_url: &str) -> Chapter {
        let mut name = format!("Vol. {}", self.number);
        if let Some(n) = self.name.as_deref().filter(|n| !n.trim().is_empty()) {
            name.push_str(&format!(" - {n}"));
        }
        Chapter {
            url: format!("{manga_url}/volume/{}", self.id),
            name,
            chapter_number: self.number as f32,
            scanlator: Some(format!("{} chapters", self.chapter_count)),
            ..Default::default()
        }
    }
}

thread_local! {
    // compiled once: a pattern compiled per chapter costs more than everything else on a title with thousands of them
    static NUMBER_IN_NAME: Regex = Regex::new(r"(?i)\b(?:ch(?:\.|apter)?|ep(?:\.|isode)?)\s?(\d+(?:\.\d+)?)").expect("a valid pattern");
}

#[derive(Deserialize)]
struct ChapterDto {
    id: i64,
    number: f32,
    name: Option<String>,
    #[serde(rename = "createdAt")]
    created_at: Option<i64>,
    #[serde(rename = "type")]
    kind: Option<String>,
}

impl ChapterDto {
    fn to_chapter(&self, manga_url: &str, code: &str) -> Chapter {
        let n = self.number.to_string();
        let (number, name) = match self.name.as_deref().filter(|n| !n.trim().is_empty()) {
            None => (self.number, format!("Ch. {n}")),
            Some(name) => {
                // "Chapter 12.5: …": the number the title itself states wins over the listed one
                match NUMBER_IN_NAME.with(|re| re.captures(name).ok().flatten().and_then(|c| c[1].parse::<f32>().ok())) {
                    None => (self.number, format!("Ch. {n} - {name}")),
                    Some(found) => (found, name.to_string()),
                }
            }
        };
        Chapter {
            url: format!("{manga_url}/{}-chapter-{n}-{code}", self.id),
            name,
            chapter_number: number,
            scanlator: Some(self.kind.clone().unwrap_or_else(|| "Unknown".into())),
            date_upload: self.created_at.map_or(0, |t| t * 1000),
            memo: None,
        }
    }
}

#[derive(Deserialize)]
struct PagesResponse {
    data: ChapterData,
}

#[derive(Deserialize)]
struct ChapterData {
    pages: Vec<PageDto>,
}

#[derive(Deserialize)]
struct PageDto {
    url: String,
}

export_source!(MangaFire);

#[cfg(test)]
mod tests;
