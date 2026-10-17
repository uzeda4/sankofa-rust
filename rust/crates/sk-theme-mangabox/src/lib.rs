//! The MangaBox family (Mangakakalot, Manganato and kin): HTML lists, a JSON chapter API, and chapter pages whose
//! image addresses sit in a script. A site is a type that implements [`Theme`] with its name and address and
//! overrides only what differs; `export_source!(Site<MySite>)` turns it into a source.
//!
//! Port of the `mangabox` multisrc of the Kotlin extensions (`lib-multisrc/mangabox`).

mod genres;

use fancy_regex::Regex;
use serde::Deserialize;
use sk_source_sdk::html::{Document, El};
use sk_source_sdk::*;
use std::marker::PhantomData;

pub use genres::GENRES;

const MERGE_PREF: &str = "pref_merge_images";

/// What a MangaBox site must say; everything else has the family's defaults.
pub trait Theme: 'static {
    const NAME: &'static str;
    const LANG: &'static str = "en";
    const BASE_URL: &'static str;
    /// Part of the source id (see `Info::version_id`); an extension's id is what keeps a library attached.
    const VERSION_ID: u32 = 1;
    /// Other addresses of the same site; the user picks one in settings.
    const MIRRORS: &'static [&'static str] = &[];
    const RATE_LIMIT: Option<RateLimit> = None;

    const POPULAR_PATH: &'static str = "manga-list/hot-manga?page=";
    const LATEST_PATH: &'static str = "manga-list/latest-manga?page=";
    const SEARCH_PATH: &'static str = "search/story/";

    const POPULAR_SELECTOR: &'static str = ":is(div.truyen-list > div.list-truyen-item-wrap, div.comic-list > .list-comic-item-wrap):has(a[data-id])";
    const POPULAR_NEXT_SELECTOR: &'static str = "div.group_page, div.group-page a:not([href]) + a:not(:contains(Last))";
    const SEARCH_SELECTOR: &'static str = ".panel_story_list .story_item, div.list-truyen-item-wrap, div.list-comic-item-wrap";
    const SEARCH_NEXT_SELECTOR: &'static str = "a.page_select + a:not(.page_last), a.page-select + a:not(.page-last)";

    const DETAILS_SELECTOR: &'static str = "div.manga-info-top, div.panel-story-info";
    const THUMBNAIL_SELECTOR: &'static str = "div.manga-info-pic img, span.info-image img";
    const DESCRIPTION_SELECTOR: &'static str = "div#noidungm, div#panel-story-info-description, div#contentBox";
    const ALT_NAME_SELECTOR: &'static str = ".story-alternative, tr:has(.info-alternative) h2";
    const ALT_NAME_LABEL: &'static str = "Alternative Name: ";

    /// The part of the title's address that names it (`/manga/<slug>`). Sites whose list links carry an id
    /// instead of a name override this.
    fn compute_slug(manga: &Manga) -> String {
        manga.url.rsplit('/').next().unwrap_or_default().to_string()
    }
}

pub struct Site<T>(PhantomData<T>);

impl<T> Default for Site<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

fn base<T: Theme>() -> String {
    base_url(T::BASE_URL)
}

/// The slug stored with the title, or the one computed from its address.
fn slug<T: Theme>(manga: &Manga) -> String {
    manga
        .memo
        .as_ref()
        .and_then(|m| m["slug"].as_str())
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| T::compute_slug(manga))
}

fn manga_url<T: Theme>(manga: &Manga) -> String {
    format!("{}/manga/{}", base::<T>(), slug::<T>(manga))
}

impl<T: Theme> Source for Site<T> {
    fn info(&self) -> Info {
        Info {
            name: T::NAME.into(),
            lang: T::LANG.into(),
            base_url: T::BASE_URL.into(),
            version_id: T::VERSION_ID,
            mirrors: T::MIRRORS.iter().map(|m| m.to_string()).collect(),
            supports_latest: true,
            rate_limit: T::RATE_LIMIT,
            prefs: vec![PrefDef::switch(
                MERGE_PREF,
                "Merge Split Images",
                "Images are sometimes split vertically. This setting enables detecting and merging split images. Note that this isn't 100% accurate.",
                false,
            )],
            ..Default::default()
        }
    }

    fn popular(&self, page: u32) -> Result<MangaPage> {
        let doc = get(&format!("{}/{}{page}", base::<T>(), T::POPULAR_PATH)).send()?.html();
        Ok(parse_list::<T>(&doc, T::POPULAR_SELECTOR, T::POPULAR_NEXT_SELECTOR))
    }

    fn latest(&self, page: u32) -> Result<MangaPage> {
        let doc = get(&format!("{}/{}{page}", base::<T>(), T::LATEST_PATH)).send()?.html();
        Ok(parse_list::<T>(&doc, T::POPULAR_SELECTOR, T::POPULAR_NEXT_SELECTOR))
    }

    fn search(&self, page: u32, query: &str, filters: &Filters) -> Result<MangaPage> {
        let url = if !query.trim().is_empty() {
            format!("{}/{}{}?page={page}", base::<T>(), T::SEARCH_PATH, url_encode(&normalize_query(query)))
        } else {
            let sort = filters.select_value("sort", &SORTS);
            let status = filters.select_value("status", &STATUSES);
            let genre = filters.select_value("genre", GENRES);
            let id = FILTER_IDS.iter().find(|(s, st, _)| *s == sort && *st == status).map(|t| t.2);
            let mut url = format!("{}/genre/{genre}", base::<T>());
            url.push_str(&format!("?{}page={page}", id.map(|i| format!("filter={i}&")).unwrap_or_default()));
            url
        };
        let doc = get(&url).send()?.html();
        Ok(parse_list::<T>(&doc, T::SEARCH_SELECTOR, T::SEARCH_NEXT_SELECTOR))
    }

    fn filters(&self, _data: Option<&Value>) -> Vec<FilterDef> {
        vec![
            FilterDef::header("NOTE: Ignored if using text search!"),
            FilterDef::Separator,
            FilterDef::select("sort", "Order by", &SORTS, 0),
            FilterDef::select("status", "Status", &STATUSES, 0),
            FilterDef::select("genre", "Category", GENRES, 0),
        ]
    }

    fn details(&self, manga: &Manga) -> Result<Manga> {
        let doc = get(&manga_url::<T>(manga)).send()?.html();
        parse_details::<T>(&doc)
    }

    fn chapters(&self, manga: &Manga) -> Result<Vec<Chapter>> {
        parse_chapters::<T>(manga, get(&chapters_url::<T>(manga)).send()?)
    }

    /// Details and chapters are separate requests; run them together.
    fn update(&self, manga: &Manga, fetch_details: bool, fetch_chapters: bool) -> Result<Update> {
        let mut requests = Vec::new();
        if fetch_details {
            requests.push(get(&manga_url::<T>(manga)));
        }
        if fetch_chapters {
            requests.push(get(&chapters_url::<T>(manga)));
        }
        let mut answers = send_all(requests).into_iter();
        let mut update = Update::default();
        if fetch_details {
            update.manga = Some(parse_details::<T>(&answers.next().ok_or("no answer")??.html())?);
        }
        if fetch_chapters {
            update.chapters = Some(parse_chapters::<T>(manga, answers.next().ok_or("no answer")??)?);
        }
        Ok(update)
    }

    fn by_url(&self, url: &str) -> Result<Option<Manga>> {
        let base = base::<T>();
        let host = |u: &str| Url::parse(u).ok().and_then(|u| u.host_str().map(str::to_string));
        if host(url) != host(&base) {
            return Ok(None);
        }
        parse_details::<T>(&get(url).send()?.html()).map(Some)
    }

    fn pages(&self, chapter: &Chapter) -> Result<Vec<Page>> {
        let url = if chapter.url.starts_with("http") { chapter.url.clone() } else { absolute(&base::<T>(), &chapter.url) };
        let resp = get(&url).send()?;
        let doc = resp.html();
        let script: String = doc.select("script:containsData(cdns =)").iter().map(|s| s.data()).collect::<Vec<_>>().join("\n");
        let cdns = [extract_array(&script, "cdns"), extract_array(&script, "backupImage")].concat();
        let images = extract_array(&script, "chapterImages");

        let urls: Vec<String> = if !images.is_empty() {
            let first = cdns.first().ok_or("the chapter lists images but no CDN")?;
            let origin = Url::parse(first).map_err(|e| format!("bad CDN {first}: {e}"))?;
            images
                .iter()
                .map(|path| {
                    let mut u = origin.clone();
                    u.set_path(&format!("/{path}").replace("//", "/"));
                    u.to_string()
                })
                .collect()
        } else {
            doc.select("div.container-chapter-reader > img").iter().map(|i| i.abs_url("src")).collect()
        };

        // Other origins of the same files, tried when the first one does not answer.
        let mirrors: Vec<String> = {
            let mut seen: Vec<String> = Vec::new();
            for c in &cdns {
                if let Some(o) = origin_of(c) {
                    if !seen.contains(&o) {
                        seen.push(o);
                    }
                }
            }
            let primary = urls.first().and_then(|u| origin_of(u));
            seen.into_iter().filter(|o| Some(o) != primary.as_ref()).collect()
        };

        let location = resp.url.clone();
        let groups = if pref_bool(MERGE_PREF, false) { merge_groups(&urls) } else { urls.iter().map(|u| (vec![u.clone()], None)).collect() };
        Ok(groups
            .into_iter()
            .enumerate()
            .map(|(i, (parts, size))| {
                let mut page = Page::new(i as u32, parts[0].clone());
                page.url = location.clone();
                page.mirrors = mirrors.clone();
                if let (true, Some((width, height))) = (parts.len() > 1, size) {
                    page.ops = vec![ImageOp::Merge { urls: parts, width, height }];
                }
                page
            })
            .collect())
    }
}

// ------------------------------------------------------------------ lists

fn parse_list<T: Theme>(doc: &Document, item_selector: &str, next_selector: &str) -> MangaPage {
    let base = base::<T>();
    let mangas = doc.select(item_selector).iter().filter_map(|el| manga_from_element::<T>(el, &base)).collect();
    let has_next = !next_selector.is_empty() && doc.select_first(next_selector).is_some();
    MangaPage { mangas, has_next }
}

fn manga_from_element<T: Theme>(el: &El, base: &str) -> Option<Manga> {
    let link = el.select_first("h3 a")?;
    let href = link.abs_url("href");
    // `substringAfter(baseUrl)`: the path, or the whole address when it is on another host.
    let url = href.split_once(base).map(|(_, rest)| rest.to_string()).unwrap_or(href);
    let mut manga = Manga {
        title: link.text(),
        thumbnail_url: el.select_first("img").map(|i| i.abs_url("src")).filter(|s| !s.is_empty()),
        url,
        ..Default::default()
    };
    manga.memo = Some(json!({ "slug": T::compute_slug(&manga) }));
    Some(manga)
}

// ------------------------------------------------------------------ details and chapters

fn parse_details<T: Theme>(doc: &Document) -> Result<Manga> {
    let mut manga = Manga { initialized: true, ..Default::default() };
    // The title's own address names it; `/manga/<slug>` is what is stored.
    let located = Manga { url: doc.location().to_string(), ..Default::default() };
    let slug = T::compute_slug(&located);

    match doc.select_first(T::DETAILS_SELECTOR) {
        Some(info) => {
            manga.title = info.select_first("h1, h2").map(|h| h.text()).unwrap_or_default();
            manga.author = Some(info.select("li:contains(author) a, td:containsOwn(author) + td a").each_text().join(", "));
            manga.status = parse_status(&info.select("li:contains(status), td:containsOwn(status) + td").text());
            let genres = info
                .select_first("div.manga-info-top li:contains(genres)")
                .map(|li| li.select("a").each_text().join(", "))
                .unwrap_or_else(|| info.select("td:containsOwn(genres) + td a").each_text().join(", "));
            manga.genre = genres.split(", ").filter(|g| !g.is_empty()).map(str::to_string).collect();
        }
        None => {
            if doc.root().select_first("body").is_some_and(|b| b.text().starts_with("REDIRECT :")) {
                return Err("Source URL has changed".into());
            }
        }
    }
    let slug = if slug.is_empty() { T::compute_slug(&Manga { url: doc.location().to_string(), title: manga.title.clone(), ..Default::default() }) } else { slug };
    manga.url = format!("/manga/{slug}");
    manga.memo = Some(json!({ "slug": slug }));

    manga.description = doc.select_first(T::DESCRIPTION_SELECTOR).map(|d| clean_description(&d.own_text(), &manga.title));
    manga.thumbnail_url = doc.select_first(T::THUMBNAIL_SELECTOR).map(|i| i.abs_url("src")).filter(|s| !s.is_empty());

    if let Some(alt) = doc.select_first(T::ALT_NAME_SELECTOR) {
        let text = alt.own_text();
        if !text.is_empty() {
            manga.description = Some(match manga.description.take().filter(|d| !d.is_empty()) {
                Some(d) => format!("{d}\n\n{}{text}", T::ALT_NAME_LABEL),
                None => format!("{}{text}", T::ALT_NAME_LABEL),
            });
        }
    }
    Ok(manga)
}

fn clean_description(text: &str, title: &str) -> String {
    let prefix = format!("{title} summary:");
    let text = text.strip_prefix(&prefix).map(str::trim_start).unwrap_or(text);
    let br = Regex::new(r"<\s*br\s*/?>").unwrap();
    let tag = Regex::new(r"<[^>]*>").unwrap();
    tag.replace_all(&br.replace_all(text, "\n"), "").into_owned()
}

fn parse_status(s: &str) -> u8 {
    if s.contains("Ongoing") {
        1
    } else if s.contains("Completed") {
        2
    } else {
        0
    }
}

fn chapters_url<T: Theme>(manga: &Manga) -> String {
    format!("{}/api/manga/{}/chapters?limit=-1", base::<T>(), slug::<T>(manga))
}

#[derive(Deserialize)]
struct ApiResponse {
    #[serde(default)]
    success: bool,
    data: Option<ApiData>,
}

#[derive(Deserialize)]
struct ApiData {
    #[serde(default)]
    chapters: Vec<ApiChapter>,
}

#[derive(Deserialize)]
struct ApiChapter {
    chapter_name: Option<String>,
    chapter_slug: Option<String>,
    chapter_num: Option<f32>,
    updated_at: Option<String>,
}

fn parse_chapters<T: Theme>(manga: &Manga, resp: Response) -> Result<Vec<Chapter>> {
    let api: ApiResponse = resp.json()?;
    if !api.success {
        return Ok(Vec::new());
    }
    let base = base::<T>();
    let slug = slug::<T>(manga);
    let scanlator = base.replace("https://", "");
    Ok(api
        .data
        .map(|d| d.chapters)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|c| {
            let chapter_slug = c.chapter_slug?;
            Some(Chapter {
                name: c.chapter_name.unwrap_or_else(|| "Chapter".into()),
                url: format!("{base}/manga/{slug}/{chapter_slug}"),
                chapter_number: c.chapter_num.unwrap_or(0.0),
                scanlator: Some(scanlator.clone()),
                date_upload: c.updated_at.as_deref().and_then(parse_iso).unwrap_or(0),
                memo: None,
            })
        })
        .collect())
}

// ------------------------------------------------------------------ pages

/// `name = ["a", "b"]` in the page's script, as a list of strings.
fn extract_array(script: &str, name: &str) -> Vec<String> {
    let re = Regex::new(&format!(r"{name}\s*=\s*\[([^\]]+)\]")).unwrap();
    let Ok(Some(caps)) = re.captures(script) else { return Vec::new() };
    caps[1]
        .split(',')
        .map(|s| s.trim().trim_matches('"').replace("\\/", "/").trim_end_matches('/').to_string())
        .collect()
}

fn origin_of(url: &str) -> Option<String> {
    let u = Url::parse(url).ok()?;
    Some(format!("{}://{}{}", u.scheme(), u.host_str()?, u.port().map(|p| format!(":{p}")).unwrap_or_default()))
}

/// Groups consecutive images of the same width into one tall page when they fit together (a site that splits
/// long strips into pieces). Sizes come from the first bytes of each file; a file whose size is unknown stays
/// alone. Returns the addresses of each page and, for merged ones, the size of the result.
fn merge_groups(urls: &[String]) -> Vec<(Vec<String>, Option<(u32, u32)>)> {
    let requests = urls.iter().map(|u| get(u).range(0, 29)).collect();
    let sizes: Vec<Option<(u32, u32)>> = send_all(requests)
        .into_iter()
        .map(|r| r.ok().and_then(|r| image_size(&r.bytes())))
        .collect();

    let mut groups: Vec<(Vec<String>, Option<(u32, u32)>)> = Vec::new();
    for (url, size) in urls.iter().zip(sizes) {
        if let (Some((w, h)), Some((urls_so_far, Some((pw, ph))))) = (size, groups.last_mut()) {
            // same width, and the merged image stays under 1.5 times as tall as it is wide
            if w == *pw && 3 * *pw > 2 * *ph + h {
                urls_so_far.push(url.clone());
                *ph += h;
                continue;
            }
        }
        groups.push((vec![url.clone()], size));
    }
    groups
}

// ------------------------------------------------------------------ search

/// The site's own `change_alias` function: no accents, punctuation to `_`.
fn normalize_query(query: &str) -> String {
    let mut out = String::with_capacity(query.len());
    for c in query.to_lowercase().chars() {
        out.push(match c {
            'à' | 'á' | 'ạ' | 'ả' | 'ã' | 'â' | 'ầ' | 'ấ' | 'ậ' | 'ẩ' | 'ẫ' | 'ă' | 'ằ' | 'ắ' | 'ặ' | 'ẳ' | 'ẵ' => 'a',
            'è' | 'é' | 'ẹ' | 'ẻ' | 'ẽ' | 'ê' | 'ề' | 'ế' | 'ệ' | 'ể' | 'ễ' => 'e',
            'ì' | 'í' | 'ị' | 'ỉ' | 'ĩ' => 'i',
            'ò' | 'ó' | 'ọ' | 'ỏ' | 'õ' | 'ô' | 'ồ' | 'ố' | 'ộ' | 'ổ' | 'ỗ' | 'ơ' | 'ờ' | 'ớ' | 'ợ' | 'ở' | 'ỡ' => 'o',
            'ù' | 'ú' | 'ụ' | 'ủ' | 'ũ' | 'ư' | 'ừ' | 'ứ' | 'ự' | 'ử' | 'ữ' => 'u',
            'ỳ' | 'ý' | 'ỵ' | 'ỷ' | 'ỹ' => 'y',
            'đ' => 'd',
            '!' | '@' | '%' | '^' | '*' | '(' | ')' | '+' | '=' | '<' | '>' | '?' | '/' | ',' | '.' | ':' | ';' | '\'' | ' ' | '"' | '&' | '#' | '[' | ']' | '~' | '-' | '$' | '_' => '_',
            other => other,
        });
    }
    let mut collapsed = String::with_capacity(out.len());
    for c in out.chars() {
        if !(c == '_' && collapsed.ends_with('_')) {
            collapsed.push(c);
        }
    }
    collapsed.trim_matches('_').to_string()
}

const SORTS: [(&str, &str); 3] = [("Latest", "latest"), ("Newest", "newest"), ("Top read", "topview")];
const STATUSES: [(&str, &str); 3] = [("ALL", "all"), ("Completed", "completed"), ("Ongoing", "ongoing")];

/// The site's `filter=` ids for (sort, status).
const FILTER_IDS: [(&str, &str, &str); 9] = [
    ("newest", "all", "1"),
    ("newest", "completed", "2"),
    ("newest", "ongoing", "3"),
    ("latest", "all", "4"),
    ("latest", "completed", "5"),
    ("latest", "ongoing", "6"),
    ("topview", "all", "7"),
    ("topview", "completed", "8"),
    ("topview", "ongoing", "9"),
];

#[cfg(test)]
mod tests;
