//! Luscious, written in Rust. Port of the Kotlin extension `all.luscious`: the site's GraphQL API lists albums, an
//! album is a title and each of its pictures is a chapter of one page (or, with "Merge Chapter", chunks of up to 1000
//! pictures read as pages). One module serves every language it is installed for (`lang()`); Portuguese keeps the id
//! the extension fixed for it, the others derive it from the name and language.
//!
//! Not ported: the network interceptor that relabelled `application/octet-stream` WebP answers (the app's image
//! pipeline recognises the format from the bytes).

mod filters;

use filters::*;
use serde::Deserialize;
use sk_source_sdk::*;

const BASE: &str = "https://www.luscious.net";
const MEMBERS: &str = "https://members.luscious.net";
const CDN_HOST: &str = "ah-img.luscious.net";
const PORTUGUESE_ID: i64 = 5_826_725_746_643_311_801;

const RESOLUTION: &str = "resolution";
const PAGE_SORT: &str = "page_sort";
const MERGE_CHAPTERS: &str = "merge_chapters";

const ALBUM_LIST_QUERY: &str = "query AlbumList($input: AlbumListInput!) { album { list(input: $input) { info { page has_next_page } items } } }";
const ALBUM_PICTURES_QUERY: &str = "query AlbumListOwnPictures($input: PictureListInput!) { picture { list(input: $input) { info { total_items total_pages page has_next_page items_per_page } items { created title url_to_original url_to_video position thumbnails { url } } } } }";
const ALBUM_GET_QUERY: &str = "query AlbumGet($id: ID!) { album { get(id: $id) { ... on Album { ...AlbumStandard } ... on MutationError { errors { code message } } } } } fragment AlbumStandard on Album { __typename id title labels description created modified like_status number_of_favorites number_of_dislikes rating moderation_status marked_for_deletion marked_for_processing number_of_pictures number_of_animated_pictures number_of_duplicates slug is_manga url download_url permissions cover { width height size url } created_by { id url name display_name user_title avatar { url size } } content { id title url } language { id title url } tags { category text url count } genres { id title slug url } audiences { id title url url } last_viewed_picture { id position url } is_featured featured_date featured_by { id url name display_name user_title avatar { url size } } }";
const ALBUM_RELATED_QUERY: &str = "query AlbumListRelated($id: ID!) { album { list_related(id: $id) { more_like_this { ...AlbumInSearchList } items_liked_like_this { ...AlbumInSearchList } items_created_by_this_user { ...AlbumInSearchList } } } } fragment AlbumInSearchList on Album { title url cover { url } }";

// ------------------------------------------------------------------ API answers

#[derive(Deserialize)]
struct Answer<T> {
    data: T,
}

#[derive(Deserialize)]
struct AlbumListData {
    album: AlbumListAlbum,
}
#[derive(Deserialize)]
struct AlbumListAlbum {
    list: AlbumList,
}
#[derive(Deserialize)]
struct AlbumList {
    info: PageInfo,
    items: Vec<AlbumRef>,
}
#[derive(Deserialize)]
struct PageInfo {
    #[serde(default)]
    has_next_page: bool,
}
#[derive(Deserialize)]
struct AlbumRef {
    url: String,
    title: String,
    cover: Cover,
}
#[derive(Deserialize, Default, Clone)]
struct Cover {
    #[serde(default)]
    url: String,
    #[serde(default)]
    width: u32,
    #[serde(default)]
    height: u32,
}

#[derive(Deserialize)]
struct AlbumGetData {
    album: AlbumGet,
}
#[derive(Deserialize)]
struct AlbumGet {
    get: FullAlbum,
}
#[derive(Deserialize)]
struct FullAlbum {
    url: String,
    title: String,
    cover: Cover,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    language: Option<Titled>,
    #[serde(default)]
    labels: Vec<String>,
    #[serde(default)]
    genres: Vec<Titled>,
    #[serde(default)]
    audiences: Vec<Titled>,
    #[serde(default)]
    tags: Vec<Tag>,
    #[serde(default)]
    number_of_pictures: u32,
    #[serde(default)]
    number_of_animated_pictures: u32,
    content: Titled,
    #[serde(default)]
    created: Option<f64>,
}
#[derive(Deserialize)]
struct Titled {
    title: String,
}
#[derive(Deserialize)]
struct Tag {
    text: String,
}

#[derive(Deserialize)]
struct RelatedData {
    album: RelatedAlbum,
}
#[derive(Deserialize)]
struct RelatedAlbum {
    list_related: Related,
}
#[derive(Deserialize)]
struct Related {
    #[serde(default)]
    more_like_this: Option<Vec<AlbumRef>>,
    #[serde(default)]
    items_liked_like_this: Option<Vec<AlbumRef>>,
    #[serde(default)]
    items_created_by_this_user: Option<Vec<AlbumRef>>,
}

#[derive(Deserialize)]
struct PicturesData {
    picture: PicturesPicture,
}
#[derive(Deserialize)]
struct PicturesPicture {
    list: PicturesList,
}
#[derive(Deserialize)]
struct PicturesList {
    items: Vec<Picture>,
}
#[derive(Deserialize)]
struct Picture {
    #[serde(default)]
    thumbnails: Vec<Cover>,
    #[serde(default)]
    url_to_original: Option<String>,
    #[serde(default)]
    url_to_video: Option<String>,
    #[serde(default)]
    position: u32,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    created: f64,
}

// ------------------------------------------------------------------ the source

#[derive(Default)]
pub struct Luscious;

fn base() -> String {
    base_url(BASE)
}

fn api(operation: &str, query: &str, variables: &Value) -> String {
    format!(
        "{}/graphql/nobatch/?operationName={operation}&query={}&variables={}",
        base(),
        url_encode(query),
        url_encode(&variables.to_string())
    )
}

fn fetch<T: for<'de> Deserialize<'de>>(operation: &str, query: &str, variables: &Value) -> Result<T> {
    Ok(get(&api(operation, query, variables)).send()?.json::<Answer<T>>()?.data)
}

fn album_list(page: u32, sort: &str, filters: &Filters, query: &str) -> Result<MangaPage> {
    let list = album_list_filters(&lang(), filters, query)?.into_iter().map(|(name, value)| json!({ "name": name, "value": value })).collect::<Vec<_>>();
    let variables = json!({ "input": { "display": sort, "page": page, "filters": list } });
    let list = fetch::<AlbumListData>("AlbumList", ALBUM_LIST_QUERY, &variables)?.album.list;
    Ok(MangaPage { mangas: list.items.into_iter().map(album_ref).collect(), has_next: list.info.has_next_page })
}

fn album_ref(a: AlbumRef) -> Manga {
    Manga { url: a.url, title: a.title, thumbnail_url: Some(a.cover.url), ..Default::default() }
}

/// The id at the end of an album address: `/albums/name_123/` -> `123`.
fn album_id(url: &str) -> String {
    url.split('?').next().unwrap_or_default().trim_end_matches('/').rsplit('_').next().unwrap_or_default().to_string()
}

fn album_info(id: &str) -> Result<FullAlbum> {
    Ok(fetch::<AlbumGetData>("AlbumGet", ALBUM_GET_QUERY, &json!({ "id": id }))?.album.get)
}

fn details_of(album: &FullAlbum) -> Manga {
    let mut genre: Vec<String> = Vec::new();
    genre.extend(album.language.iter().map(|l| l.title.clone()));
    genre.extend(album.labels.iter().cloned());
    genre.extend(album.genres.iter().map(|g| g.title.clone()));
    genre.extend(album.audiences.iter().map(|a| a.title.clone()));
    genre.extend(album.tags.iter().map(|t| t.text.clone()));
    genre.push(album.content.title.clone());
    let artist = album.tags.iter().find(|t| t.text.contains("Artist:")).map(|t| t.text.split_once(':').map(|x| x.1).unwrap_or_default().trim().to_string());
    Manga {
        url: album.url.clone(),
        title: album.title.clone(),
        thumbnail_url: Some(album.cover.url.clone()),
        description: Some(format!(
            "{}\n\nPictures: {}\nAnimated Pictures: {}",
            album.description.clone().unwrap_or_default(),
            album.number_of_pictures,
            album.number_of_animated_pictures
        )),
        author: artist.clone(),
        artist,
        genre,
        initialized: true,
        ..Default::default()
    }
}

fn pictures_page(id: &str, page: u32) -> Result<Vec<Picture>> {
    let variables = json!({
        "input": { "filters": [{ "name": "album_id", "value": id }], "display": pref_str(PAGE_SORT, "position"), "page": page, "items_per_page": 50 }
    });
    Ok(fetch::<PicturesData>("AlbumListOwnPictures", ALBUM_PICTURES_QUERY, &variables)?.picture.list.items)
}

/// The address of a picture at the quality the user chose; the original (or the animation as a GIF) by default.
fn picture_url(p: &Picture) -> String {
    let url = match pref_str(RESOLUTION, "-1").parse::<i32>() {
        Ok(i) if i >= 0 && (i as usize) < p.thumbnails.len() => p.thumbnails[i as usize].url.clone(),
        _ => match (&p.url_to_video, &p.url_to_original) {
            (Some(v), _) => v.replace(".mp4", ".gif"),
            (None, Some(o)) => o.clone(),
            _ => p.thumbnails.iter().max_by_key(|t| t.width * t.height).map(|t| t.url.clone()).unwrap_or_default(),
        },
    };
    if url.starts_with("//") { format!("https:{url}") } else { url }
}

impl Source for Luscious {
    fn info(&self) -> Info {
        let lang = lang();
        Info {
            name: "Luscious".into(),
            id: if lang == "pt-BR" { PORTUGUESE_ID } else { 0 },
            lang,
            base_url: BASE.into(),
            mirrors: vec![BASE.into(), MEMBERS.into()],
            supports_latest: true,
            supports_related: true,
            custom_urls: true,
            prefs: vec![
                PrefDef::list(RESOLUTION, "Image resolution", &[("Low", "2"), ("Medium", "1"), ("High", "0"), ("Original", "-1")], "-1"),
                PrefDef::list(PAGE_SORT, "Page Sort", &[("Position", "position"), ("Date", "date_newest"), ("Rating", "rating_all_time")], "position"),
                PrefDef::switch(
                    MERGE_CHAPTERS,
                    "Merge Chapter",
                    "If checked, merges all content of one album into chapters of up to 1000 images each, labeled as 'Merged Chapter (Part 1)', 'Merged Chapter (Part 2)', and so on. Note: you must be logged into the WebView to access more than 1000 images.",
                    false,
                ),
            ],
            ..Default::default()
        }
    }

    fn popular(&self, page: u32) -> Result<MangaPage> {
        album_list(page, POPULAR_SORT, &Filters::default(), "")
    }

    fn latest(&self, page: u32) -> Result<MangaPage> {
        album_list(page, LATEST_SORT, &Filters::default(), "")
    }

    fn search(&self, page: u32, query: &str, filters: &Filters) -> Result<MangaPage> {
        let options = sort_options(current_year());
        let sort = options.get(filters.index("sort")).map(|o| o.1.as_str()).unwrap_or(POPULAR_SORT);
        album_list(page, sort, filters, query)
    }

    fn filters(&self, _data: Option<&Value>) -> Vec<FilterDef> {
        filter_defs(&lang())
    }

    fn by_url(&self, url: &str) -> Result<Option<Manga>> {
        let Ok(u) = Url::parse(url) else { return Ok(None) };
        let own_domain = Url::parse(BASE).ok().and_then(|b| b.host_str().and_then(|h| h.split_once('.').map(|x| x.1.to_string()))).unwrap_or_default();
        if !u.host_str().is_some_and(|h| h.ends_with(&own_domain)) {
            return Ok(None);
        }
        let segments: Vec<&str> = u.path_segments().map(|s| s.collect()).unwrap_or_default();
        if segments.first() != Some(&"albums") {
            return Ok(None);
        }
        let Some(slug) = segments.get(1) else { return Ok(None) };
        let id = slug.rsplit('_').next().unwrap_or_default();
        Ok(Some(details_of(&album_info(id)?)))
    }

    fn manga_url(&self, _manga: &Manga) -> Option<String> {
        None
    }

    fn chapter_url(&self, chapter: &Chapter) -> Option<String> {
        Some(if chapter.url.starts_with("/albums/") {
            format!("{}{}", base(), chapter.url.split('?').next().unwrap_or_default())
        } else {
            format!("https://{CDN_HOST}{}", chapter.url)
        })
    }

    fn update(&self, manga: &Manga, fetch_details: bool, fetch_chapters: bool) -> Result<Update> {
        let id = album_id(&manga.url);
        let album = album_info(&id)?;
        let chapters = if !fetch_chapters {
            None
        } else if pref_bool(MERGE_CHAPTERS, false) {
            Some(merged_chapters(manga, &album))
        } else {
            Some(picture_chapters(&id, &album)?)
        };
        let details = fetch_details.then(|| Manga { url: manga.url.clone(), ..details_of(&album) });
        Ok(Update { manga: details, chapters })
    }

    fn details(&self, manga: &Manga) -> Result<Manga> {
        self.update(manga, true, false)?.manga.ok_or_else(|| "no details".into())
    }

    fn chapters(&self, manga: &Manga) -> Result<Vec<Chapter>> {
        self.update(manga, false, true)?.chapters.ok_or_else(|| "no chapters".into())
    }

    fn pages(&self, chapter: &Chapter) -> Result<Vec<Page>> {
        if !chapter.url.starts_with("/albums/") {
            return Ok(vec![Page::new(0, format!("https://{CDN_HOST}{}", chapter.url))]);
        }
        let chunk = chapter.url.split_once("?chunk=").map(|x| x.1).unwrap_or("1").split('#').next().unwrap_or("1").parse::<u32>().unwrap_or(1).max(1);
        let id = album_id(&chapter.url);
        let mut pages: Vec<Page> = Vec::new();
        for page in ((chunk - 1) * 20 + 1)..=(chunk * 20) {
            let pictures = pictures_page(&id, page)?;
            if pictures.is_empty() {
                break;
            }
            let count = pictures.len();
            for p in &pictures {
                pages.push(Page::new(pages.len() as u32, on_cdn(&picture_url(p))));
            }
            if count < 50 {
                break;
            }
        }
        Ok(pages)
    }

    fn related(&self, manga: &Manga) -> Result<Vec<Manga>> {
        let related = fetch::<RelatedData>("AlbumListRelated", ALBUM_RELATED_QUERY, &json!({ "id": album_id(&manga.url) }))?.album.list_related;
        Ok([related.more_like_this, related.items_liked_like_this, related.items_created_by_this_user].into_iter().flatten().flatten().map(album_ref).collect())
    }
}

/// The picture's address on the image host (some answers name another one).
fn on_cdn(url: &str) -> String {
    match Url::parse(url) {
        Ok(mut u) => {
            let _ = u.set_host(Some(CDN_HOST));
            u.to_string()
        }
        Err(_) => url.to_string(),
    }
}

/// One chapter per 1000 pictures, newest part first.
fn merged_chapters(manga: &Manga, album: &FullAlbum) -> Vec<Chapter> {
    let chunks = album.number_of_pictures.div_ceil(1000).max(1);
    let date = (album.created.unwrap_or(0.0) as i64) * 1000;
    (1..=chunks)
        .rev()
        .map(|i| Chapter {
            url: format!("{}?chunk={i}", manga.url),
            name: if chunks == 1 { "Merged Chapter".into() } else { format!("Merged Chapter (Part {i})") },
            chapter_number: i as f32,
            date_upload: date,
            ..Default::default()
        })
        .collect()
}

/// Every picture is a chapter, newest (highest position) first. The API stops listing at 1000 pictures unless the user
/// is signed in on the site.
fn picture_chapters(id: &str, album: &FullAlbum) -> Result<Vec<Chapter>> {
    let mut chapters: Vec<Chapter> = Vec::new();
    for page in 1.. {
        let pictures = pictures_page(id, page)?;
        if pictures.is_empty() {
            break;
        }
        for p in &pictures {
            chapters.push(Chapter {
                url: path_of(&picture_url(p)),
                name: format!("{} - {}", p.position, p.title.clone().unwrap_or_default()),
                chapter_number: p.position as f32,
                date_upload: (p.created as i64) * 1000,
                ..Default::default()
            });
        }
        if page * 50 >= album.number_of_pictures {
            break;
        }
    }
    chapters.reverse();
    Ok(chapters)
}

export_source!(Luscious);

#[cfg(test)]
mod tests;
