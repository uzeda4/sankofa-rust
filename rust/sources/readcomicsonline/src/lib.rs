//! Read Comics Online, written in Rust. Port of the Kotlin extension `en.readcomicsonline` (MMRCMS theme, but the
//! extension overrides nearly all of it, so the one site is ported directly; version id 1).
//! Lists and search are HTML pages, a title's details and chapters share one page, the reader lists every image.

use sk_source_sdk::html::{Document, El};
use sk_source_sdk::*;

const BASE: &str = "https://readcomicsonline.ru";

#[derive(Default)]
pub struct ReadComicsOnline;

impl Source for ReadComicsOnline {
    fn info(&self) -> Info {
        Info { name: "Read Comics Online".into(), lang: "en".into(), base_url: BASE.into(), supports_latest: true, ..Default::default() }
    }

    fn popular(&self, page: u32) -> Result<MangaPage> {
        comic_list(&format!("{BASE}/comic-list?sort=views&page={page}"), false)
    }

    fn latest(&self, page: u32) -> Result<MangaPage> {
        comic_list(&format!("{BASE}/comic-list?sort=latest&page={page}"), true)
    }

    fn search(&self, page: u32, query: &str, filters: &Filters) -> Result<MangaPage> {
        let url = format!(
            "{BASE}/advanced-search?name={}&page={page}&release={}&author={}",
            url_encode(query),
            url_encode(filters.text("release")),
            url_encode(filters.text("author")),
        );
        let doc = get(&url).send()?.html();
        let mangas = doc
            .select("div a:has(img)")
            .iter()
            .filter_map(|a| {
                Some(Manga {
                    url: path_of(&a.abs_url("href")),
                    title: a.select_first("p")?.text(),
                    thumbnail_url: Some(a.select_first("img")?.abs_url("src")),
                    ..Default::default()
                })
            })
            .collect();
        Ok(MangaPage { mangas, has_next: doc.select_first("span a[rel=next]").is_some() })
    }

    fn filters(&self, _data: Option<&Value>) -> Vec<FilterDef> {
        vec![FilterDef::text("release", "Year of release"), FilterDef::text("author", "Author")]
    }

    fn by_url(&self, url: &str) -> Result<Option<Manga>> {
        let Ok(u) = Url::parse(url) else { return Ok(None) };
        let segments: Vec<&str> = u.path_segments().map(|s| s.filter(|p| !p.is_empty()).collect()).unwrap_or_default();
        if u.host_str() != Url::parse(BASE).ok().as_ref().and_then(|b| b.host_str()) || segments.len() < 2 || segments[0] != "comic" {
            return Ok(None);
        }
        let stub = Manga { url: path_of(url), ..Default::default() };
        let doc = get(url).send()?.html();
        Ok(Some(parse_details(&doc, &stub)))
    }

    fn update(&self, manga: &Manga, fetch_details: bool, fetch_chapters: bool) -> Result<Update> {
        let doc = get(&absolute(BASE, &manga.url)).send()?.html();
        Ok(Update { manga: fetch_details.then(|| parse_details(&doc, manga)), chapters: fetch_chapters.then(|| parse_chapters(&doc)) })
    }

    fn details(&self, manga: &Manga) -> Result<Manga> {
        self.update(manga, true, false)?.manga.ok_or_else(|| "no details".into())
    }

    fn chapters(&self, manga: &Manga) -> Result<Vec<Chapter>> {
        self.update(manga, false, true)?.chapters.ok_or_else(|| "no chapters".into())
    }

    fn pages(&self, chapter: &Chapter) -> Result<Vec<Page>> {
        let doc = get(&absolute(BASE, &chapter.url)).send()?.html();
        Ok(doc.select("#reader-all img").iter().enumerate().map(|(i, img)| Page::new(i as u32, image_attr(img))).collect())
    }
}

fn comic_list(url: &str, distinct: bool) -> Result<MangaPage> {
    let doc = get(url).send()?.html();
    let mut mangas: Vec<Manga> = Vec::new();
    for el in doc.select("div.comic-list-layout .grid > .group").iter() {
        let Some(anchor) = el.select_first("a.block.text-sm.font-semibold") else { continue };
        let path = path_of(&anchor.abs_url("href"));
        if distinct && mangas.iter().any(|m| m.url == path) {
            continue;
        }
        let cover = el.select_first("img").map(|i| i.attr("src"));
        mangas.push(Manga { thumbnail_url: Some(guess_cover(&path, cover.as_deref())), url: path, title: anchor.text(), ..Default::default() });
    }
    Ok(MangaPage { mangas, has_next: doc.select_first("nav a[rel=next]").is_some() })
}

/// The cover the site stores for a title when the list shows none (or its placeholder).
fn guess_cover(manga_url: &str, url: Option<&str>) -> String {
    match url {
        Some(u) if !u.is_empty() && !u.ends_with("no-image.png") => u.to_string(),
        _ => format!("{BASE}/uploads/manga/{}/cover/cover_250x350.jpg", manga_url.rsplit('/').next().unwrap_or_default()),
    }
}

/// The picture of an `<img>`, looking at the lazy-loading attributes first; absolute.
fn image_attr(el: &El) -> String {
    for name in ["data-background-image", "data-cfsrc", "data-lazy-src", "data-src"] {
        if el.has_attr(name) {
            return el.abs_url(name);
        }
    }
    el.abs_url("src")
}

fn parse_details(doc: &Document, previous: &Manga) -> Manga {
    let mut manga = Manga { url: previous.url.clone(), title: previous.title.clone(), initialized: true, ..Default::default() };
    if let Some(t) = doc.select_first("h1.text-2xl") {
        manga.title = t.text();
    }
    let cover = doc.select_first("img.w-full.rounded-xl").map(|i| image_attr(&i)).filter(|s| !s.is_empty());
    manga.thumbnail_url = Some(guess_cover(doc.location(), cover.as_deref()));
    manga.description = doc.select_first("div:has(> div > h3:contains(Synopsis)) > div > p").map(|p| p.text());
    manga.status = doc
        .select_first("div.flex.flex-wrap.gap-2 span.rounded-full")
        .map(|s| match s.text().to_lowercase().as_str() {
            "complete" | "مكتملة" | "complet" | "completo" | "zakończone" | "concluído" | "finalizado" => 2,
            "ongoing" | "مستمرة" | "en cours" | "em lançamento" | "prace w toku" | "ativo" | "em andamento" | "activo" | "publicándose" | "publicandose" => 1,
            "dropped" => 5,
            _ => 0,
        })
        .unwrap_or(0);
    manga.genre = doc.select("dl div:contains(Genres:) a").each_text();
    let authors = doc.select("div:has(span:contains(Author:)) > a").each_text().join(", ");
    manga.author = Some(authors).filter(|a| !a.is_empty());
    manga
}

fn parse_chapters(doc: &Document) -> Vec<Chapter> {
    let title = doc.select_first("h1.text-2xl").map(|t| t.text()).unwrap_or_default();
    doc.select(".overflow-hidden.border-ink-600 > a")
        .iter()
        .map(|a| {
            let name = a.select_first(".text-brand-400").map(|n| n.text()).unwrap_or_else(|| a.text());
            Chapter {
                url: path_of(&a.abs_url("href")),
                name: clean_chapter_name(&title, &name),
                date_upload: a.select_first(".text-slate-500").and_then(|d| parse_date(&d.text(), "d MMM yyyy", "en")).unwrap_or(0),
                ..Default::default()
            }
        })
        .collect()
}

/// Sites that write "Title 12 : Chapter 12" get "12: Chapter 12"; the title is dropped and a repeated half is not shown twice.
fn clean_chapter_name(manga_title: &str, name: &str) -> String {
    let initial = if manga_title.is_empty() { name.to_string() } else { name.replacen(manga_title, "", 1) };
    let mut parts = initial.splitn(2, ':').map(str::trim);
    match (parts.next().unwrap_or_default(), parts.next()) {
        (first, Some(second)) if first != second => format!("{first}: {second}"),
        (first, _) => first.to_string(),
    }
}

export_source!(ReadComicsOnline);

#[cfg(test)]
mod tests;
