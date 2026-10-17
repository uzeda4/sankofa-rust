//! Manga Demon (demonicscans.org), written in Rust. Port of the Kotlin extension `en.mangademon` (version id 2):
//! HTML lists with `?list=N` paging, a search page without paging, and a title page that holds the chapter list.

use sk_source_sdk::html::Document;
use sk_source_sdk::*;

const BASE: &str = "https://demonicscans.org";

const STATUS: [(&str, &str); 3] = [("All", "all"), ("Ongoing", "ongoing"), ("Completed", "completed")];
const SORT: [(&str, &str); 2] = [("Top Views", "VIEWS DESC"), ("A To Z", "NAME ASC")];

/// `(label, id)` of the genres the advanced search offers.
const GENRES: [(&str, &str); 33] = [
    ("Action", "1"),
    ("Adventure", "2"),
    ("Comedy", "3"),
    ("Cooking", "34"),
    ("Doujinshi", "25"),
    ("Drama", "4"),
    ("Ecchi", "19"),
    ("Fantasy", "5"),
    ("Gender Bender", "30"),
    ("Harem", "10"),
    ("Historical", "28"),
    ("Horror", "8"),
    ("Isekai", "33"),
    ("Josei", "31"),
    ("Martial Arts", "6"),
    ("Mature", "22"),
    ("Mecha", "32"),
    ("Mystery", "15"),
    ("One Shot", "26"),
    ("Psychological", "11"),
    ("Romance", "12"),
    ("School Life", "13"),
    ("Sci-fi", "16"),
    ("Seinen", "17"),
    ("Shoujo", "14"),
    ("Shoujo Ai", "23"),
    ("Shounen", "7"),
    ("Shounen Ai", "29"),
    ("Slice of Life", "21"),
    ("Smut", "27"),
    ("Sports", "20"),
    ("Supernatural", "9"),
    ("Tragedy", "18"),
];

#[derive(Default)]
pub struct MangaDemon;

impl Source for MangaDemon {
    fn info(&self) -> Info {
        Info {
            name: "Manga Demon".into(),
            lang: "en".into(),
            base_url: BASE.into(),
            version_id: 2,
            supports_latest: true,
            rate_limit: Some(RateLimit { permits: 2, period_ms: 1000 }),
            ..Default::default()
        }
    }

    fn popular(&self, page: u32) -> Result<MangaPage> {
        advanced(&format!("{BASE}/advanced.php?list={page}&status=all&orderby=VIEWS%20DESC"))
    }

    fn latest(&self, page: u32) -> Result<MangaPage> {
        let doc = get(&format!("{BASE}/lastupdates.php?list={page}")).send()?.html();
        let mangas = doc
            .select("div#updates-container > div.updates-element:not(:has(.toffee-badge))")
            .iter()
            .filter_map(|el| {
                let info = el.select_first("div.updates-element-info")?;
                let link = info.select_first("a")?;
                Some(Manga {
                    url: decoded(&link.attr("href")),
                    title: link.own_text(),
                    thumbnail_url: Some(el.select_first("div.thumb img")?.abs_url("src")),
                    ..Default::default()
                })
            })
            .collect();
        Ok(MangaPage { mangas, has_next: has_next(&doc) })
    }

    fn search(&self, page: u32, query: &str, filters: &Filters) -> Result<MangaPage> {
        if query.is_empty() {
            let mut url = format!("{BASE}/advanced.php?list={page}");
            for genre in filters.included("genres") {
                url.push_str(&format!("&genre[]={}", url_encode(genre)));
            }
            url.push_str(&format!("&status={}", url_encode(filters.select_value("status", &STATUS))));
            url.push_str(&format!("&orderby={}", url_encode(filters.select_value("sort", &SORT))));
            return advanced(&url);
        }
        let doc = get(&format!("{BASE}/search.php?manga={}", url_encode(query))).send()?.html();
        let mangas = doc
            .select("body > a[href]")
            .iter()
            .filter_map(|a| {
                Some(Manga {
                    url: decoded(&a.attr("href")),
                    title: a.select_first("div.seach-right > div")?.own_text(),
                    thumbnail_url: Some(a.select_first("img")?.abs_url("src")),
                    ..Default::default()
                })
            })
            .collect();
        Ok(MangaPage { mangas, has_next: false })
    }

    fn filters(&self, _data: Option<&Value>) -> Vec<FilterDef> {
        let genres: Vec<(String, String)> = GENRES.iter().map(|(l, v)| (l.to_string(), v.to_string())).collect();
        vec![
            FilterDef::header("Ignored when using text search"),
            FilterDef::Separator,
            FilterDef::select("sort", "Sort", &SORT, 0),
            FilterDef::select("status", "Status", &STATUS, 0),
            FilterDef::checkboxes("genres", "Genre", &genres),
        ]
    }

    fn update(&self, manga: &Manga, _fetch_details: bool, _fetch_chapters: bool) -> Result<Update> {
        let doc = get(&absolute(BASE, &manga.url)).send()?.html();
        Ok(Update { manga: Some(parse_details(&doc, manga)), chapters: Some(parse_chapters(&doc)) })
    }

    fn details(&self, manga: &Manga) -> Result<Manga> {
        self.update(manga, true, false)?.manga.ok_or_else(|| "no details".into())
    }

    fn chapters(&self, manga: &Manga) -> Result<Vec<Chapter>> {
        self.update(manga, false, true)?.chapters.ok_or_else(|| "no chapters".into())
    }

    fn pages(&self, chapter: &Chapter) -> Result<Vec<Page>> {
        let doc = get(&absolute(BASE, &chapter.url)).send()?.html();
        Ok(doc
            .select("div > img.imgholder")
            .iter()
            .enumerate()
            .map(|(i, img)| Page::new(i as u32, img.abs_url("src")))
            .collect())
    }
}

/// What `URLEncoder.encode` followed by `URI.getPath` made of an address in the Kotlin extension: the address as written.
fn decoded(href: &str) -> String {
    href.to_string()
}

fn has_next(doc: &Document) -> bool {
    doc.select_first("div.pagination > ul > a > li:contains(Next)").is_some()
}

fn advanced(url: &str) -> Result<MangaPage> {
    let doc = get(url).send()?.html();
    let mangas = doc
        .select("div#advanced-content > div.advanced-element")
        .iter()
        .filter_map(|el| {
            Some(Manga {
                url: decoded(&el.select_first("a")?.attr("href")),
                title: el.select_first("h1")?.own_text(),
                thumbnail_url: el.select_first("img").map(|i| i.abs_url("src")),
                ..Default::default()
            })
        })
        .collect();
    Ok(MangaPage { mangas, has_next: has_next(&doc) })
}

fn parse_details(doc: &Document, previous: &Manga) -> Manga {
    let mut manga = Manga { url: previous.url.clone(), title: previous.title.clone(), initialized: true, ..Default::default() };
    let Some(info) = doc.select_first("div#manga-info-container") else { return manga };
    if let Some(t) = info.select_first("h1.big-fat-titles") {
        manga.title = t.own_text();
    }
    manga.thumbnail_url = info.select_first("div#manga-page img").map(|i| i.abs_url("src"));
    manga.genre = info.select("div.genres-list > li").each_text();
    manga.description = info.select_first("div#manga-info-rightColumn > div > div.white-font").map(|d| d.text());
    let stat = |label: &str| info.select(&format!("div#manga-info-stats > div:has(> li:eq(0):contains({label})) > li:eq(1)")).text();
    let author = stat("Author");
    manga.author = Some(author).filter(|a| !a.to_lowercase().contains("updating"));
    let status = stat("Status").to_lowercase();
    manga.status = if status.contains("ongoing") {
        1
    } else if status.contains("completed") {
        2
    } else {
        0
    };
    manga
}

fn parse_chapters(doc: &Document) -> Vec<Chapter> {
    doc.select("div#chapters-list a.chplinks")
        .iter()
        .map(|a| Chapter {
            url: decoded(&a.attr("href")),
            name: a.own_text(),
            date_upload: a.select_first("span").and_then(|s| parse_date(&s.own_text(), "yyyy-MM-dd", "en")).unwrap_or(0),
            ..Default::default()
        })
        .collect()
}

export_source!(MangaDemon);

#[cfg(test)]
mod tests {
    use super::*;

    const LIST: &str = include_str!("../tests/fixtures/list.html");
    const LATEST: &str = include_str!("../tests/fixtures/latest.html");
    const SEARCH: &str = include_str!("../tests/fixtures/search.html");
    const DETAIL: &str = include_str!("../tests/fixtures/detail.html");
    const CHAPTER: &str = include_str!("../tests/fixtures/chapter.html");

    fn mock(f: impl Fn(&str) -> Option<&'static str> + 'static) {
        set_mock_http(move |r| match f(&r.url) {
            Some(body) => Ok(HttpResponse { status: 200, body: body.into(), url: r.url.clone(), ..Default::default() }),
            None => Ok(HttpResponse { status: 404, url: r.url.clone(), ..Default::default() }),
        });
    }

    #[test]
    fn the_id_is_the_one_the_extension_had_on_the_device() {
        assert_eq!(sk_source::extension_id("Manga Demon", "en", 2), 2_900_023_289_777_642_714);
    }

    #[test]
    fn popular_and_filtered_lists_share_the_advanced_page() {
        mock(|u| (u == "https://demonicscans.org/advanced.php?list=1&status=all&orderby=VIEWS%20DESC").then_some(LIST));
        let page = MangaDemon.popular(1).unwrap();
        assert_eq!(page.mangas.len(), 2);
        assert_eq!(page.mangas[0].url, "/manga/Martial-Peak");
        assert_eq!(page.mangas[0].title, "Martial Peak");
        assert_eq!(page.mangas[0].thumbnail_url.as_deref(), Some("https://readermc.org/images/thumbnails/Martial%20Peak.webp"));
        assert!(page.has_next);

        mock(|u| {
            assert_eq!(u, "https://demonicscans.org/advanced.php?list=2&genre[]=1&genre[]=6&status=ongoing&orderby=NAME%20ASC");
            Some(LIST)
        });
        let f: Filters = serde_json::from_str(r#"{"sort":1,"status":1,"genres":{"1":true,"6":true,"2":false}}"#).unwrap();
        MangaDemon.search(2, "", &f).unwrap();
    }

    #[test]
    fn latest_skips_paid_entries_and_search_has_no_paging() {
        mock(|u| (u == "https://demonicscans.org/lastupdates.php?list=3").then_some(LATEST));
        let page = MangaDemon.latest(3).unwrap();
        assert!(!page.mangas.is_empty());
        assert_eq!(page.mangas[0].url, "/manga/Global-Martial-Arts");
        assert_eq!(page.mangas[0].title, "Global Martial Arts");

        mock(|u| (u == "https://demonicscans.org/search.php?manga=solo%20lev").then_some(SEARCH));
        let found = MangaDemon.search(1, "solo lev", &Filters::default()).unwrap();
        assert_eq!(found.mangas[0].title, "Solo Leveling");
        assert_eq!(found.mangas[0].url, "/manga/Solo-Leveling");
        assert!(!found.has_next);
    }

    #[test]
    fn the_title_page_has_details_and_chapters() {
        mock(|u| (u == "https://demonicscans.org/manga/Martial-Peak").then_some(DETAIL));
        let manga = Manga { url: "/manga/Martial-Peak".into(), title: "x".into(), ..Default::default() };
        let up = MangaDemon.update(&manga, true, true).unwrap();
        let m = up.manga.unwrap();
        assert_eq!(m.title, "Martial Peak");
        assert_eq!(m.genre, ["Historical", "Martial Arts", "Fantasy", "Adventure", "Action"]);
        assert_eq!((m.status, m.author.as_deref()), (1, Some("Momo (II),Pikapi")));
        assert!(m.description.as_deref().unwrap().starts_with("The journey to the martial peak"));
        assert!(m.thumbnail_url.as_deref().unwrap().contains("Martial%20Peak.webp"));
        let ch = up.chapters.unwrap();
        assert_eq!(ch.len(), 3);
        assert_eq!(ch[0].name, "Chapter 3862");
        assert_eq!(ch[0].url, "/chaptered.php?manga=1&chapter=3862");
        assert_eq!(ch[0].date_upload, millis(2025, 12, 22, 0, 0, 0));
    }

    #[test]
    fn pages_are_the_images_with_the_holder_class() {
        mock(|u| (u == "https://demonicscans.org/chaptered.php?manga=1&chapter=3862").then_some(CHAPTER));
        let pages = MangaDemon.pages(&Chapter { url: "/chaptered.php?manga=1&chapter=3862".into(), ..Default::default() }).unwrap();
        assert_eq!(pages.len(), 3);
        assert_eq!(pages[0].image_url, "https://demonicscans.org/img/free_ads.jpg");
    }
}
