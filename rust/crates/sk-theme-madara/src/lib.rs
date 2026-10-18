//! Madara: the WordPress manga theme behind hundreds of sites. Lists load through `admin-ajax.php` (`madara_load_more`),
//! a title is its numeric post id, details and chapters come from the title's page or from one of five chapter
//! endpoints, and pages are the images of the chapter page (some sites encrypt them).
//!
//! Port of `Madara` and `MadaraBase` of the Kotlin `madara` multisrc. Not ported: the variant for sites without
//! `admin-ajax` (`MadaraNoAjax`) and the "view count" ping the Kotlin theme sent after opening a title or chapter.

use aes::Aes256;
use base64::Engine;
use cbc::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};
use fancy_regex::Regex;
use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};
use sk_source_sdk::html::{Document, El};
use sk_source_sdk::*;
use std::marker::PhantomData;

/// How a site hands out a title's chapter list.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ChapterMode {
    /// In the title's own page.
    MangaPage,
    /// `admin-ajax.php` with `action=manga_get_chapters`.
    AdminAjax,
    /// `POST <title>/ajax/chapters/`.
    MangaAjax,
    /// The same, paged with `?t=1`, `?t=2`, ... until it runs out.
    MangaAjaxPaginated,
    /// `POST /index.php` with `manga_ajax`.
    MangaAjaxQuery,
}

pub trait Theme: 'static {
    const NAME: &'static str;
    const LANG: &'static str = "en";
    const BASE_URL: &'static str;
    const VERSION_ID: u32 = 1;
    const RATE_LIMIT: Option<RateLimit> = None;
    /// Requests whose path starts with this are not rate limited (`/wp-content/uploads/`: the pictures).
    const RATE_LIMIT_SKIP_PATH: &'static str = "";
    const SUPPORTS_LATEST: bool = true;
    const SUPPORTS_FILTER_FETCHING: bool = true;

    /// The site lists titles in plain archive pages (`/manga/page/2/?m_orderby=views`) and has no `madara_load_more`
    /// endpoint (the Kotlin `MadaraNoAjax`).
    const NO_AJAX: bool = false;
    const NEXT_PAGE_SELECTOR: &'static str = "div.nav-previous, a.nextpostslink, #navigation-ajax";
    const ORDER_QUERY_PARAMETER: &'static str = "m_orderby";
    const SEARCH_QUERY_PARAMETER: &'static str = "s";

    const MANGA_SUB_STRING: &'static str = "manga";
    const GENRE_DIRECTORY: &'static str = "manga-genre";
    const FILTER_NON_MANGA_ITEMS: bool = true;
    const SUPPORTS_POST_ID: bool = true;
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaPage;
    const PAGE_SIZE: u32 = 25;
    const AJAX_TEMPLATE: &'static str = "madara-core/content/content-archive";

    const ARCHIVE_SELECTOR: &'static str = "div.page-item-detail, .manga__item, .c-tabs-item__content";
    const ARCHIVE_TITLE_SELECTOR: Option<&'static str> = None;
    const ARCHIVE_URL_SELECTOR: &'static str = ".post-title a";
    const DETAILS_TITLE: &'static str = "div.post-title h3, div.post-title h1, #manga-title > h1";
    const DETAILS_AUTHOR: &'static str = "div.author-content > a, div.manga-authors > a";
    const DETAILS_ARTIST: &'static str = "div.artist-content > a";
    const DETAILS_STATUS: &'static str = "div.summary-content, div.summary-heading:contains(Status) + div";
    const DETAILS_DESCRIPTION: &'static str = "div.description-summary div.summary__content, div.summary_content div.post-content_item > h5 + div, div.summary_content div.manga-excerpt";
    const DETAILS_THUMBNAIL: &'static str = "div.summary_image img";
    const DETAILS_GENRE: &'static str = "div.genres-content a";
    const DETAILS_TAG: &'static str = "div.tags-content a";
    const SERIES_TYPE: &'static str = ".post-content_item:contains(Type) .summary-content";
    const ALT_NAME: &'static str = ".post-content_item:contains(Alt) .summary-content";
    const UPDATING_REGEX: &'static str = "(?i)Updating|Atualizando";
    const CHAPTER_LIST_SELECTOR: &'static str = "li.wp-manga-chapter";
    const CHAPTER_NAME_SELECTOR: Option<&'static str> = None;
    const CHAPTER_URL_SELECTOR: &'static str = "a";
    const CHAPTER_DATE_SELECTOR: &'static str = "span.chapter-release-date";
    const PAGE_LIST_SELECTOR: &'static str = "div.page-break, li.blocks-gallery-item, .reading-content .text-left:not(:has(.blocks-gallery-item))";
    const FILTER_GENRES_SELECTOR: &'static str = "div.genres";
    /// `java.time` pattern of absolute chapter dates.
    const CHAPTER_DATE_PATTERN: &'static str = "MMMM d, yyyy";

    /// The address of a picture in `el`, trying the lazy-loading attributes first.
    fn image_from_element(el: &El) -> Option<String> {
        default_image(el)
    }

    /// Lets a site fix a cover address (a resized variant, say).
    fn process_thumbnail(url: Option<String>, _from_search: bool) -> Option<String> {
        url
    }
}

pub struct Site<T>(PhantomData<T>);

impl<T> Default for Site<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

const GENERIC_GENRES: [&str; 4] = ["manga", "manhwa", "manhua", "webtoon"];
const STATUS: [(&str, &str); 4] = [("Completed", "end"), ("Ongoing", "on-going"), ("Canceled", "canceled"), ("On Hold", "on-hold")];
const ORDER_BY: [(&str, &str); 7] = [
    ("Relevance", ""),
    ("Latest", "latest"),
    ("A-Z", "alphabet"),
    ("Rating", "rating"),
    ("Trending", "trending"),
    ("Most Views", "views"),
    ("New", "new-manga"),
];
fn pairs(a: &[(&str, &str)]) -> Vec<(String, String)> {
    a.iter().map(|(l, v)| (l.to_string(), v.to_string())).collect()
}

const GENRE_CONDITION: [(&str, &str); 2] = [("OR", ""), ("AND", "1")];
const ADULT: [(&str, &str); 3] = [("All", ""), ("None", "0"), ("Only", "1")];

fn base<T: Theme>() -> String {
    base_url(T::BASE_URL)
}

/// A genre as the site lists it: its name, slug and archive path.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GenreRoute {
    pub name: String,
    pub slug: String,
    pub path: String,
}

// ------------------------------------------------------------------ the source

impl<T: Theme> Source for Site<T> {
    fn info(&self) -> Info {
        Info {
            name: T::NAME.into(),
            lang: T::LANG.into(),
            base_url: T::BASE_URL.into(),
            version_id: T::VERSION_ID,
            supports_latest: T::SUPPORTS_LATEST,
            supports_filter_fetching: T::SUPPORTS_FILTER_FETCHING,
            supports_related: true,
            rate_limit: T::RATE_LIMIT.map(|mut r| {
                r.skip_path = T::RATE_LIMIT_SKIP_PATH.into();
                r
            }),
            ..Default::default()
        }
    }

    fn popular(&self, page: u32) -> Result<MangaPage> {
        ajax_list::<T>(page, Browse::Popular, "", &Filters::default())
    }

    fn latest(&self, page: u32) -> Result<MangaPage> {
        ajax_list::<T>(page, Browse::Latest, "", &Filters::default())
    }

    fn search(&self, page: u32, query: &str, filters: &Filters) -> Result<MangaPage> {
        ajax_list::<T>(page, Browse::Search, query, filters)
    }

    fn filters(&self, data: Option<&Value>) -> Vec<FilterDef> {
        let genres = genre_routes(data);
        let mut f = vec![
            FilterDef::text("author", "Author"),
            FilterDef::text("artist", "Artist"),
            FilterDef::text("release", "Year of Released"),
            FilterDef::checkboxes("status", "Status", &pairs(&STATUS)),
            FilterDef::select("order", "Order By", &ORDER_BY, 0),
            FilterDef::select("adult", "Adult Content", &ADULT, 0),
        ];
        if !genres.is_empty() {
            f.push(FilterDef::header("Genres filter may not work for all sources"));
            f.push(FilterDef::select("genre_condition", "Genre condition", &GENRE_CONDITION, 0));
            let items: Vec<(String, String)> = genres.iter().map(|g| (g.name.clone(), g.slug.clone())).collect();
            f.push(FilterDef::checkboxes("genres", "Genres", &items));
        }
        f
    }

    fn filter_data(&self) -> Result<Option<Value>> {
        let doc = get(&format!("{}/{}/", base::<T>(), T::MANGA_SUB_STRING)).send()?.html();
        let mut seen = Vec::<String>::new();
        let mut routes = Vec::<GenreRoute>::new();
        for a in doc.select(T::FILTER_GENRES_SELECTOR).select(&format!("a[href*='/{}/']", T::GENRE_DIRECTORY)).iter() {
            let path = path_only(&a.abs_url("href"));
            let name = a.text();
            let slug = path.trim_end_matches('/').rsplit('/').next().unwrap_or_default().to_string();
            if !name.is_empty() && !slug.is_empty() && !seen.contains(&slug) {
                seen.push(slug.clone());
                routes.push(GenreRoute { name, slug, path });
            }
        }
        Ok(Some(serde_json::to_value(routes).map_err(|e| e.to_string())?))
    }

    fn manga_url(&self, manga: &Manga) -> Option<String> {
        let b = base::<T>();
        if let Some(p) = memo_path(manga) {
            return Some(format!("{}{p}", b));
        }
        if !manga.url.chars().all(|c| c.is_ascii_digit()) {
            return Some(format!("{}{}", b, manga.url));
        }
        Some(format!("{}/?p={}", b, manga.url))
    }

    fn chapter_url(&self, chapter: &Chapter) -> Option<String> {
        let path = chapter.memo.as_ref()?["mangaPath"].as_str()?.to_string();
        Some(format!("{}{}/{}/", base::<T>(), path.trim_end_matches('/'), chapter.url))
    }

    fn update(&self, manga: &Manga, fetch_details: bool, fetch_chapters: bool) -> Result<Update> {
        let id = if T::SUPPORTS_POST_ID { manga_id(manga) } else { Some(String::new()) };
        let path = memo_path(manga);
        let memo_usable = T::CHAPTER_MODE != ChapterMode::MangaPage && id.is_some() && path.is_some();

        // everything the chapter endpoints need is already stored: no title page for the chapters alone
        if fetch_chapters && !fetch_details && memo_usable {
            return Ok(Update { manga: None, chapters: Some(fetch_chapter_list::<T>(path.as_deref().unwrap(), id.as_deref().unwrap(), None)?) });
        }

        let doc = details_document::<T>(manga)?;
        let resolved_id = if T::SUPPORTS_POST_ID { id.or_else(|| manga_id_of(&doc)).ok_or("Missing Madara post ID")? } else { String::new() };
        let resolved_path = path.unwrap_or_else(|| path_only(doc.location()));
        let updated = if fetch_details { Some(parse_details::<T>(&doc, &resolved_id, &manga.url)?) } else { None };
        let chapters = if fetch_chapters { Some(fetch_chapter_list::<T>(&resolved_path, &resolved_id, Some(&doc))?) } else { None };
        Ok(Update { manga: updated, chapters })
    }

    fn details(&self, manga: &Manga) -> Result<Manga> {
        self.update(manga, true, false)?.manga.ok_or_else(|| "no details".into())
    }

    fn chapters(&self, manga: &Manga) -> Result<Vec<Chapter>> {
        self.update(manga, false, true)?.chapters.ok_or_else(|| "no chapters".into())
    }

    fn pages(&self, chapter: &Chapter) -> Result<Vec<Page>> {
        let url = self.chapter_url(chapter).ok_or("Refresh the chapter list.")?;
        let first = get(&url).send()?.html();
        let doc = if first.select_first("#single-pager").is_some() {
            let sep = if url.contains('?') { '&' } else { '?' };
            get(&format!("{url}{sep}style=list")).send()?.html()
        } else {
            first
        };
        parse_pages::<T>(&doc)
    }

    fn related(&self, manga: &Manga) -> Result<Vec<Manga>> {
        let Some(id) = manga_id(manga) else { return Ok(Vec::new()) };
        let genres = related_genres(manga);
        if genres.is_empty() {
            return Ok(Vec::new());
        }
        let mut form = Form::default();
        form.add("action", "madara_load_more");
        form.add("page", "0");
        form.add("template", T::AJAX_TEMPLATE);
        form.add("vars[posts_per_page]", &T::PAGE_SIZE.to_string());
        form.add("vars[template]", "archive");
        form.add("vars[post_type]", "wp-manga");
        form.add("vars[post_status]", "publish");
        form.add("vars[orderby]", "rand");
        form.add("vars[sidebar]", "right");
        form.add("vars[manga_archives_item_layout]", "big_thumbnail");
        form.add("vars[post__not_in][0]", &id);
        form.add("vars[tax_query][0][taxonomy]", "wp-manga-genre");
        form.add("vars[tax_query][0][field]", "slug");
        for (i, g) in genres.iter().enumerate() {
            form.add(&format!("vars[tax_query][0][terms][{i}]"), &g.slug);
        }
        Ok(parse_archive::<T>(&form.post::<T>("/wp-admin/admin-ajax.php")?.html()))
    }
}

// ------------------------------------------------------------------ form bodies

/// An `x-www-form-urlencoded` body built field by field (Madara's `vars[...]` lists run to dozens).
#[derive(Default)]
struct Form(Vec<(String, String)>);

impl Form {
    fn add(&mut self, key: &str, value: &str) {
        self.0.push((key.into(), value.into()));
    }

    fn post<T: Theme>(&self, path: &str) -> Result<Response> {
        let fields: Vec<(&str, &str)> = self.0.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        post(&format!("{}{path}", base::<T>())).header("X-Requested-With", "XMLHttpRequest").form(&fields).send()
    }
}

#[derive(PartialEq, Clone, Copy)]
enum Browse {
    Popular,
    Latest,
    Search,
}

/// A page of the archive without the ajax endpoint: `/<manga>/page/<n>/?m_orderby=<order>` or a genre's own path.
fn archive_page<T: Theme>(page: u32, mode: Browse, query: &str, filters: &Filters) -> Result<MangaPage> {
    let order = match mode {
        Browse::Popular => "views",
        Browse::Latest => "latest",
        Browse::Search => ORDER_BY.get(filters.index("order")).map(|o| o.1).unwrap_or(""),
    };
    let mut path = format!("/{}/", T::MANGA_SUB_STRING);
    if mode == Browse::Search {
        if let Some(genre) = genre_routes(filter_data().as_ref()).into_iter().find(|g| filters.included("genres").contains(&g.slug.as_str())) {
            path = genre.path;
        }
    }
    let mut url = Url::parse(&format!("{}{path}", base::<T>())).map_err(|e| e.to_string())?;
    if page > 1 {
        url.path_segments_mut().map_err(|_| "bad base address")?.pop_if_empty().extend(["page", &page.to_string(), ""]);
    }
    let searching = mode == Browse::Search && !query.trim().is_empty();
    if searching {
        // a text search is the site's own search page, not an archive
        let mut s = Url::parse(&format!("{}/", base::<T>())).map_err(|e| e.to_string())?;
        if page > 1 {
            if T::SUPPORTS_POST_ID {
                return Ok(MangaPage { mangas: Vec::new(), has_next: false });
            }
            s.path_segments_mut().map_err(|_| "bad base address")?.pop_if_empty().extend(["page", &page.to_string(), ""]);
        }
        s.query_pairs_mut().append_pair(T::SEARCH_QUERY_PARAMETER, query.trim()).append_pair("post_type", "wp-manga");
        let doc = get(s.as_str()).send()?.html();
        let mangas = parse_archive::<T>(&doc);
        let has_next = !T::SUPPORTS_POST_ID && doc.select_first(T::NEXT_PAGE_SELECTOR).is_some();
        return Ok(MangaPage { mangas, has_next });
    }
    if !order.is_empty() {
        url.query_pairs_mut().append_pair(T::ORDER_QUERY_PARAMETER, order);
    }
    let doc = get(url.as_str()).send()?.html();
    Ok(MangaPage { mangas: parse_archive::<T>(&doc), has_next: doc.select_first(T::NEXT_PAGE_SELECTOR).is_some() })
}

fn ajax_list<T: Theme>(page: u32, mode: Browse, query: &str, filters: &Filters) -> Result<MangaPage> {
    if T::NO_AJAX {
        return archive_page::<T>(page, mode, query, filters);
    }
    let mut form = Form::default();
    form.add("action", "madara_load_more");
    form.add("page", &page.saturating_sub(1).to_string());
    form.add("template", T::AJAX_TEMPLATE);
    form.add("vars[paged]", "1");
    form.add("vars[template]", "archive");
    form.add("vars[posts_per_page]", &T::PAGE_SIZE.to_string());
    form.add("vars[post_type]", "wp-manga");
    form.add("vars[post_status]", "publish");
    form.add("vars[manga_archives_item_layout]", "big_thumbnail");
    if T::FILTER_NON_MANGA_ITEMS {
        form.add("vars[meta_query][0][key]", "_wp_manga_chapter_type");
        form.add("vars[meta_query][0][value]", "manga");
    }
    match mode {
        Browse::Popular => sort(&mut form, "_wp_manga_views"),
        Browse::Latest => sort(&mut form, "_latest_update"),
        Browse::Search => add_filters(&mut form, query, filters, u32::from(T::FILTER_NON_MANGA_ITEMS)),
    }
    let mangas = parse_archive::<T>(&form.post::<T>("/wp-admin/admin-ajax.php")?.html());
    let has_next = mangas.len() as u32 == T::PAGE_SIZE;
    Ok(MangaPage { mangas, has_next })
}

fn sort(form: &mut Form, key: &str) {
    form.add("vars[orderby]", "meta_value_num");
    form.add("vars[meta_key]", key);
    form.add("vars[order]", "DESC");
}

fn add_filters(form: &mut Form, query: &str, filters: &Filters, first_meta: u32) {
    if !query.trim().is_empty() {
        form.add("vars[s]", query);
    }
    let (mut meta, mut tax) = (first_meta, 0u32);
    let genres = filters.included("genres");

    for (key, taxonomy) in [("author", "wp-manga-author"), ("artist", "wp-manga-artist"), ("release", "wp-manga-release")] {
        let text = filters.text(key);
        if !text.trim().is_empty() {
            form.add(&format!("vars[tax_query][{tax}][taxonomy]"), taxonomy);
            form.add(&format!("vars[tax_query][{tax}][field]"), "name");
            form.add(&format!("vars[tax_query][{tax}][terms]"), text);
            tax += 1;
        }
    }

    let statuses = filters.included("status");
    if !statuses.is_empty() {
        form.add(&format!("vars[meta_query][{meta}][key]"), "_wp_manga_status");
        form.add(&format!("vars[meta_query][{meta}][compare]"), "IN");
        for (i, s) in statuses.iter().enumerate() {
            form.add(&format!("vars[meta_query][{meta}][value][{i}]"), s);
        }
        meta += 1;
    }

    match filters.select_value("order", &ORDER_BY) {
        "latest" => sort(form, "_latest_update"),
        "alphabet" => {
            form.add("vars[orderby]", "post_title");
            form.add("vars[order]", "ASC");
        }
        "rating" => {
            form.add("vars[meta_query][query_average_reviews][key]", "_manga_avarage_reviews");
            form.add("vars[meta_query][query_average_reviews][compare]", "EXISTS");
            form.add("vars[meta_query][query_total_reviews][key]", "_manga_total_votes");
            form.add("vars[meta_query][query_total_reviews][compare]", "EXISTS");
            form.add("vars[orderby][query_average_reviews]", "DESC");
            form.add("vars[orderby][query_total_reviews]", "DESC");
        }
        "trending" => sort(form, "_wp_manga_week_views_value"),
        "views" => sort(form, "_wp_manga_views"),
        "new-manga" => {
            form.add("vars[orderby]", "date");
            form.add("vars[order]", "DESC");
        }
        _ => {}
    }

    let adult = filters.index("adult");
    if adult != 0 {
        form.add(&format!("vars[meta_query][{meta}][key]"), "manga_adult_content");
        form.add(&format!("vars[meta_query][{meta}][compare]"), if adult == 1 { "not exists" } else { "exists" });
    }

    if !genres.is_empty() {
        if filters.select_value("genre_condition", &GENRE_CONDITION) == "1" {
            form.add(&format!("vars[tax_query][{tax}][operation]"), "AND");
        }
        form.add(&format!("vars[tax_query][{tax}][taxonomy]"), "wp-manga-genre");
        form.add(&format!("vars[tax_query][{tax}][field]"), "slug");
        for (i, g) in genres.iter().enumerate() {
            form.add(&format!("vars[tax_query][{tax}][terms][{i}]"), g);
        }
    }
}

// ------------------------------------------------------------------ archive and details

fn parse_archive<T: Theme>(doc: &Document) -> Vec<Manga> {
    doc.select(T::ARCHIVE_SELECTOR)
        .iter()
        .filter_map(|el| {
            let id = if T::SUPPORTS_POST_ID {
                let own = Some(el.attr("data-post-id")).filter(|s| !s.trim().is_empty());
                own.or_else(|| el.select_first("[data-post-id]").map(|e| e.attr("data-post-id")).filter(|s| !s.trim().is_empty()))?
            } else {
                String::new()
            };
            archive_manga::<T>(&el, &id)
        })
        .collect()
}

fn archive_manga<T: Theme>(el: &El, id: &str) -> Option<Manga> {
    let link = el.select_first(T::ARCHIVE_URL_SELECTOR)?;
    let href = link.abs_url("href");
    if href.trim().is_empty() {
        return None;
    }
    let path = path_only(&href);
    let title = T::ARCHIVE_TITLE_SELECTOR.and_then(|s| el.select_first(s)).map(|t| t.text()).unwrap_or_else(|| link.text());
    Some(Manga {
        url: if T::SUPPORTS_POST_ID { id.to_string() } else { path.clone() },
        title,
        thumbnail_url: T::process_thumbnail(el.select_first("img").and_then(|i| T::image_from_element(&i)), true),
        memo: Some(manga_memo(&path, &[], None)),
        ..Default::default()
    })
}

fn manga_memo(path: &str, genres: &[GenreRoute], legacy_id: Option<&str>) -> Value {
    let mut m = serde_json::Map::new();
    m.insert("path".into(), json!(path));
    if !genres.is_empty() {
        m.insert("genres".into(), json!(genres));
    }
    if let Some(id) = legacy_id {
        m.insert("id".into(), json!(id));
    }
    Value::Object(m)
}

fn memo_path(manga: &Manga) -> Option<String> {
    manga.memo.as_ref()?["path"].as_str().map(str::to_string)
}

fn is_number(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
}

/// The post id of a title: its stored address when that is a number, else the one kept in the memo.
fn manga_id(manga: &Manga) -> Option<String> {
    if is_number(&manga.url) {
        return Some(manga.url.clone());
    }
    manga.memo.as_ref()?["id"].as_str().map(str::to_string)
}

/// The post id a title page states about itself.
fn manga_id_of(doc: &Document) -> Option<String> {
    let non_blank = |s: String| Some(s).filter(|s| !s.trim().is_empty());
    doc.select_first("[id^=manga-chapters-holder]")
        .and_then(|e| non_blank(e.attr("data-id")))
        .or_else(|| doc.select_first("input.rating-post-id").and_then(|e| non_blank(e.attr("value"))))
        .or_else(|| doc.select_first("a[data-post]").and_then(|e| non_blank(e.attr("data-post"))))
        .or_else(|| {
            let href = doc.select_first("link[rel=shortlink]")?.attr("href");
            let after = href.split_once("?p=")?.1;
            non_blank(after.split('&').next().unwrap_or_default().to_string())
        })
}

/// The title page: by its stored path, else by `?p=<id>` (the site redirects to the current address).
fn details_document<T: Theme>(manga: &Manga) -> Result<Document> {
    let b = base::<T>();
    let id = manga_id(manga);
    let path = memo_path(manga).or_else(|| Some(manga.url.clone()).filter(|u| u.starts_with('/')));
    if let Some(path) = path {
        let resp = get(&format!("{b}{path}")).allow_errors().send()?;
        if (200..300).contains(&resp.status) {
            return Ok(resp.html());
        }
    }
    let id = id.ok_or("Missing Madara post ID from redirection")?;
    Ok(get(&format!("{b}/?p={id}")).send()?.html())
}

fn parse_details<T: Theme>(doc: &Document, id: &str, stored_url: &str) -> Result<Manga> {
    let path = path_only(doc.location());
    let updating = Regex::new(T::UPDATING_REGEX).map_err(|e| e.to_string())?;
    let is_updating = |s: &str| updating.is_match(s).unwrap_or(false);
    let names = |sel: &str| -> Option<String> {
        let v: Vec<String> = doc.select(sel).iter().map(|e| e.text()).filter(|t| !is_updating(t)).collect();
        Some(v.join(", ")).filter(|s| !s.trim().is_empty())
    };

    // a numeric stored address stays; a legacy path-based one is kept and the id goes to the memo
    let legacy = !is_number(stored_url) && !stored_url.is_empty() && T::SUPPORTS_POST_ID;
    let url = if !T::SUPPORTS_POST_ID {
        path.clone()
    } else if legacy {
        stored_url.to_string()
    } else {
        id.to_string()
    };
    let title = doc.select_first(T::DETAILS_TITLE).ok_or("the page has no title (the site changed?)")?.own_text();

    let mut description = doc.select_first(T::DETAILS_DESCRIPTION).map(|e| {
        let paragraphs: Vec<String> = e.select("p").iter().map(|p| p.text()).collect();
        if paragraphs.is_empty() { e.text() } else { paragraphs.join("\n\n") }
    });
    if let Some(alt) = doc.select_first(T::ALT_NAME).map(|e| e.own_text()).filter(|a| !a.is_empty() && !is_updating(a)) {
        let parts: Vec<String> = description.into_iter().chain([format!("Alternative Names: {alt}")]).collect();
        description = Some(parts.join("\n\n"));
    }

    let mut genres = Vec::<GenreRoute>::new();
    for a in doc.select(T::DETAILS_GENRE).iter() {
        let href = a.abs_url("href");
        if href.trim().is_empty() {
            continue;
        }
        let p = path_only(&href);
        let slug = p.trim_end_matches('/').rsplit('/').next().unwrap_or_default().to_string();
        if !slug.is_empty() {
            genres.push(GenreRoute { name: a.text(), slug, path: p });
        }
    }
    let mut genre: Vec<String> = genres.iter().map(|g| g.name.clone()).collect();
    genre.extend(doc.select(T::DETAILS_TAG).iter().map(|e| e.text()));
    if let Some(t) = doc.select_first(T::SERIES_TYPE).map(|e| e.text()).filter(|t| !t.is_empty()) {
        genre.push(t);
    }
    let mut seen = Vec::<String>::new();
    genre.retain(|g| {
        let k = g.to_lowercase();
        !seen.contains(&k) && {
            seen.push(k);
            true
        }
    });

    Ok(Manga {
        url,
        title,
        author: names(T::DETAILS_AUTHOR),
        artist: names(T::DETAILS_ARTIST),
        description,
        thumbnail_url: T::process_thumbnail(doc.select_first(T::DETAILS_THUMBNAIL).and_then(|i| T::image_from_element(&i)), false),
        status: doc.select(T::DETAILS_STATUS).iter().last().map(|e| to_status(&e.text())).unwrap_or(0),
        genre,
        initialized: true,
        memo: Some(manga_memo(&path, &genres, (T::SUPPORTS_POST_ID && !id.is_empty()).then_some(id))),
        ..Default::default()
    })
}

/// `https://site/manga/x/` -> `/manga/x/` (what `encodedPath` is).
fn path_only(url: &str) -> String {
    let rest = url.split_once("://").map(|x| x.1).unwrap_or(url);
    let path = match rest.find('/') {
        Some(i) if url.contains("://") => &rest[i..],
        _ if url.contains("://") => "/",
        _ => rest,
    };
    path.split(['?', '#']).next().unwrap_or(path).to_string()
}

fn to_status(s: &str) -> u8 {
    let s = s.to_lowercase();
    let any = |words: &[&str]| words.iter().any(|w| s.contains(w));
    if any(&["completed", "completo", "completado", "concluído", "concluido", "finalizado", "achevé", "terminé", "hoàn thành", "مكتملة", "مكتمل", "已完结", "tamamlandı", "đã hoàn thành", "завершено", "tamamlanan", "complété"]) || s == "end" {
        2
    } else if any(&[
        "ongoing", "on going", "updating", "продолжается", "em lançamento", "em andamento", "en cours", "ativo", "lançando", "đang tiến hành", "còn nữa",
        "devam ediyor", "in corso", "in arrivo", "مستمرة", "مستمر", "en curso", "emision", "curso", "en marcha", "publicandose", "publicándose",
        "en emision", "连载中", "đang làm", "em postagem", "devam eden", "em progresso", "atualizações semanais", "виходить", "đang ra",
    ]) {
        1
    } else if any(&["on hold", "hiatus", "pausado", "en espera", "durduruldu", "beklemede", "đang chờ", "متوقف", "en pause", "заморожено", "en attente", "tạm dừng"]) {
        6
    } else if any(&["canceled", "cancelled", "cancelado", "iptal edildi", "đã hủy", "ملغي", "abandonné", "заброшено", "annulé"]) {
        5
    } else {
        0
    }
}

fn genre_routes(data: Option<&Value>) -> Vec<GenreRoute> {
    data.and_then(|d| serde_json::from_value(d.clone()).ok()).unwrap_or_default()
}

/// Up to three genres of the title that say something more than "manga" or "webtoon".
fn related_genres(manga: &Manga) -> Vec<GenreRoute> {
    let all: Vec<GenreRoute> = manga.memo.as_ref().and_then(|m| serde_json::from_value(m["genres"].clone()).ok()).unwrap_or_default();
    let specific: Vec<GenreRoute> = all.iter().filter(|g| !GENERIC_GENRES.contains(&g.slug.to_lowercase().as_str())).cloned().collect();
    let chosen = if specific.is_empty() { all } else { specific };
    chosen.into_iter().take(3).collect()
}

// ------------------------------------------------------------------ chapters

fn fetch_chapter_list<T: Theme>(manga_path: &str, id: &str, page: Option<&Document>) -> Result<Vec<Chapter>> {
    match T::CHAPTER_MODE {
        ChapterMode::MangaPage => Ok(parse_chapter_list::<T>(page.ok_or("Manga page is required for this chapter mode")?, manga_path)),
        ChapterMode::AdminAjax => {
            let mut form = Form::default();
            form.add("action", "manga_get_chapters");
            form.add("manga", id);
            Ok(parse_chapter_list::<T>(&form.post::<T>("/wp-admin/admin-ajax.php")?.html(), manga_path))
        }
        ChapterMode::MangaAjax => Ok(parse_chapter_list::<T>(&Form::default().post::<T>(&chapter_ajax_path(manga_path))?.html(), manga_path)),
        ChapterMode::MangaAjaxPaginated => {
            let mut result: Vec<Chapter> = Vec::new();
            let mut last_url: Option<String> = None;
            for n in 1.. {
                let resp = post(&format!("{}{}?t={n}", base::<T>(), chapter_ajax_path(manga_path))).header("X-Requested-With", "XMLHttpRequest").allow_errors().send()?;
                if resp.status == 404 {
                    break;
                }
                if !(200..300).contains(&resp.status) {
                    return Err(format!("HTTP {}", resp.status));
                }
                let chapters = parse_chapter_list::<T>(&resp.html(), manga_path);
                if chapters.is_empty() || chapters.last().map(|c| c.url.clone()) == last_url {
                    break;
                }
                last_url = chapters.last().map(|c| c.url.clone());
                result.extend(chapters);
            }
            Ok(result)
        }
        ChapterMode::MangaAjaxQuery => {
            let slug = manga_path.trim_end_matches('/').rsplit('/').next().unwrap_or_default();
            let mut form = Form::default();
            form.add("manga-core", slug);
            form.add("manga_ajax", "1");
            form.add("maction", "get_chapters");
            Ok(parse_chapter_list::<T>(&form.post::<T>("/index.php")?.html(), manga_path))
        }
    }
}

fn chapter_ajax_path(manga_path: &str) -> String {
    format!("{}/ajax/chapters/", manga_path.trim_end_matches('/'))
}

fn parse_chapter_list<T: Theme>(doc: &Document, manga_path: &str) -> Vec<Chapter> {
    let now = now_ms();
    doc.select(T::CHAPTER_LIST_SELECTOR)
        .iter()
        .filter_map(|el| {
            let link = el.select_first(T::CHAPTER_URL_SELECTOR)?;
            let href = link.abs_url("href");
            if href.trim().is_empty() {
                return None;
            }
            let slug = path_only(&href).trim_end_matches('/').rsplit('/').next()?.to_string();
            if slug.is_empty() {
                return None;
            }
            let date = el
                .select_first("img:not(.thumb)")
                .map(|i| i.attr("alt"))
                .filter(|s| !s.trim().is_empty())
                .or_else(|| el.select_first("span a").map(|a| a.attr("title")).filter(|s| !s.trim().is_empty()))
                .or_else(|| el.select_first(T::CHAPTER_DATE_SELECTOR).map(|d| d.text()));
            Some(Chapter {
                url: slug,
                name: T::CHAPTER_NAME_SELECTOR.and_then(|s| el.select_first(s)).map(|n| n.text()).unwrap_or_else(|| link.text()),
                date_upload: parse_chapter_date(date.as_deref(), T::CHAPTER_DATE_PATTERN, now),
                memo: Some(json!({ "mangaPath": manga_path })),
                ..Default::default()
            })
        })
        .collect()
}

/// "Today", "3 days ago" (in several languages) or an absolute date; 0 when it is none of those.
fn parse_chapter_date(date: Option<&str>, pattern: &str, now: i64) -> i64 {
    let Some(value) = date else { return 0 };
    let normalized = value.to_lowercase();
    let midnight = now - now.rem_euclid(86_400_000);
    if ["today", "hoje", "hoy"].iter().any(|w| normalized.starts_with(w)) {
        return midnight;
    }
    if ["yesterday", "ontem", "ayer", "يوم واحد"].iter().any(|w| normalized.starts_with(w)) {
        return midnight - 86_400_000;
    }
    if let Some(amount) = first_number(&normalized) {
        let has = |words: &[&str]| contains_word(&normalized, words);
        let unit = if has(&["year", "año", "ano", "năm", "yıl", "سنة", "سنوات"]) {
            Some(Unit::Year)
        } else if has(&["month", "mes", "tháng", "ay", "شهر", "أشهر", "شهور"]) {
            Some(Unit::Month)
        } else if has(&["week", "semana", "tuần", "hafta", "أسبوع", "أسابيع"]) {
            Some(Unit::Week)
        } else if has(&["day", "día", "dia", "jour", "hari", "gün", "ngày", "giorni", "أيام", "天"]) {
            Some(Unit::Day)
        } else if has(&["hour", "hora", "heure", "jam", "saat", "giờ", "ore", "ساعة", "ساعات", "小时"]) {
            Some(Unit::Hour)
        } else if has(&["minute", "minuto", "min", "menit", "dakika", "phút", "دقيقة", "دقائق"]) {
            Some(Unit::Minute)
        } else if has(&["second", "segundo", "sec", "detik", "giây", "ثانية", "ثوان"]) {
            Some(Unit::Second)
        } else {
            None
        };
        if let Some(unit) = unit {
            return minus(now, amount, unit);
        }
    }
    parse_date(value, pattern, &lang()).unwrap_or(0)
}

/// The first run of digits in `s`. (Hand-rolled: a regex compiled per chapter costs more than the rest of the page.)
fn first_number(s: &str) -> Option<i64> {
    let start = s.find(|c: char| c.is_ascii_digit())?;
    s[start..].chars().take_while(char::is_ascii_digit).collect::<String>().parse().ok()
}

/// Whether one of `words` appears in `s` not glued to a letter before it, or "ay" would match inside "mayıs" and "days".
fn contains_word(s: &str, words: &[&str]) -> bool {
    words.iter().any(|w| s.match_indices(w).any(|(i, _)| !s[..i].chars().next_back().is_some_and(char::is_alphabetic)))
}

// ------------------------------------------------------------------ pages

fn parse_pages<T: Theme>(doc: &Document) -> Result<Vec<Page>> {
    let location = doc.location().to_string();
    let page = |index: usize, url: String| {
        let mut p = Page::new(index as u32, url);
        p.headers = vec![("Referer".into(), location.clone())];
        p
    };
    let Some(protector) = doc.select_first("#chapter-protector-data") else {
        return Ok(doc
            .select(T::PAGE_LIST_SELECTOR)
            .iter()
            .enumerate()
            .filter_map(|(i, el)| el.select_first("img").and_then(|img| T::image_from_element(&img)).map(|u| page(i, u)))
            .collect());
    };
    let src = protector.attr("src");
    let script = match src.strip_prefix("data:text/javascript;base64,") {
        Some(b64) => String::from_utf8_lossy(&base64::engine::general_purpose::STANDARD.decode(b64.trim()).map_err(|e| e.to_string())?).into_owned(),
        None => protector.inner_html(),
    };
    let password = script.split_once("wpmangaprotectornonce='").and_then(|x| x.1.split_once("';")).map(|x| x.0).unwrap_or_default();
    let encrypted = script.split_once("chapter_data='").and_then(|x| x.1.split_once("';")).map(|x| x.0).unwrap_or_default().replace("\\/", "/");
    let data: ProtectorData = serde_json::from_str(&encrypted).map_err(|e| format!("bad chapter data: {e}"))?;
    let raw = decrypt_chapter_data(&data, password)?;
    // the plaintext is a JSON string that itself holds a JSON list
    let inner: String = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let images: Vec<String> = serde_json::from_str(&inner).map_err(|e| e.to_string())?;
    Ok(images.into_iter().enumerate().map(|(i, u)| page(i, u)).collect())
}

#[derive(Deserialize)]
struct ProtectorData {
    #[serde(rename = "ct")]
    ciphertext: String,
    #[serde(rename = "s")]
    salt: String,
}

/// CryptoJS-style AES-256-CBC: key and IV from MD5 rounds over password and salt (OpenSSL's `EVP_BytesToKey`).
fn decrypt_chapter_data(data: &ProtectorData, password: &str) -> Result<String> {
    let salt: Vec<u8> = (0..data.salt.len() / 2).filter_map(|i| u8::from_str_radix(data.salt.get(2 * i..2 * i + 2)?, 16).ok()).collect();
    let mut key_iv: Vec<u8> = Vec::new();
    let mut previous: Vec<u8> = Vec::new();
    while key_iv.len() < 48 {
        let mut h = Md5::new();
        h.update(&previous);
        h.update(password.as_bytes());
        h.update(&salt);
        previous = h.finalize().to_vec();
        key_iv.extend_from_slice(&previous);
    }
    let mut buf = base64::engine::general_purpose::STANDARD.decode(data.ciphertext.trim()).map_err(|e| e.to_string())?;
    let decrypted = cbc::Decryptor::<Aes256>::new_from_slices(&key_iv[..32], &key_iv[32..48])
        .map_err(|e| e.to_string())?
        .decrypt_padded_mut::<Pkcs7>(&mut buf)
        .map_err(|_| "the chapter could not be decrypted (wrong key or the site changed)".to_string())?;
    String::from_utf8(decrypted.to_vec()).map_err(|e| e.to_string())
}

/// `data-src`, `data-lazy-src`, ... then `srcset` (its widest candidate), then `src`; absolute.
pub fn default_image(el: &El) -> Option<String> {
    for name in ["data-src", "data-lazy-src", "data-lzl-src", "data-cfsrc", "data-manga-src"] {
        if !el.attr(name).trim().is_empty() {
            return Some(el.abs_url(name));
        }
    }
    let srcset = el.attr("srcset");
    if !srcset.trim().is_empty() {
        let mut best: Option<(f32, String)> = None;
        for candidate in srcset.split(',') {
            let mut parts = candidate.trim().split_whitespace();
            let Some(url) = parts.next() else { continue };
            let weight = parts.next().and_then(|d| d.trim_end_matches(['w', 'x']).parse::<f32>().ok()).unwrap_or(0.0);
            if best.as_ref().map_or(true, |(w, _)| weight >= *w) {
                best = Some((weight, url.to_string()));
            }
        }
        if let Some((_, url)) = best {
            let base = Url::parse(&el.abs_url("src")).ok();
            return Some(sk_source_sdk::html::resolve(base.as_ref(), &url)).filter(|s| !s.is_empty());
        }
    }
    Some(el.abs_url("src")).filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests;
