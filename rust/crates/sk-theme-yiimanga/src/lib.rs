//! Yiimanga: small Yii2 manga and comic sites with `/page/<n>/` listings, a `/search/?s=` page and one HTML page that
//! holds a title's details and chapters. A site is a type that implements [`Theme`] with its title path.
//!
//! Port of the `yiimanga` multisrc of the Kotlin extensions.

use sk_source_sdk::html::{Document, El};
use sk_source_sdk::*;
use std::marker::PhantomData;

pub trait Theme: 'static {
    const NAME: &'static str;
    const LANG: &'static str = "en";
    const BASE_URL: &'static str;
    const VERSION_ID: u32 = 1;

    /// First segment of a title's address (`/manga/<slug>/`).
    const MANGA_PATH: &'static str;
    const SEARCH_PARAM: &'static str = "s";
    /// Catalog route the site sorts with `?order=views`; without one "popular" is the latest list.
    const POPULAR_PATH: Option<&'static str> = None;
    const SUPPORTS_FILTER_FETCHING: bool = false;
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

/// `<base>/seg/.../page/<n>/`
fn list_url<T: Theme>(segments: &[&str], page: u32) -> String {
    let mut url = base::<T>();
    for s in segments {
        url.push('/');
        url.push_str(s);
    }
    format!("{url}/page/{page}/")
}

/// The address without its host: what the app stores for a title or chapter.
fn path_of(abs: &str) -> String {
    match Url::parse(abs) {
        Ok(u) => format!("{}{}", u.path(), u.query().map(|q| format!("?{q}")).unwrap_or_default()),
        Err(_) => abs.to_string(),
    }
}

fn list_page(doc: &Document) -> MangaPage {
    let mangas = doc
        .select(".latest-list .latest-item, .list .list-item")
        .iter()
        .filter_map(|el| {
            let link = el.select_first(".latest-right a:has(h3, h4), .item-title a")?;
            Some(Manga {
                url: path_of(&link.abs_url("href")),
                title: link.text(),
                thumbnail_url: el.select_first(".latest-left img, .item-img img").map(|i| i.abs_url("src")).filter(|u| !u.is_empty()),
                ..Default::default()
            })
        })
        .collect();
    MangaPage { mangas, has_next: doc.select_first("#pagination li.next a").is_some() }
}

fn fetch(url: &str) -> Result<Document> {
    Ok(get(url).send()?.html())
}

/// The genre list the site gave (`[[name, slug], ...]`), with "All" first.
fn genre_options(data: Option<&Value>) -> Vec<(String, String)> {
    let mut v = vec![("All".to_string(), String::new())];
    if let Some(list) = data.and_then(Value::as_array) {
        for g in list {
            if let (Some(n), Some(s)) = (g[0].as_str(), g[1].as_str()) {
                v.push((n.to_string(), s.to_string()));
            }
        }
    }
    v
}

impl<T: Theme> Source for Site<T> {
    fn info(&self) -> Info {
        Info {
            name: T::NAME.into(),
            lang: T::LANG.into(),
            base_url: T::BASE_URL.into(),
            version_id: T::VERSION_ID,
            supports_latest: T::POPULAR_PATH.is_some(),
            supports_filter_fetching: T::SUPPORTS_FILTER_FETCHING,
            ..Default::default()
        }
    }

    fn popular(&self, page: u32) -> Result<MangaPage> {
        match T::POPULAR_PATH {
            None => self.latest(page),
            Some(path) => Ok(list_page(&fetch(&format!("{}?order=views", list_url::<T>(&[path], page)))?)),
        }
    }

    fn latest(&self, page: u32) -> Result<MangaPage> {
        Ok(list_page(&fetch(&list_url::<T>(&[], page))?))
    }

    fn search(&self, page: u32, query: &str, filters: &Filters) -> Result<MangaPage> {
        if !query.trim().is_empty() {
            let mut url = Url::parse(&format!("{}/search/", base::<T>())).map_err(|e| e.to_string())?;
            url.query_pairs_mut().append_pair(T::SEARCH_PARAM, query).append_pair("page", &page.to_string());
            return Ok(list_page(&fetch(url.as_str())?));
        }
        let options = genre_options(filter_data().as_ref());
        let slug = options.get(filters.index("genre")).map(|o| o.1.clone()).unwrap_or_default();
        if slug.is_empty() {
            return self.latest(page);
        }
        Ok(list_page(&fetch(&list_url::<T>(&["genre", &slug], page))?))
    }

    fn filters(&self, data: Option<&Value>) -> Vec<FilterDef> {
        let options = genre_options(data);
        let pairs: Vec<(&str, &str)> = options.iter().map(|(n, s)| (n.as_str(), s.as_str())).collect();
        vec![FilterDef::header("Ignored when using text search"), FilterDef::select("genre", "Genre", &pairs, 0)]
    }

    fn filter_data(&self) -> Result<Option<Value>> {
        let doc = fetch(&base::<T>())?;
        let mut seen = std::collections::BTreeSet::new();
        let mut genres: Vec<(String, String)> = Vec::new();
        for a in doc.select(".genre-list a, a.item-nav[href*=/genre/], a.item-nav-mb[href*=/genre/]").iter() {
            let path = path_of(&a.abs_url("href"));
            let slug = path.trim_start_matches('/').split('/').nth(1).unwrap_or_default().to_string();
            if !slug.is_empty() && seen.insert(slug.clone()) {
                genres.push((a.text(), slug));
            }
        }
        genres.sort();
        Ok(Some(json!(genres.into_iter().map(|(n, s)| json!([n, s])).collect::<Vec<_>>())))
    }

    fn details(&self, manga: &Manga) -> Result<Manga> {
        let doc = fetch(&format!("{}{}", base::<T>(), manga.url))?;
        Ok(parse_details(&doc, &manga.url))
    }

    fn chapters(&self, manga: &Manga) -> Result<Vec<Chapter>> {
        let doc = fetch(&format!("{}{}", base::<T>(), manga.url))?;
        Ok(parse_chapters(&doc))
    }

    fn update(&self, manga: &Manga, fetch_details: bool, fetch_chapters: bool) -> Result<Update> {
        // one page holds both
        let doc = fetch(&format!("{}{}", base::<T>(), manga.url))?;
        Ok(Update {
            manga: fetch_details.then(|| parse_details(&doc, &manga.url)),
            chapters: fetch_chapters.then(|| parse_chapters(&doc)),
        })
    }

    fn pages(&self, chapter: &Chapter) -> Result<Vec<Page>> {
        let doc = fetch(&format!("{}{}", base::<T>(), chapter.url))?;
        Ok(doc
            .select(".reading-content img, .read-content img")
            .iter()
            .enumerate()
            .map(|(i, el)| Page { index: i as u32, url: String::new(), image_url: el.abs_url("src"), headers: Vec::new(), ops: Vec::new(), mirrors: Vec::new() })
            .collect())
    }
}

fn text_or_none(el: &El) -> Option<String> {
    Some(el.text().trim().to_string()).filter(|t| !t.is_empty())
}

/// The value cell of the row whose heading (`<h5>` or `.sh`) is one of `labels`.
fn info<'a>(doc: &'a Document, labels: &[&str]) -> Option<El<'a>> {
    let heading = doc
        .select(".main-info-list li > h5, .post-content_item .summary-heading > h5, .info-item > span.sh")
        .iter()
        .find(|h| {
            let t = h.text();
            let t = t.trim().trim_end_matches(':').trim_end_matches(" -").to_string();
            labels.contains(&t.as_str())
        })
        .cloned()?;
    let row = heading.closest("li, .post-content_item, .info-item")?;
    row.children().into_iter().find(|c| !c.is("h5, .summary-heading, .sh"))
}

/// Links in a row carry their own commas: use their own text; a row without links is plain text.
fn names(el: &El) -> Option<String> {
    let links = el.select("a");
    let parts: Vec<String> = if links.is_empty() {
        text_or_none(el).into_iter().collect()
    } else {
        links.iter().map(|l| l.own_text().trim().to_string()).filter(|t| !t.is_empty()).collect()
    };
    Some(parts.join(", ")).filter(|s| !s.is_empty())
}

fn parse_details(doc: &Document, url: &str) -> Manga {
    let status_text = info(doc, &["Status", "Estado"]).map(|e| e.text()).or_else(|| doc.select_first(".manga-detail .info p.type").map(|e| e.text()));
    let status = match status_text.map(|s| s.trim().to_lowercase()).as_deref() {
        Some("ongoing") => 1,
        Some("completed") => 2,
        _ => 0,
    };
    let synopsis = doc.select_first(".short-desc-content, .dsct, .description-summary").and_then(|block| {
        let parts = block.select("p, li");
        let texts: Vec<String> = if parts.is_empty() { vec![block.text()] } else { parts.iter().map(|p| p.text()).collect() };
        Some(texts.into_iter().map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect::<Vec<_>>().join("\n\n")).filter(|s| !s.is_empty())
    });
    let alt = info(doc, &["Alternative"]).and_then(|e| text_or_none(&e)).or_else(|| doc.select_first(".main-info-list li > h4, .manga-detail .info h6").and_then(|e| text_or_none(&e)));
    let description = [synopsis, alt.map(|a| format!("Alternative names: {a}"))].into_iter().flatten().collect::<Vec<_>>().join("\n\n");
    Manga {
        url: url.to_string(),
        title: doc.select_first("h1.main-info-title, .post-title h1, .manga-detail .info h1").map(|e| e.text()).unwrap_or_default(),
        thumbnail_url: doc.select_first("img.img-cover, .summary_image img, .manga-detail .poster img").map(|e| e.abs_url("src")).filter(|u| !u.is_empty()),
        author: info(doc, &["Author(s)", "Author", "Autores"]).and_then(|e| names(&e)),
        artist: info(doc, &["Artist(s)", "Artist", "Artistas"]).and_then(|e| names(&e)),
        genre: info(doc, &["Genres", "Genre(s)", "Géneros"]).and_then(|e| names(&e)).map(|g| g.split(", ").map(str::to_string).collect()).unwrap_or_default(),
        status,
        description: Some(description).filter(|d| !d.is_empty()),
        ..Default::default()
    }
}

fn chapter_date(text: &str) -> i64 {
    // `yyyy` also swallows a two-digit year ("26" -> year 26), so a result before 1990 means the next pattern is the one
    ["dd MMM yyyy", "dd MMM yy", "MMM yyyy", "MMM yy"]
        .iter()
        .filter_map(|p| parse_date(text, p, "en"))
        .find(|&ms| ms > millis(1990, 1, 1, 0, 0, 0))
        .unwrap_or(0)
}

fn parse_chapters(doc: &Document) -> Vec<Chapter> {
    doc.select("ul.chapter-list li, ul.row-content-chapter li, .list-items li.item")
        .iter()
        .filter_map(|el| {
            let link = el.select_first("a[href]")?;
            let name = el.select_first(".chapter-name, .item a > span:first-child")?.text();
            let date = el.select_first(".ct-update, .chapter-time, .item a > span:nth-child(2)").map(|e| chapter_date(&e.text())).unwrap_or(0);
            Some(Chapter { url: path_of(&link.abs_url("href")), name, chapter_number: -1.0, scanlator: None, date_upload: date, memo: None })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chapter_dates_come_in_the_sites_three_shapes() {
        // "06 Oct 2026", the two-digit year "12 Oct 26", and a month with no day "Oct 26"
        assert_eq!(chapter_date("06 Oct 2026"), millis(2026, 10, 6, 0, 0, 0));
        assert_eq!(chapter_date("12 Oct 26"), millis(2026, 10, 12, 0, 0, 0));
        assert_eq!(chapter_date("Oct 26"), millis(2026, 10, 1, 0, 0, 0));
        assert_eq!(chapter_date("not a date"), 0);
    }

    #[test]
    fn a_title_keeps_its_path_without_the_host() {
        assert_eq!(path_of("https://site.test/comic/a-b/?x=1"), "/comic/a-b/?x=1");
    }
}
