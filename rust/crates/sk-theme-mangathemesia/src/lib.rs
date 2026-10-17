//! MangaThemesia (formerly WPMangaStream / WPMangaReader): the WordPress manga theme used by a long list of scanlation
//! sites. Lists are search pages (`/manga/?page=2&order=popular`), details and chapters sit in one HTML page, and the
//! chapter page holds the images or a `ts_reader.run({...})` script. A site is a type that implements [`Theme`] with
//! its name and address and overrides only what differs.
//!
//! Port of the `mangathemesia` multisrc of the Kotlin extensions. Not ported: the "view count" ping the Kotlin theme
//! sent to the site's `admin-ajax.php` on every page it opened (it only feeds the site's own statistics and cost an
//! extra request), and the interface languages other than English.

use base64::Engine;
use fancy_regex::Regex;
use serde::Deserialize;
use sk_source_sdk::html::{Document, El, Elements};
use sk_source_sdk::*;
use std::marker::PhantomData;

pub trait Theme: 'static {
    const NAME: &'static str;
    const LANG: &'static str = "en";
    const BASE_URL: &'static str;
    const VERSION_ID: u32 = 1;
    const RATE_LIMIT: Option<RateLimit> = None;

    /// Where the title list lives, with a leading slash.
    const MANGA_URL_DIRECTORY: &'static str = "/manga";
    const PROJECT_PAGE: &'static str = "/project";
    const HAS_PROJECT_PAGE: bool = false;
    /// `java.time` pattern of the chapter dates.
    const DATE_PATTERN: &'static str = "MMMM d, yyyy";

    const LIST_SELECTOR: &'static str = ".utao .uta .imgu, .listupd .bs .bsx, .listo .bs .bsx";
    const NEXT_PAGE_SELECTOR: &'static str = "div.pagination .next, div.hpage .r";
    const DETAILS_SELECTOR: &'static str = "div.bigcontent, div.animefull, div.main-info, div.postbody";
    const TITLE_SELECTOR: &'static str = ".entry-title, .ts-breadcrumb li:last-child span";
    const DESCRIPTION_SELECTOR: &'static str = ".desc, .entry-content[itemprop=description]";
    const THUMBNAIL_SELECTOR: &'static str = ".infomanga > div[itemprop=image] img, .thumb img";
    const CHAPTER_LIST_SELECTOR: &'static str = "div.bxcl li, div.cl li, #chapterlist li, ul li:has(div.chbox):has(div.eph-num)";
    const PAGE_SELECTOR: &'static str = "div#readerarea img";

    fn artist_selector() -> String {
        expand(
            ".infotable tr:contains(%s) td:last-child, .tsinfo .imptdt:contains(%s) i, .fmed b:contains(%s)+span, span:contains(%s)",
            &["artist", "Artiste", "Artista", "الرسام", "الناشر", "İllüstratör", "Çizer", "Sanatçı", "นักวาด"],
        )
    }

    fn author_selector() -> String {
        expand(
            ".infotable tr:contains(%s) td:last-child, .tsinfo .imptdt:contains(%s) i, .fmed b:contains(%s)+span, span:contains(%s)",
            &["Author", "Auteur", "autor", "المؤلف", "Mangaka", "seniman", "Pengarang", "Yazar", "ผู้แต่ง", "นักเขียน"],
        )
    }

    fn alt_name_selector() -> String {
        format!(
            ".alternative, .wd-full:contains(alt) span, .alter, .seriestualt, {}",
            expand(".infotable tr:contains(%s) td:last-child", &["Alternative", "Alternatif", "الأسماء الثانوية"])
        )
    }

    fn genre_selector() -> String {
        format!("div.gnr a, .mgen a, .seriestugenre a, {}", expand("span:contains(%s)", &["genre", "التصنيف"]))
    }

    fn type_selector() -> String {
        format!(
            "{}, a[href*=type\\=]",
            expand(
                ".infotable tr:contains(%s) td:last-child, .tsinfo .imptdt:contains(%s) i, .tsinfo .imptdt:contains(%s) a, .fmed b:contains(%s)+span, span:contains(%s) a",
                &["type", "ประเภท", "النوع", "tipe", "Türü"],
            )
        )
    }

    fn status_selector() -> String {
        expand(
            ".infotable tr:contains(%s) td:last-child, .tsinfo .imptdt:contains(%s) i, .fmed b:contains(%s)+span span:contains(%s)",
            &["status", "Statut", "Durum", "連載状況", "Estado", "الحالة", "حالة العمل", "สถานะ", "stato", "Statüsü"],
        )
    }
}

/// `template` once per word, with each `%s` replaced, joined by commas (a selector group).
fn expand(template: &str, words: &[&str]) -> String {
    words.iter().map(|w| template.replace("%s", w)).collect::<Vec<_>>().join(", ")
}

pub struct Site<T>(PhantomData<T>);

impl<T> Default for Site<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

const STATUS: [(&str, &str); 5] = [("All", ""), ("Ongoing", "ongoing"), ("Completed", "completed"), ("Hiatus", "hiatus"), ("Dropped", "dropped")];
const TYPES: [(&str, &str); 5] = [("All", ""), ("Manga", "Manga"), ("Manhwa", "Manhwa"), ("Manhua", "Manhua"), ("Comic", "Comic")];
const ORDERS: [(&str, &str); 6] = [
    ("Default", ""),
    ("A-Z", "title"),
    ("Z-A", "titlereverse"),
    ("Latest Update", "update"),
    ("Latest Added", "latest"),
    ("Popular", "popular"),
];
const PROJECT: [(&str, &str); 2] = [("Show all manga", ""), ("Show only project manga", "project-filter-on")];

fn base<T: Theme>() -> String {
    base_url(T::BASE_URL)
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
            supports_related: true,
            rate_limit: T::RATE_LIMIT,
            ..Default::default()
        }
    }

    fn popular(&self, page: u32) -> Result<MangaPage> {
        self.list(page, "", &[("order", "popular")], None)
    }

    fn latest(&self, page: u32) -> Result<MangaPage> {
        self.list(page, "", &[("order", "update")], None)
    }

    fn search(&self, page: u32, query: &str, filters: &Filters) -> Result<MangaPage> {
        let mut params: Vec<(String, String)> = Vec::new();
        let author = filters.text("author");
        if !author.is_empty() {
            params.push(("author".into(), author.into()));
        }
        let year = filters.text("year");
        if !year.is_empty() {
            params.push(("yearx".into(), year.into()));
        }
        params.push(("status".into(), filters.select_value("status", &STATUS).into()));
        params.push(("type".into(), filters.select_value("type", &TYPES).into()));
        params.push(("order".into(), filters.select_value("order", &ORDERS).into()));
        for g in filters.included("genres") {
            params.push(("genre[]".into(), g.into()));
        }
        for g in filters.excluded("genres") {
            params.push(("genre[]".into(), format!("-{g}")));
        }
        let project = T::HAS_PROJECT_PAGE && filters.select_value("project", &PROJECT) == "project-filter-on";
        let pairs: Vec<(&str, &str)> = params.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        self.list(page, query, &pairs, project.then_some(T::PROJECT_PAGE))
    }

    fn filters(&self, data: Option<&Value>) -> Vec<FilterDef> {
        let mut f = vec![
            FilterDef::Separator,
            FilterDef::text("author", "Author"),
            FilterDef::text("year", "Year"),
            FilterDef::select("status", "Status", &STATUS, 0),
            FilterDef::select("type", "Type", &TYPES, 0),
            FilterDef::select("order", "Sort By", &ORDERS, 0),
        ];
        let genres: Vec<GenreData> = data.and_then(|d| serde_json::from_value(d.clone()).ok()).unwrap_or_default();
        if !genres.is_empty() {
            let items: Vec<(String, String)> = genres.into_iter().map(|g| (g.name, g.value)).collect();
            f.push(FilterDef::header("Genre exclusion is not available for all sources"));
            f.push(FilterDef::tristates("genres", "Genre", &items));
        }
        if T::HAS_PROJECT_PAGE {
            f.push(FilterDef::Separator);
            f.push(FilterDef::header("NOTE: Can't be used with other filter!"));
            f.push(FilterDef::header(&format!("{} Project List page", T::NAME)));
            f.push(FilterDef::select("project", "Filter Project", &PROJECT, 0));
        }
        f
    }

    fn filter_data(&self) -> Result<Option<Value>> {
        let doc = get(&format!("{}{}", base::<T>(), T::MANGA_URL_DIRECTORY)).send()?.html();
        let genres: Vec<GenreData> = doc
            .select_first("ul.genrez")
            .map(|ul| {
                ul.select("li")
                    .iter()
                    .filter_map(|li| {
                        Some(GenreData {
                            name: li.select_first("label")?.text(),
                            value: li.select_first("input[type=checkbox]")?.attr("value"),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(Some(serde_json::to_value(genres).map_err(|e| e.to_string())?))
    }

    fn details(&self, manga: &Manga) -> Result<Manga> {
        parse_details::<T>(&get(&manga_url::<T>(manga)).send()?.html())
    }

    fn chapters(&self, manga: &Manga) -> Result<Vec<Chapter>> {
        Ok(parse_chapters::<T>(&get(&manga_url::<T>(manga)).send()?.html()))
    }

    /// One page holds both, so fetch it once.
    fn update(&self, manga: &Manga, fetch_details: bool, fetch_chapters: bool) -> Result<Update> {
        let doc = get(&manga_url::<T>(manga)).send()?.html();
        Ok(Update {
            manga: if fetch_details { Some(parse_details::<T>(&doc)?) } else { None },
            chapters: if fetch_chapters { Some(parse_chapters::<T>(&doc)) } else { None },
        })
    }

    fn by_url(&self, url: &str) -> Result<Option<Manga>> {
        let host = |u: &str| Url::parse(u).ok().and_then(|u| u.host_str().map(str::to_string));
        if host(url) != host(&base::<T>()) {
            return Ok(None);
        }
        let manga = parse_details::<T>(&get(url).send()?.html())?;
        Ok(Some(manga)).map(|m| m.filter(|m| !m.title.is_empty()))
    }

    fn related(&self, manga: &Manga) -> Result<Vec<Manga>> {
        let doc = get(&manga_url::<T>(manga)).send()?.html();
        Ok(parse_list::<T>(&doc).mangas.into_iter().filter(|m| !m.title.is_empty()).collect())
    }

    fn pages(&self, chapter: &Chapter) -> Result<Vec<Page>> {
        let resp = get(&absolute(&base::<T>(), &chapter.url)).send()?;
        let doc = resp.html();
        let from_html: Vec<Page> = doc
            .select(T::PAGE_SELECTOR)
            .iter()
            .map(|img| img_attr(img))
            .filter(|u| !u.is_empty())
            .enumerate()
            .map(|(i, u)| Page::new(i as u32, u))
            .collect();
        // Some sites load the pages with JavaScript.
        if !from_html.is_empty() {
            return Ok(from_html);
        }
        // `ts_reader.run({` in base64
        let script = doc
            .select_first("script[src^=data:text/javascript;base64,dHNfcmVhZGVyLnJ1bih7]")
            .and_then(|s| s.attr("src").split_once("base64,").map(|x| x.1.to_string()))
            .and_then(|b| base64::engine::general_purpose::STANDARD.decode(b.trim()).ok())
            .map(|b| String::from_utf8_lossy(&b).into_owned())
            .unwrap_or_else(|| resp.body.clone());
        let images = Regex::new(r#"["']?(?:images|imageUrls)["']?\s*[:=]\s*(\[.*?])"#)
            .ok()
            .and_then(|re| re.captures(script.as_str()).ok().flatten().map(|c| c[1].to_string()))
            .and_then(|json| serde_json::from_str::<Vec<String>>(&json).ok())
            .unwrap_or_default();
        let base = base::<T>();
        Ok(images
            .into_iter()
            .enumerate()
            .map(|(i, u)| Page::new(i as u32, if u.starts_with('/') { format!("{base}{u}") } else { u }))
            .collect())
    }
}

impl<T: Theme> Site<T> {
    /// The title list with `page`, a search `query` and extra `params`; `project` swaps the first path segment.
    fn list(&self, page: u32, query: &str, params: &[(&str, &str)], project: Option<&str>) -> Result<MangaPage> {
        let dir = project.unwrap_or(T::MANGA_URL_DIRECTORY).trim_start_matches('/');
        let mut url = format!("{}/{dir}/?", base::<T>());
        if !query.is_empty() {
            url.push_str(&format!("title={}&", url_encode(query)));
        }
        url.push_str(&format!("page={page}"));
        for (k, v) in params {
            url.push_str(&format!("&{}={}", url_encode(k).replace("%5B%5D", "[]"), url_encode(v)));
        }
        Ok(parse_list::<T>(&get(&url).send()?.html()))
    }
}

fn manga_url<T: Theme>(manga: &Manga) -> String {
    absolute(&base::<T>(), &manga.url)
}

#[derive(serde::Serialize, Deserialize)]
struct GenreData {
    name: String,
    value: String,
}

// ------------------------------------------------------------------ lists

fn parse_list<T: Theme>(doc: &Document) -> MangaPage {
    let mangas = doc
        .select(T::LIST_SELECTOR)
        .iter()
        .map(|el| Manga {
            thumbnail_url: Some(img_attr_first(&el.select("img"))).filter(|s| !s.is_empty()),
            title: el.select("a").attr("title"),
            url: path_of(&el.select("a").attr("href")),
            ..Default::default()
        })
        .collect();
    MangaPage { mangas, has_next: doc.select_first(T::NEXT_PAGE_SELECTOR).is_some() }
}

/// `data-lazy-src`, `data-src`, `data-cfsrc`, then `src`, as an absolute address.
fn img_attr(img: &El) -> String {
    for name in ["data-lazy-src", "data-src", "data-cfsrc"] {
        if img.has_attr(name) {
            return img.abs_url(name);
        }
    }
    img.abs_url("src")
}

fn img_attr_first(images: &Elements) -> String {
    images.first().map(img_attr).unwrap_or_default()
}

// ------------------------------------------------------------------ details

fn parse_details<T: Theme>(doc: &Document) -> Result<Manga> {
    let mut manga = Manga { url: path_of(doc.location()), initialized: true, ..Default::default() };
    let Some(details) = doc.select_first(T::DETAILS_SELECTOR) else { return Ok(manga) };

    manga.title = details.select_first(T::TITLE_SELECTOR).ok_or("the page has no title (the site changed?)")?.text();
    manga.artist = details.select_first(&T::artist_selector()).and_then(|e| remove_placeholder(e.own_text()));
    manga.author = details.select_first(&T::author_selector()).and_then(|e| remove_placeholder(e.own_text()));
    manga.description = details.select_first(T::DESCRIPTION_SELECTOR).and_then(|e| e.text_or_none());

    // Alternative names go into the description.
    if let Some(alt) = details.select_first(&T::alt_name_selector()).map(|e| e.own_text()).filter(|a| !a.trim().is_empty()) {
        let names = alt.split(['|', '/', '•', ',', ';']).map(|n| format!("- {}", n.trim())).collect::<Vec<_>>().join("\n");
        let head = manga.description.take().map(|d| format!("{d}\n\n")).unwrap_or_default();
        manga.description = Some(format!("{head}{}", format!("Alternative Names: \n{names}").trim()));
    }

    let mut genres: Vec<String> = details.select(&T::genre_selector()).each_text();
    // The series type (manga/manhwa/manhua/other) is also a genre.
    if let Some(t) = details.select_first(&T::type_selector()).map(|e| e.own_text()).filter(|t| !t.trim().is_empty()) {
        genres.push(t);
    }
    manga.genre = genres.iter().map(|g| capitalize(g.trim())).filter(|g| !g.is_empty()).collect();

    manga.status = parse_status(&details.select_first(&T::status_selector()).map(|e| e.text()).unwrap_or_default());
    manga.thumbnail_url = Some(img_attr_first(&details.select(T::THUMBNAIL_SELECTOR))).filter(|s| !s.is_empty());
    Ok(manga)
}

fn remove_placeholder(s: String) -> Option<String> {
    let t = s.trim();
    (!(t.is_empty() || matches!(t, "-" | "N/A" | "n/a" | "Unknown"))).then_some(s)
}

/// Lower case, then the first letter upper case ("SLICE OF life" -> "Slice of life").
fn capitalize(s: &str) -> String {
    let lower = s.to_lowercase();
    let mut c = lower.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}

fn parse_status(s: &str) -> u8 {
    let s = s.to_lowercase();
    let any = |words: &[&str]| words.iter().any(|w| s.contains(w));
    if any(&[
        "مستمرة", "en curso", "ongoing", "on going", "new season", "mass released", "ativo", "en cours", "en cours de publication", "đang tiến hành",
        "em lançamento", "онгоінг", "publishing", "devam ediyor", "em andamento", "in corso", "güncel", "berjalan", "продолжается", "updating",
        "lançando", "in arrivo", "emision", "en emision", "مستمر", "curso", "en marcha", "publicandose", "publicando", "连载中", "devam etmekte", "連載中",
    ]) {
        1
    } else if any(&[
        "completed", "completo", "complété", "fini", "achevé", "terminé", "tamamlandı", "đã hoàn thành", "hoàn thành", "مكتملة", "завершено", "finished",
        "finalizad", "completata", "one-shot", "bitti", "tamat", "completado", "concluído", "完結", "concluido", "已完结", "bitmiş",
    ]) {
        2
    } else if any(&["canceled", "cancelled", "cancelado", "cancellato", "cancelados", "dropped", "discontinued", "abandonné"]) {
        5
    } else if any(&["hiatus", "on hold", "season end", "pausado", "en espera", "en pause", "en attente", "hiato"]) {
        6
    } else {
        0
    }
}

// ------------------------------------------------------------------ chapters

fn parse_chapters<T: Theme>(doc: &Document) -> Vec<Chapter> {
    let mut chapters: Vec<Chapter> = doc
        .select(T::CHAPTER_LIST_SELECTOR)
        .iter()
        .filter_map(|el| {
            let links = el.select("a");
            let name = Some(el.select(".lch a, .chapternum").text())
                .filter(|n| !n.trim().is_empty())
                .or_else(|| links.first().map(|a| a.text()))?;
            Some(Chapter {
                url: path_of(&links.attr("href")),
                name,
                date_upload: el
                    .select_first(".chapterdate")
                    .map(|d| d.text())
                    .and_then(|t| parse_date(&t, T::DATE_PATTERN, T::LANG).or_else(|| parse_date(&t, T::DATE_PATTERN, "en")))
                    .unwrap_or(0),
                ..Default::default()
            })
        })
        .collect();

    // A site that gives no chapter dates at least gets one for the newest, from "Updated On".
    if chapters.first().is_some_and(|c| c.date_upload == 0) {
        let updated = doc
            .select(".listinfo time[itemprop=dateModified], .fmed:contains(update) time, span:contains(update) time")
            .attr("datetime");
        if let Some(ms) = parse_date(&updated, "yyyy-MM-dd", "en") {
            chapters[0].date_upload = ms;
        }
    }
    chapters
}

#[cfg(test)]
mod tests;
