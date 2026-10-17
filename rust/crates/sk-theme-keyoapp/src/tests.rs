//! Keyoapp against responses saved from a live site (Asmodeus Scans, trimmed).

use super::*;

struct Test;
impl Theme for Test {
    const NAME: &'static str = "Test";
    const BASE_URL: &'static str = "https://asmotoon.com";
}
type S = Site<Test>;

const SEARCH: &str = include_str!("../tests/fixtures/asmo-search.json");
const LATEST: &str = include_str!("../tests/fixtures/asmo-latest.html");
const DETAIL: &str = include_str!("../tests/fixtures/asmo-detail.html");
const CHAPTERS: &str = include_str!("../tests/fixtures/asmo-chapters.json");
const BROWSE: &str = include_str!("../tests/fixtures/asmo-browse.html");

fn mock(f: impl Fn(&str) -> Option<&'static str> + 'static) {
    set_mock_http(move |r| match f(&r.url) {
        Some(body) => Ok(HttpResponse { status: 200, body: body.into(), url: r.url.clone(), ..Default::default() }),
        None => Ok(HttpResponse { status: 404, url: r.url.clone(), ..Default::default() }),
    });
}

#[test]
fn popular_and_search_use_the_ajax_endpoint() {
    set_context("en", &[]);
    mock(|u| (u == "https://asmotoon.com/?_ajax=search&sort=popular&offset=20").then_some(SEARCH));
    let page = S::default().popular(2).unwrap();
    assert_eq!(page.mangas.len(), 2);
    assert_eq!(page.mangas[0].url, "/the-lazy-aristocrat");
    assert_eq!(page.mangas[0].title, "The Lazy Aristocrat");
    assert_eq!(page.mangas[0].thumbnail_url.as_deref(), Some("https://wsrv.nl/?url=cdn.meowing.org/uploads/RoIGEtuCQ8_"));
    assert!(page.has_next);

    // a select on its first option sends nothing; a tag is sent as tag[]
    mock(|u| {
        assert_eq!(u, "https://asmotoon.com/?_ajax=search&q=lazy&offset=0&status=ongoing&sort=popular&tag%5B%5D=action");
        Some(SEARCH)
    });
    let f: Filters = serde_json::from_str(r#"{"type":0,"status":1,"sort":2,"tags":{"action":true,"drama":false}}"#).unwrap();
    S::default().search(1, " lazy ", &f).unwrap();
}

#[test]
fn latest_reads_the_recent_chapter_cards() {
    set_context("en", &[]);
    mock(|u| (u == "https://asmotoon.com/?_ajax=recent-chapters&page=1").then_some(LATEST));
    let page = S::default().latest(1).unwrap();
    assert_eq!(page.mangas.len(), 2);
    assert_eq!(page.mangas[0].url, "/a-cool-british-beautiful-girl-apparently-wants-to-homestay-with-me");
    assert!(page.mangas[0].thumbnail_url.as_deref().unwrap().starts_with("https://wsrv.nl/"));
    assert!(page.has_next);
}

#[test]
fn details_and_chapters_in_one_update() {
    set_context("en", &[]);
    mock(|u| match u {
        "https://asmotoon.com/the-lazy-aristocrat" => Some(DETAIL),
        "https://asmotoon.com/?_chapters_html=1&series_slug=the-lazy-aristocrat" => Some(CHAPTERS),
        _ => None,
    });
    let manga = Manga { url: "/the-lazy-aristocrat".into(), ..Default::default() };
    let up = S::default().update(&manga, true, true).unwrap();
    let m = up.manga.unwrap();
    assert_eq!(m.title, "The Lazy Aristocrat");
    assert_eq!(m.status, 1);
    assert!(m.thumbnail_url.as_deref().unwrap().starts_with("https://"), "{:?}", m.thumbnail_url);
    assert!(!m.genre.is_empty() && m.description.as_deref().is_some_and(|d| !d.is_empty()), "{m:?}");

    // the locked chapter is hidden unless the setting is on
    let ch = up.chapters.unwrap();
    assert_eq!(ch.len(), 2);
    assert!(ch.iter().all(|c| !c.name.starts_with('🔒')));
    set_context("en", &[(SHOW_PAID_PREF, json!(true))]);
    let all = S::default().chapters(&manga).unwrap();
    assert_eq!(all.len(), 3);
    assert_eq!(all[0].name, "🔒 Chapter 16");
    assert_eq!(all[0].url, "/the-lazy-aristocrat-chapter-16");
}

#[test]
fn an_unknown_series_is_an_error() {
    set_context("en", &[]);
    mock(|_| Some("<html><body>nothing</body></html>"));
    let err = S::default().details(&Manga { url: "/nope".into(), ..Default::default() }).unwrap_err();
    assert_eq!(err, "Series not found");
}

#[test]
fn genres_are_listed_from_the_browse_page() {
    set_context("en", &[]);
    mock(|u| (u == "https://asmotoon.com/?browse=1").then_some(BROWSE));
    let data = S::default().filter_data().unwrap().unwrap();
    assert_eq!(data, json!(["academy", "action", "adaption"]));
    let f = S::default().filters(Some(&data));
    assert_eq!(f.len(), 4);
    assert!(matches!(&f[3], FilterDef::Group { items, .. } if items.len() == 3));
    mock(|_| Some("<p>no genres</p>"));
    assert!(S::default().filter_data().is_err());
}

#[test]
fn relative_and_absolute_dates() {
    let now = millis(2024, 3, 31, 10, 20, 45) + 123;
    let base = millis(2024, 3, 31, 10, 20, 0);
    assert_eq!(relative("2 hours ago", now), base - 2 * 3600 * 1000);
    assert_eq!(relative("a day ago", now), base - 86_400_000);
    assert_eq!(relative("1 month ago", now), millis(2024, 2, 29, 10, 20, 0));
    assert_eq!(relative("3 wks ago", now), base - 21 * 86_400_000);
    assert_eq!(relative("yesterday", now), 0);
    assert_eq!(parse_chapter_date::<Test>("Mar 5, 2024"), millis(2024, 3, 5, 0, 0, 0));
    assert_eq!(parse_chapter_date::<Test>("whenever"), 0);
}

