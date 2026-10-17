//! MangaFox (fanfox.net), written in Rust. Port of the Kotlin extension `en.mangafox`: HTML lists, one page for a
//! title's details and chapters, and the mobile site's "roll" reader for the pages (all of a chapter's images at once).

use sk_source_sdk::html::Document;
use sk_source_sdk::*;

const BASE: &str = "https://fanfox.net";
const MOBILE: &str = "https://m.fanfox.net";

const TYPES: [(&str, &str); 8] = [
    ("Any", "0"),
    ("Japanese Manga", "1"),
    ("Korean Manhwa", "2"),
    ("Chinese Manhua", "3"),
    ("European Manga", "4"),
    ("American Manga", "5"),
    ("HongKong Manga", "6"),
    ("Other Manga", "7"),
];
const COMPLETED: [(&str, &str); 3] = [("Either", "0"), ("Yes", "2"), ("No", "1")];
const TEXT_METHOD: [(&str, &str); 3] = [("contain", "cw"), ("begin", "bw"), ("end", "ew")];
const RATING_METHOD: [(&str, &str); 3] = [("is", "eq"), ("less than", "lt"), ("more than", "gt")];
const RATING: [(&str, &str); 7] = [
    ("any star", ""),
    ("no star", "0"),
    ("1 star", "1"),
    ("2 stars", "2"),
    ("3 stars", "3"),
    ("4 stars", "4"),
    ("5 stars", "5"),
];
const YEAR_METHOD: [(&str, &str); 3] = [("on", "eq"), ("before", "lt"), ("after", "gt")];

/// `(label, id)` of the site's genres.
const GENRES: [(&str, u32); 37] = [
    ("Action", 1),
    ("Adventure", 2),
    ("Comedy", 3),
    ("Drama", 4),
    ("Fantasy", 5),
    ("Martial Arts", 6),
    ("Shounen", 7),
    ("Horror", 8),
    ("Supernatural", 9),
    ("Harem", 10),
    ("Psychological", 11),
    ("Romance", 12),
    ("School Life", 13),
    ("Shoujo", 14),
    ("Mystery", 15),
    ("Sci-fi", 16),
    ("Seinen", 17),
    ("Tragedy", 18),
    ("Ecchi", 19),
    ("Sports", 20),
    ("Slice of Life", 21),
    ("Mature", 22),
    ("Shoujo Ai", 23),
    ("Webtoons", 24),
    ("Doujinshi", 25),
    ("One Shot", 26),
    ("Smut", 27),
    ("Yaoi", 28),
    ("Josei", 29),
    ("Historical", 30),
    ("Shounen Ai", 31),
    ("Gender Bender", 32),
    ("Adult", 33),
    ("Yuri", 34),
    ("Mecha", 35),
    ("Lolicon", 36),
    ("Shotacon", 37),
];

#[derive(Default)]
pub struct MangaFox;

impl Source for MangaFox {
    fn info(&self) -> Info {
        Info {
            name: "MangaFox".into(),
            lang: "en".into(),
            base_url: BASE.into(),
            supports_latest: true,
            rate_limit: Some(RateLimit { permits: 1, period_ms: 1000 }),
            // The mobile site lists every page of a chapter at once with `readway=2`; `isAdult` skips the age gate.
            cookies: vec![CookieDef::new("m.fanfox.net", "readway", "2"), CookieDef::new("fanfox.net", "isAdult", "1")],
            ..Default::default()
        }
    }

    fn popular(&self, page: u32) -> Result<MangaPage> {
        directory(page, "")
    }

    fn latest(&self, page: u32) -> Result<MangaPage> {
        directory(page, "?latest")
    }

    fn search(&self, page: u32, query: &str, filters: &Filters) -> Result<MangaPage> {
        let mut params: Vec<(String, String)> = vec![("title".into(), query.into())];
        let mut push = |k: &str, v: &str| params.push((k.into(), v.into()));
        push("name", filters.text("name"));
        push("type", filters.select_value("type", &TYPES));
        push("st", filters.select_value("st", &COMPLETED));
        for key in ["author", "artist"] {
            let g = filters.group_state(key);
            push(&format!("{key}_method"), g.select_value("method", &TEXT_METHOD));
            push(key, g.text("text"));
        }
        let rating = filters.group_state("rating");
        push("rating_method", rating.select_value("method", &RATING_METHOD));
        push("rating", rating.select_value("value", &RATING));
        let year = filters.group_state("released");
        push("released_method", year.select_value("method", &YEAR_METHOD));
        push("released", year.text("text"));
        let ids = |keys: Vec<&str>| keys.iter().filter_map(|k| k.parse::<u32>().ok()).map(|i| i.to_string()).collect::<Vec<_>>().join(",");
        let genres = ids(filters.included("genres"));
        let excluded = ids(filters.excluded("genres"));
        push("genres", &genres);
        push("nogenres", &excluded);
        push("sort", "");
        push("stype", "1");
        let query_string = params.iter().map(|(k, v)| format!("{k}={}", url_encode(v))).collect::<Vec<_>>().join("&");
        // The site pages its search with `page=`.
        let url = format!("{BASE}/search?{query_string}{}", if page > 1 { format!("&page={page}") } else { String::new() });
        let doc = get(&url).send()?.html();
        Ok(parse_list(&doc, "ul.manga-list-4-list li"))
    }

    fn filters(&self, _data: Option<&Value>) -> Vec<FilterDef> {
        let text_with_method = |key: &str, name: &str, method_options: &[(&str, &str)]| FilterDef::Group {
            key: key.into(),
            name: name.into(),
            items: vec![FilterDef::select("method", "Method", method_options, 0), FilterDef::text("text", name)],
        };
        let genres: Vec<(String, String)> = GENRES.iter().map(|(l, id)| (l.to_string(), id.to_string())).collect();
        vec![
            FilterDef::text("name", "Name"),
            FilterDef::select("type", "Type", &TYPES, 0),
            FilterDef::select("st", "Completed Series", &COMPLETED, 0),
            text_with_method("author", "Author", &TEXT_METHOD),
            text_with_method("artist", "Artist", &TEXT_METHOD),
            FilterDef::Group {
                key: "rating".into(),
                name: "Rating".into(),
                items: vec![FilterDef::select("method", "Method", &RATING_METHOD, 0), FilterDef::select("value", "Rating", &RATING, 0)],
            },
            text_with_method("released", "Release year", &YEAR_METHOD),
            FilterDef::tristates("genres", "Genre", &genres),
        ]
    }

    fn update(&self, manga: &Manga, _fetch_details: bool, _fetch_chapters: bool) -> Result<Update> {
        // One page has both; the site gives no cheaper way to ask for only one.
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
        let path = chapter.url.replace("/manga/", "/roll_manga/");
        let doc = get(&format!("{MOBILE}{path}")).header("Referer", &format!("{MOBILE}/")).send()?.html();
        Ok(doc
            .select("#viewer img")
            .iter()
            .enumerate()
            .map(|(i, img)| Page::new(i as u32, img.abs_url("data-original")))
            .collect())
    }
}

fn directory(page: u32, suffix: &str) -> Result<MangaPage> {
    let page_str = if page != 1 { format!("{page}.html") } else { String::new() };
    let doc = get(&format!("{BASE}/directory/{page_str}{suffix}")).send()?.html();
    Ok(parse_list(&doc, "ul.manga-list-1-list li"))
}

fn parse_list(doc: &Document, selector: &str) -> MangaPage {
    let mangas = doc
        .select(selector)
        .iter()
        .filter_map(|li| {
            let a = li.select_first("a")?;
            Some(Manga {
                url: path_of(&a.abs_url("href")),
                title: a.attr("title"),
                thumbnail_url: a.select_first("img").map(|i| i.abs_url("src")).filter(|s| !s.is_empty()),
                ..Default::default()
            })
        })
        .collect();
    MangaPage { mangas, has_next: doc.select_first(".pager-list-left a.active + a + a").is_some() }
}

fn parse_details(doc: &Document, previous: &Manga) -> Manga {
    let mut manga = Manga { url: previous.url.clone(), title: previous.title.clone(), initialized: true, ..Default::default() };
    if let Some(info) = doc.select_first(".detail-info-right") {
        if let Some(t) = info.select_first(".detail-info-right-title-font") {
            manga.title = t.text();
        }
        manga.author = Some(info.select(".detail-info-right-say a").each_text().join(", "));
        manga.genre = info.select(".detail-info-right-tag-list a").each_text();
        manga.description = info.select_first("p.fullcontent").map(|p| p.text());
        manga.status = parse_status(info.select_first(".detail-info-right-title-tip").map(|t| t.text()).as_deref());
        manga.thumbnail_url = doc.select_first(".detail-info-cover-img").map(|i| i.abs_url("src")).filter(|s| !s.is_empty());
    }
    manga
}

fn parse_status(s: Option<&str>) -> u8 {
    match s {
        Some(s) if s.contains("Ongoing") => 1,
        Some(s) if s.contains("Completed") => 2,
        _ => 0,
    }
}

fn parse_chapters(doc: &Document) -> Vec<Chapter> {
    doc.select("ul.detail-main-list li a")
        .iter()
        .filter_map(|a| {
            let lines = a.select(".detail-main-list-main p");
            Some(Chapter {
                url: path_of(&a.abs_url("href")),
                name: lines.first()?.text(),
                date_upload: lines.last().map(|p| parse_chapter_date(&p.text(), now_ms())).unwrap_or(0),
                ..Default::default()
            })
        })
        .collect()
}

/// "Today", "3 hours ago" and "Yesterday" are dates at midnight; otherwise "Mar 15,2021".
fn parse_chapter_date(date: &str, now: i64) -> i64 {
    let midnight = now - now.rem_euclid(86_400_000);
    if date.contains("Today") || date.contains(" ago") {
        midnight
    } else if date.contains("Yesterday") {
        midnight - 86_400_000
    } else {
        parse_date(date.trim(), "MMM d,yyyy", "en").unwrap_or(0)
    }
}

export_source!(MangaFox);

#[cfg(test)]
mod tests {
    use super::*;

    const DIR: &str = include_str!("../tests/fixtures/dir.html");
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
        assert_eq!(sk_source::extension_id("MangaFox", "en", 1), 6_484_561_431_658_238_800);
    }

    #[test]
    fn the_directory_lists_titles_and_knows_when_there_is_a_next_page() {
        mock(|u| (u == "https://fanfox.net/directory/2.html?latest").then_some(DIR));
        let page = MangaFox.latest(2).unwrap();
        assert_eq!(page.mangas.len(), 2);
        assert!(page.mangas[0].url.starts_with("/manga/a_story_about"), "{}", page.mangas[0].url);
        assert!(page.mangas[0].title.starts_with("A Story About Treating"));
        assert!(page.mangas[0].thumbnail_url.as_deref().unwrap().starts_with("https://fmcdn.mfcdn.net/"));
        assert!(page.has_next, "the active page is followed by two more");
    }

    #[test]
    fn search_sends_every_field_the_site_expects() {
        mock(|u| {
            assert_eq!(
                u,
                "https://fanfox.net/search?title=naruto&name=&type=2&st=2&author_method=bw&author=Kishi&artist_method=cw&artist=&rating_method=gt&rating=4&released_method=eq&released=&genres=1%2C3&nogenres=8&sort=&stype=1"
            );
            Some(SEARCH)
        });
        let f: Filters = serde_json::from_str(
            r#"{"type":2,"st":1,"author":{"method":1,"text":"Kishi"},"rating":{"method":2,"value":5},"genres":{"1":1,"3":1,"8":2,"2":0}}"#,
        )
        .unwrap();
        assert_eq!(MangaFox.search(1, "naruto", &f).unwrap().mangas.len(), 2);
    }

    #[test]
    fn a_title_page_gives_details_and_chapters() {
        mock(|u| (u == "https://fanfox.net/manga/x/").then_some(DETAIL));
        let manga = Manga { url: "/manga/x/".into(), title: "old".into(), ..Default::default() };
        let up = MangaFox.update(&manga, true, true).unwrap();
        let m = up.manga.unwrap();
        assert!(m.title.starts_with("A Story About Treating"));
        assert_eq!((m.status, m.author.as_deref()), (1, Some("MATSUMOTO Kengo")));
        assert_eq!(m.genre, ["Action", "Comedy", "Fantasy", "Shounen", "Romance"]);
        assert!(m.description.as_deref().unwrap().starts_with("Female knight Leo"));
        let ch = up.chapters.unwrap();
        assert_eq!(ch.len(), 3);
        assert_eq!(ch[0].name, "Vol.01 Ch.131");
        assert!(ch[0].url.ends_with("/v01/c131/1.html"));
        assert_eq!(ch[0].date_upload, millis(2021, 3, 15, 0, 0, 0));
    }

    #[test]
    fn pages_come_from_the_mobile_roll_reader() {
        mock(|u| {
            assert_eq!(u, "https://m.fanfox.net/roll_manga/x/v01/c131/1.html");
            Some(CHAPTER)
        });
        let pages = MangaFox.pages(&Chapter { url: "/manga/x/v01/c131/1.html".into(), ..Default::default() }).unwrap();
        assert_eq!(pages.len(), 3);
        assert!(pages[0].image_url.starts_with("https://zjcdn.mangafox.me/store/manga/27418/01-131.0/compressed/f000.jpg?token="), "{}", pages[0].image_url);
    }

    #[test]
    fn dates_relative_to_today_are_midnight() {
        let now = millis(2024, 5, 10, 15, 30, 0);
        assert_eq!(parse_chapter_date("3 hours ago", now), millis(2024, 5, 10, 0, 0, 0));
        assert_eq!(parse_chapter_date("Today 10:00", now), millis(2024, 5, 10, 0, 0, 0));
        assert_eq!(parse_chapter_date("Yesterday", now), millis(2024, 5, 9, 0, 0, 0));
        assert_eq!(parse_chapter_date("Mar 15,2021 ", now), millis(2021, 3, 15, 0, 0, 0));
    }
}
