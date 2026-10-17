//! MangaThemesia against pages saved from a live site (Arena Scans, trimmed to what the selectors read).

use super::*;

struct Test;
impl Theme for Test {
    const NAME: &'static str = "Arena Scans";
    const BASE_URL: &'static str = "https://arenascan.com";
}
type S = Site<Test>;

const LIST: &str = include_str!("../tests/fixtures/arena-list.html");
const DETAIL: &str = include_str!("../tests/fixtures/arena-detail.html");
const CHAPTER: &str = include_str!("../tests/fixtures/arena-chapter.html");

fn mock(f: impl Fn(&str) -> Option<&'static str> + 'static) {
    set_mock_http(move |r| match f(&r.url) {
        Some(body) => Ok(HttpResponse { status: 200, body: body.into(), url: r.url.clone(), ..Default::default() }),
        None => Ok(HttpResponse { status: 404, url: r.url.clone(), ..Default::default() }),
    });
}

#[test]
fn popular_is_the_search_page_ordered_by_popularity() {
    set_context("en", &[]);
    mock(|u| (u == "https://arenascan.com/manga/?page=1&order=popular").then_some(LIST));
    let page = S::default().popular(1).unwrap();
    assert_eq!(page.mangas.len(), 3);
    assert_eq!(page.mangas[0].title, "How to Get My Husband on My Side");
    assert_eq!(page.mangas[0].url, "/manga/how-to-get-my-husband-on-my-side/");
    assert!(page.mangas[0].thumbnail_url.as_deref().unwrap().starts_with("https://arenascan.com/wp-content/uploads/"));
    assert!(page.has_next, "the Next link of the pager");
}

#[test]
fn search_sends_the_query_then_the_filters_in_the_sites_order() {
    set_context("en", &[]);
    mock(|u| {
        assert_eq!(
            u,
            "https://arenascan.com/manga/?title=my%20husband&page=2&author=Kim&yearx=2024&status=ongoing&type=Manhwa&order=update&genre[]=action&genre[]=-drama"
        );
        Some(LIST)
    });
    let filters: Filters = serde_json::from_str(
        r#"{"author":"Kim","year":"2024","status":1,"type":2,"order":3,"genres":{"action":1,"drama":2,"comedy":0}}"#,
    )
    .unwrap();
    assert_eq!(S::default().search(2, "my husband", &filters).unwrap().mangas.len(), 3);
    // with no filters the site still gets every select, empty
    mock(|u| {
        assert_eq!(u, "https://arenascan.com/manga/?page=1&status=&type=&order=");
        Some(LIST)
    });
    S::default().search(1, "", &Filters::default()).unwrap();
}

#[test]
fn details_read_the_real_title_page() {
    set_context("en", &[]);
    mock(|u| (u == "https://arenascan.com/manga/how-to-get-my-husband-on-my-side/").then_some(DETAIL));
    let manga = Manga { url: "/manga/how-to-get-my-husband-on-my-side/".into(), ..Default::default() };
    let up = S::default().update(&manga, true, true).unwrap();
    let m = up.manga.unwrap();
    assert_eq!(m.title, "How to Get My Husband on My Side");
    assert!(m.thumbnail_url.as_deref().unwrap().ends_with(".jpg"));
    assert!(m.description.as_deref().is_some_and(|d| d.len() > 40), "{:?}", m.description);
    assert!(!m.genre.is_empty(), "genres come from .mgen a");
    assert!(m.genre.iter().all(|g| g.chars().next().unwrap().is_uppercase()), "{:?}", m.genre);
    // this page has an empty status and "n/a" authors: unknown status, no author
    assert_eq!((m.status, m.author, m.artist), (0, None, None));

    let ch = up.chapters.unwrap();
    assert_eq!(ch.len(), 4);
    assert_eq!(ch[0].name, "Chapter 137");
    assert_eq!(ch[0].url, "/how-to-get-my-husband-on-my-side-137/");
    assert_eq!(ch[0].date_upload, parse_date("February 14, 2026", "MMMM d, yyyy", "en").unwrap());
    assert!(ch[0].date_upload > 0);
}

#[test]
fn pages_come_from_the_noscript_images_or_the_ts_reader_script() {
    set_context("en", &[]);
    mock(|u| (u == "https://arenascan.com/how-to-get-my-husband-on-my-side-137/").then_some(CHAPTER));
    let pages = S::default().pages(&Chapter { url: "/how-to-get-my-husband-on-my-side-137/".into(), ..Default::default() }).unwrap();
    assert!(pages.len() >= 5, "{}", pages.len());
    assert_eq!(pages[0].image_url, "https://cdn.arenascan.com/arena-bucket/173838/137/1.jpg");
    assert_eq!(pages[1].index, 1);

    // without images in the HTML the page's script lists them, relative ones against the site
    mock(|_| Some(r#"<script>ts_reader.run({"sources":[{"source":"Server 1","images":["https://a.test/1.jpg","/2.jpg"]}]});</script>"#));
    let pages = S::default().pages(&Chapter { url: "/c/".into(), ..Default::default() }).unwrap();
    assert_eq!(pages.iter().map(|p| p.image_url.as_str()).collect::<Vec<_>>(), ["https://a.test/1.jpg", "https://arenascan.com/2.jpg"]);

    // and the base64 form of that script
    let b64 = base64::engine::general_purpose::STANDARD.encode(r#"ts_reader.run({"images":["https://b.test/9.jpg"]})"#);
    let html = format!(r#"<script src="data:text/javascript;base64,dHNfcmVhZGVyLnJ1bih7{}"></script>"#, &b64[b64.find("dHNfcmVhZGVyLnJ1bih7").map(|i| i + 20).unwrap_or(0)..]);
    mock_owned(html);
    let pages = S::default().pages(&Chapter { url: "/c/".into(), ..Default::default() }).unwrap();
    assert_eq!(pages.len(), 1);
    assert_eq!(pages[0].image_url, "https://b.test/9.jpg");
}

fn mock_owned(body: String) {
    set_mock_http(move |r| Ok(HttpResponse { status: 200, body: body.clone(), url: r.url.clone(), ..Default::default() }));
}

#[test]
fn filters_list_genres_once_the_site_data_is_known() {
    let none = S::default().filters(None);
    assert_eq!(none.len(), 6);
    let data = serde_json::json!([{"name": "Action", "value": "action"}, {"name": "Drama", "value": "drama"}]);
    let with = S::default().filters(Some(&data));
    assert_eq!(with.len(), 8);
    assert!(matches!(&with[7], FilterDef::Group { items, .. } if items.len() == 2));

    mock_owned(r#"<ul class="genrez"><li><input type="checkbox" value="action"><label>Action</label></li><li><input type="checkbox" value="drama"><label>Drama</label></li></ul>"#.into());
    let d = S::default().filter_data().unwrap().unwrap();
    assert_eq!(d[1]["value"], "drama");
}

#[test]
fn status_and_helpers() {
    assert_eq!(parse_status("Ongoing"), 1);
    assert_eq!(parse_status("Status Completed"), 2);
    assert_eq!(parse_status("Hiatus"), 6);
    assert_eq!(parse_status("Dropped"), 5);
    assert_eq!(parse_status("???"), 0);
    assert_eq!(capitalize("SLICE OF life"), "Slice of life");
    assert_eq!(remove_placeholder("-".into()), None);
    assert_eq!(remove_placeholder(" Jane ".into()).as_deref(), Some(" Jane "));
}
