//! ZeistManga: manga sites built on a Blogger blog. Titles and chapters are posts with labels; lists and chapter
//! lists come from the blog's JSON feed (`/feeds/posts/default/-/<label>?alt=json`), details and pages from the
//! posts' HTML. A site is a type that implements [`Theme`]; `export_source!(Site<MySite>)` makes it a source.
//!
//! Port of the `zeistmanga` multisrc of the Kotlin extensions (`lib-multisrc/zeistmanga`). Search filters are not
//! ported yet: no installed site turns them on.

use fancy_regex::Regex;
use serde::Deserialize;
use sk_source_sdk::html::Document;
use sk_source_sdk::*;
use std::marker::PhantomData;

const MAX_MANGA_RESULTS: u32 = 20;
const MAX_CHAPTER_RESULTS: u32 = 150;
const CHAPTER_CHUNK: u32 = 50;

pub trait Theme: 'static {
    const NAME: &'static str;
    const LANG: &'static str = "en";
    const BASE_URL: &'static str;
    const VERSION_ID: u32 = 1;
    const SUPPORTS_LATEST: bool = true;

    const MANGA_CATEGORY: &'static str = "Series";
    const CHAPTER_CATEGORY: &'static str = "Chapter";
    const EXCLUDED_CATEGORIES: &'static [&'static str] = &["Anime", "Novel", "Novela"];
    /// Newest chapter by `updated` instead of `published` date.
    const PREFER_CHAPTER_UPDATED_DATE: bool = false;
    const SUPPORTS_CHAPTER_FEED: bool = true;
    const USE_OLD_CHAPTER_FEED: bool = false;
    const USE_NEW_CHAPTER_FEED: bool = false;

    const POPULAR_SELECTOR: &'static str = "div.PopularPosts div.grid > figure";
    const POPULAR_TITLE: &'static str = "figcaption > a";
    const POPULAR_URL: &'static str = "figcaption > a";

    const DETAILS_SELECTOR: &'static str = ".grid.gtc-235fr";
    const DETAILS_THUMBNAIL: &'static str = "img";
    const DETAILS_DESCRIPTION: &'static str = "#synopsis";
    const DETAILS_GENRES: &'static str = "div.mt-15 > a[rel=tag]";
    const DETAILS_AUTHOR: &'static str = "span#author";
    const DETAILS_ARTIST: &'static str = "span#artist";
    const DETAILS_ALT_NAME: &'static str = "header > p";
    const DETAILS_STATUS: &'static str = "span[data-status]";
    const DETAILS_INFO: &'static str = ".y6x11p";
    const DETAILS_INFO_TITLE: &'static str = "strong";
    const DETAILS_INFO_DESCRIPTION: &'static str = "span.dt";

    const PAGE_SELECTOR: &'static str = "div.check-box div.separator";
    const CHAPTER_FEED_REGEX: &'static str = r#"clwd\.run\(["'](.*?)["']\)"#;
    const CHAPTER_SCRIPT_SELECTOR: &'static str = "#clwd > script";

    const STATUS_ONGOING: &'static [&'static str] = &["ongoing", "en curso", "en emisión", "em lançamento", "activo", "ativo", "lançando", "مستمر", "مستمرة"];
    const STATUS_COMPLETED: &'static [&'static str] = &["completed", "completo", "finalizado", "مكتمل", "مكتملة"];
    const STATUS_HIATUS: &'static [&'static str] = &["hiatus", "pausado"];
    const STATUS_CANCELLED: &'static [&'static str] = &["cancelled", "dropped", "dropado", "abandonado", "cancelado"];
    const STATUS_LABELS: &'static [&'static str] = &["Status", "Estado", "الحالة"];
    const AUTHOR_LABELS: &'static [&'static str] = &["Author", "Autor", "Mangaka", "الكاتب", "Yazar"];
    const ARTIST_LABELS: &'static [&'static str] = &["Artist", "Artista", "الرسام", "Çizer"];
}

pub struct Site<T>(PhantomData<T>);

impl<T> Default for Site<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T: Theme> Source for Site<T> {
    fn info(&self) -> Info {
        Info {
            name: T::NAME.into(),
            lang: T::LANG.into(),
            base_url: T::BASE_URL.into(),
            version_id: T::VERSION_ID,
            supports_latest: T::SUPPORTS_LATEST,
            ..Default::default()
        }
    }

    fn popular(&self, page: u32) -> Result<MangaPage> {
        if !T::SUPPORTS_LATEST {
            return self.latest(page);
        }
        let doc = get(T::BASE_URL).send()?.html();
        let mangas = doc
            .select(T::POPULAR_SELECTOR)
            .iter()
            .filter_map(|el| {
                let link = el.select_first(T::POPULAR_URL)?;
                Some(Manga {
                    thumbnail_url: el.select_first("img").map(|i| i.abs_url("src")),
                    title: el.select_first(T::POPULAR_TITLE)?.text(),
                    url: path_of(&link.attr("href")),
                    ..Default::default()
                })
            })
            .collect();
        Ok(MangaPage { mangas, has_next: false })
    }

    fn latest(&self, page: u32) -> Result<MangaPage> {
        let start = MAX_MANGA_RESULTS * (page - 1) + 1;
        let url = api_url::<T>(T::MANGA_CATEGORY)
            .query("orderby", "published")
            .query("max-results", &(MAX_MANGA_RESULTS + 1).to_string())
            .query("start-index", &start.to_string())
            .build();
        parse_feed_list::<T>(&get(&url).send()?.text())
    }

    fn search(&self, page: u32, query: &str, _filters: &Filters) -> Result<MangaPage> {
        let start = MAX_MANGA_RESULTS * (page - 1) + 1;
        let mut url = api_url::<T>(T::MANGA_CATEGORY);
        if !query.trim().is_empty() {
            url = url
                .query("q", query)
                .query("max-results", &(MAX_MANGA_RESULTS + 1).to_string())
                .query("start-index", &start.to_string());
        }
        // The blog's own search needs the label in the query text itself.
        let url = replace_last(&url.build(), "q=", &format!("q=label:{}+", T::MANGA_CATEGORY));
        parse_feed_list::<T>(&get(&url).send()?.text())
    }

    fn update(&self, manga: &Manga, fetch_details: bool, fetch_chapters: bool) -> Result<Update> {
        let page_url = absolute(T::BASE_URL, &manga.url);
        let remembered = manga.memo.as_ref().and_then(|m| serde_json::from_value::<FeedUrl>(m["feedUrl"].clone()).ok()).filter(feed_is_valid::<T>);

        if let (Some(feed), true) = (remembered.as_ref(), T::SUPPORTS_CHAPTER_FEED) {
            // The feed address is known: details and chapters do not depend on each other.
            let chapters = fetch_chapters.then(|| chapter_list::<T>(&feed.url)).transpose()?;
            let details = if fetch_details { Some(parse_details::<T>(&get(&page_url).send()?.html(), manga)?) } else { None };
            return Ok(Update { manga: details.map(|d| Manga { memo: manga.memo.clone(), ..d }), chapters });
        }

        let doc = get(&page_url).send()?.html();
        let feed_url = chapter_feed_url::<T>(&doc, &manga.title)?;
        let mut details = parse_details::<T>(&doc, manga)?;
        details.memo = Some(json!({ "feedUrl": FeedUrl {
            url: feed_url.clone(),
            old: T::USE_OLD_CHAPTER_FEED,
            new: T::USE_NEW_CHAPTER_FEED,
            category: T::CHAPTER_CATEGORY.into(),
        } }));
        Ok(Update { manga: Some(details), chapters: if fetch_chapters { Some(chapter_list::<T>(&feed_url)?) } else { None } })
    }

    fn details(&self, manga: &Manga) -> Result<Manga> {
        self.update(manga, true, false)?.manga.ok_or_else(|| "no details".into())
    }

    fn chapters(&self, manga: &Manga) -> Result<Vec<Chapter>> {
        self.update(manga, false, true)?.chapters.ok_or_else(|| "no chapters".into())
    }

    fn by_url(&self, url: &str) -> Result<Option<Manga>> {
        let parsed = Url::parse(url).map_err(|e| e.to_string())?;
        if parsed.path_segments().map(|s| s.count()).unwrap_or(0) < 3 {
            return Ok(None);
        }
        let stub = Manga { url: path_of(url), ..Default::default() };
        let doc = get(url).send()?.html();
        let mut m = parse_details::<T>(&doc, &stub)?;
        if m.title.is_empty() {
            m.title = doc
                .select_first("meta[property='og:title']")
                .map(|t| t.attr("content").split('-').next().unwrap_or_default().trim().to_string())
                .unwrap_or_default();
        }
        m.url = path_of(url);
        Ok(Some(m))
    }

    fn pages(&self, chapter: &Chapter) -> Result<Vec<Page>> {
        let doc = get(&absolute(T::BASE_URL, &chapter.url)).send()?.html();
        Ok(doc
            .select(T::PAGE_SELECTOR)
            .select("img[src]")
            .iter()
            .enumerate()
            .map(|(i, img)| Page::new(i as u32, img.abs_url("src")))
            .collect())
    }
}

// ------------------------------------------------------------------ feed

#[derive(Deserialize, Default)]
struct FeedDto {
    feed: Option<FeedBody>,
}

#[derive(Deserialize, Default)]
struct FeedBody {
    #[serde(rename = "openSearch$totalResults")]
    total_results: Option<Text>,
    #[serde(rename = "openSearch$itemsPerPage")]
    items_per_page: Option<Text>,
    #[serde(default)]
    entry: Option<Vec<Entry>>,
}

#[derive(Deserialize)]
struct Text {
    #[serde(rename = "$t")]
    t: String,
}

#[derive(Deserialize)]
struct Entry {
    title: Option<Text>,
    published: Option<Text>,
    updated: Option<Text>,
    #[serde(default)]
    category: Option<Vec<Term>>,
    #[serde(default)]
    link: Option<Vec<Link>>,
    content: Option<Text>,
    #[serde(rename = "media$thumbnail")]
    thumbnail: Option<Thumbnail>,
}

#[derive(Deserialize)]
struct Term {
    term: String,
}

#[derive(Deserialize)]
struct Link {
    rel: String,
    href: String,
}

#[derive(Deserialize)]
struct Thumbnail {
    url: String,
}

#[derive(serde::Serialize, Deserialize, Clone)]
struct FeedUrl {
    url: String,
    old: bool,
    new: bool,
    category: String,
}

fn feed_is_valid<T: Theme>(f: &FeedUrl) -> bool {
    f.old == T::USE_OLD_CHAPTER_FEED && f.new == T::USE_NEW_CHAPTER_FEED && f.category == T::CHAPTER_CATEGORY
}

impl Entry {
    fn has_category(&self, name: &str) -> bool {
        self.category.iter().flatten().any(|c| c.term == name)
    }

    /// The post's own address (`rel=alternate`) without the site, as the extension stored it (`substringAfter`).
    fn path(&self, base: &str) -> Option<String> {
        self.link.iter().flatten().find(|l| l.rel == "alternate").map(|l| substring_after(&l.href, base))
    }

    fn thumbnail(&self) -> Option<String> {
        match &self.thumbnail {
            Some(t) => {
                // Blogger resizes through the address; ask for a 600 px wide picture.
                let a = Regex::new(r"/s.+?-c/").unwrap().replace(&t.url, "/w600/").into_owned();
                Some(Regex::new(r"=s(?!.*=s).+?-c$").unwrap().replace(&a, "=w600").into_owned())
            }
            None => {
                let html = &self.content.as_ref()?.t;
                sk_source_sdk::html::Document::parse(html, "").select_first("img").map(|i| i.attr("src"))
            }
        }
    }
}

fn parse_feed_list<T: Theme>(body: &str) -> Result<MangaPage> {
    let dto: FeedDto = serde_json::from_str(body).map_err(|e| format!("bad feed: {e}"))?;
    let mut mangas: Vec<Manga> = dto
        .feed
        .and_then(|f| f.entry)
        .unwrap_or_default()
        .iter()
        .filter(|e| e.has_category(T::MANGA_CATEGORY) && !T::EXCLUDED_CATEGORIES.iter().any(|c| e.has_category(c)))
        .filter_map(|e| {
            Some(Manga { title: e.title.as_ref()?.t.clone(), url: e.path(T::BASE_URL)?, thumbnail_url: e.thumbnail(), ..Default::default() })
        })
        .collect();
    // One more than a page is requested; its presence says there is a next page.
    let has_next = mangas.len() == MAX_MANGA_RESULTS as usize + 1;
    if has_next {
        mangas.pop();
    }
    Ok(MangaPage { mangas, has_next })
}

/// Builds `<base>/feeds/posts/default/-/<feed>?alt=json&...`.
struct ApiUrl {
    url: Url,
}

fn api_url<T: Theme>(feed: &str) -> ApiUrl {
    let mut url = Url::parse(&format!("{}/feeds/posts/default/-/", T::BASE_URL.trim_end_matches('/'))).expect("base address");
    url.path_segments_mut().expect("http address").pop_if_empty().push(feed);
    url.query_pairs_mut().append_pair("alt", "json");
    ApiUrl { url }
}

impl ApiUrl {
    fn query(mut self, k: &str, v: &str) -> Self {
        self.url.query_pairs_mut().append_pair(k, v);
        self
    }

    fn build(self) -> String {
        self.url.to_string()
    }
}

/// Replaces the last occurrence of `old` in `s`.
fn replace_last(s: &str, old: &str, new: &str) -> String {
    match s.rfind(old) {
        Some(i) => format!("{}{new}{}", &s[..i], &s[i + old.len()..]),
        None => s.to_string(),
    }
}

/// `url` with the query parameter `key` set to `value`, replacing earlier ones.
fn set_query(url: &str, key: &str, value: &str) -> Result<String> {
    let mut u = Url::parse(url).map_err(|e| format!("bad feed address {url}: {e}"))?;
    let kept: Vec<(String, String)> = u.query_pairs().filter(|(k, _)| k != key).map(|(k, v)| (k.into_owned(), v.into_owned())).collect();
    u.query_pairs_mut().clear().extend_pairs(kept).append_pair(key, value);
    Ok(u.to_string())
}

// ------------------------------------------------------------------ details

fn parse_details<T: Theme>(doc: &Document, previous: &Manga) -> Result<Manga> {
    let profile = doc.select_first(T::DETAILS_SELECTOR).ok_or("the title page has no details block (the site changed?)")?;
    let mut manga = Manga {
        url: previous.url.clone(),
        title: previous.title.clone(),
        thumbnail_url: profile.select_first(T::DETAILS_THUMBNAIL).map(|i| i.abs_url("src")),
        genre: profile.select(T::DETAILS_GENRES).each_text(),
        author: profile.select_first(T::DETAILS_AUTHOR).map(|e| e.text()),
        artist: profile.select_first(T::DETAILS_ARTIST).map(|e| e.text()),
        status: parse_status::<T>(&profile.select_first(T::DETAILS_STATUS).map(|e| e.text()).unwrap_or_default()),
        initialized: true,
        ..Default::default()
    };
    let mut description = profile.select(T::DETAILS_DESCRIPTION).text();
    description.push_str("\n\n");
    if let Some(alt) = profile.select_first(T::DETAILS_ALT_NAME).map(|e| e.text()).filter(|t| !t.trim().is_empty()) {
        description.push_str("Alternative name(s): ");
        description.push_str(&alt);
    }
    manga.description = Some(description.trim().to_string());
    manga.genre.retain(|g| !g.is_empty());

    for element in profile.select(T::DETAILS_INFO).iter() {
        let info = Some(element.own_text().trim().to_string())
            .filter(|t| !t.is_empty())
            .or_else(|| element.select_first(T::DETAILS_INFO_TITLE).map(|t| t.text().trim().to_string()))
            .unwrap_or_default();
        let value = element.select(T::DETAILS_INFO_DESCRIPTION).text().trim().to_string();
        let has = |labels: &[&str]| labels.iter().any(|l| info.contains(l));
        if manga.status == 0 && has(T::STATUS_LABELS) {
            manga.status = parse_status::<T>(&value);
        } else if manga.author.is_none() && has(T::AUTHOR_LABELS) {
            manga.author = Some(value);
        } else if manga.artist.is_none() && has(T::ARTIST_LABELS) {
            manga.artist = Some(value);
        }
    }
    Ok(manga)
}

fn parse_status<T: Theme>(s: &str) -> u8 {
    let s = s.to_lowercase();
    let s = s.trim();
    if T::STATUS_ONGOING.contains(&s) {
        1
    } else if T::STATUS_COMPLETED.contains(&s) {
        2
    } else if T::STATUS_HIATUS.contains(&s) {
        6
    } else if T::STATUS_CANCELLED.contains(&s) {
        5
    } else {
        0
    }
}

// ------------------------------------------------------------------ chapters

fn fetch_feed(url: &str, start: u32, max: u32) -> Result<FeedDto> {
    let paged = set_query(&set_query(url, "start-index", &start.to_string())?, "max-results", &max.to_string())?;
    get(&paged).send()?.json()
}

fn chapter_list<T: Theme>(feed_url: &str) -> Result<Vec<Chapter>> {
    // The first request says how many chapters there are and how many the server returns per page.
    let first = fetch_feed(feed_url, 1, MAX_CHAPTER_RESULTS)?;
    let body = first.feed.ok_or("Failed to parse from chapter API")?;
    let total = body.total_results.as_ref().and_then(|t| t.t.parse::<u32>().ok()).unwrap_or(MAX_CHAPTER_RESULTS);
    let per_page = body.items_per_page.as_ref().and_then(|t| t.t.parse::<u32>().ok()).unwrap_or(CHAPTER_CHUNK).max(1);
    let mut entries = body.entry.ok_or("Failed to parse from chapter API")?;

    // The rest, in parallel.
    let mut starts = Vec::new();
    let mut at = entries.len() as u32;
    while at < total {
        starts.push(at + 1);
        at += per_page;
    }
    let requests = starts
        .iter()
        .map(|s| set_query(&set_query(feed_url, "start-index", &s.to_string())?, "max-results", &per_page.to_string()).map(|u| get(&u)))
        .collect::<Result<Vec<_>>>()?;
    for answer in send_all(requests) {
        let dto: FeedDto = answer?.json()?;
        entries.extend(dto.feed.and_then(|f| f.entry).ok_or("Failed to parse from chapter API")?);
    }

    Ok(entries
        .iter()
        .filter(|e| e.has_category(T::CHAPTER_CATEGORY))
        .filter_map(|e| {
            let published = e.published.as_ref().map(|t| t.t.trim().to_string());
            let updated = e.updated.as_ref().map(|t| t.t.trim().to_string());
            let date = if T::PREFER_CHAPTER_UPDATED_DATE { updated.or(published) } else { published.or(updated) };
            Some(Chapter {
                name: e.title.as_ref()?.t.clone(),
                url: e.path(T::BASE_URL)?,
                date_upload: date.as_deref().and_then(parse_iso).unwrap_or(0),
                ..Default::default()
            })
        })
        .collect())
}

/// Where the blog lists a title's chapters: found in the page's script, or the old/new layouts' own places.
fn chapter_feed_url<T: Theme>(doc: &Document, title: &str) -> Result<String> {
    if T::USE_NEW_CHAPTER_FEED {
        return new_chapter_feed_url::<T>(doc);
    }
    if T::USE_OLD_CHAPTER_FEED {
        return old_chapter_feed_url::<T>(doc);
    }
    let label = match doc.select_first(T::CHAPTER_SCRIPT_SELECTOR) {
        None => return old_chapter_feed_url::<T>(doc).or_else(|_| new_chapter_feed_url::<T>(doc)),
        Some(script) => Regex::new(T::CHAPTER_FEED_REGEX)
            .ok()
            .and_then(|re| {
                let data = script.data();
                re.captures(data.as_str()).ok().flatten().map(|c| c[1].to_string())
            })
            // No label in the script: the blog labels chapters with the title.
            .unwrap_or_else(|| title.to_string()),
    };
    let mut url = api_url::<T>(T::CHAPTER_CATEGORY).url;
    url.path_segments_mut().map_err(|_| "bad feed address")?.extend(label.split('/'));
    Ok(url.to_string())
}

fn old_chapter_feed_url<T: Theme>(doc: &Document) -> Result<String> {
    let src = doc.select_first("#myUL > script").ok_or("no chapter list script")?.attr("src");
    let re = Regex::new(r"([^']+)\?").unwrap();
    let feed = re.captures(src.as_str()).ok().flatten().map(|c| c[1].to_string()).ok_or("Failed to find chapter feed")?;
    // The script's address is the blog's own feed path, so it goes after the site address.
    Ok(format!("{}{feed}?alt=json", T::BASE_URL.trim_end_matches('/')))
}

fn new_chapter_feed_url<T: Theme>(doc: &Document) -> Result<String> {
    let (script, re) = match doc.select_first(T::CHAPTER_SCRIPT_SELECTOR) {
        Some(s) => (s, Regex::new(T::CHAPTER_FEED_REGEX).unwrap()),
        None => (doc.select_first("#latest > script").ok_or("no chapter list script")?, Regex::new(r"label\s*=\s*'([^']+)'").unwrap()),
    };
    let data = script.data();
    let feed = re.captures(data.as_str()).ok().flatten().map(|c| c[1].to_string()).ok_or("Failed to find chapter feed")?;
    Ok(api_url::<T>(&feed).query("start-index", "1").query("max-results", "999999").build())
}

#[cfg(test)]
mod tests;
