//! Asura Scans against API answers and Astro pages saved from the live site (trimmed to what the source reads).

use super::*;

const LIST: &str = include_str!("../tests/fixtures/list.json");
const DETAIL: &str = include_str!("../tests/fixtures/detail.html");
const CHAPTER: &str = include_str!("../tests/fixtures/chapter.html");
const SCRAMBLED: &str = include_str!("../tests/fixtures/chapter-scrambled.html");
const BROWSE: &str = include_str!("../tests/fixtures/browse.html");
const CREATORS: &str = include_str!("../tests/fixtures/creators.json");

fn mock(f: impl Fn(&str) -> Option<&'static str> + 'static) {
    set_mock_http(move |r| match f(&r.url) {
        Some(body) => Ok(HttpResponse { status: 200, body: body.into(), url: r.url.clone(), ..Default::default() }),
        None => Ok(HttpResponse { status: 404, url: r.url.clone(), ..Default::default() }),
    });
}

#[test]
fn the_id_is_the_one_the_extension_had_on_the_device() {
    assert_eq!(sk_source::extension_id("Asura Scans", "en", 1), 6_247_824_327_199_706_550);
}

#[test]
fn lists_come_from_the_api_with_the_old_address_form() {
    set_context("en", &[]);
    mock(|u| (u == "https://api.asurascans.com/api/series?offset=20&limit=20&sort=popular&order=desc").then_some(LIST));
    let page = AsuraScans.popular(2).unwrap();
    assert_eq!(page.mangas.len(), 2);
    assert_eq!(page.mangas[0].url, "/series/overgeared");
    assert_eq!(page.mangas[0].title, "Overgeared");
    assert_eq!(page.mangas[0].memo.as_ref().unwrap()["slug"], "overgeared-1bb35e56");
    assert!(page.mangas[0].thumbnail_url.as_deref().unwrap().ends_with(".webp"));
    assert!(page.has_next);
}

#[test]
fn search_sends_sort_status_genres_creator_and_minimum_chapters() {
    set_context("en", &[]);
    set_filter_data(Some(json!({
        "genres": [{"name": "Action", "slug": "action"}],
        "artists": ["Ann", "Bo"],
        "authors": ["Cy"]
    })));
    mock(|u| {
        assert_eq!(
            u,
            "https://api.asurascans.com/api/series?offset=0&limit=20&search=lazy%20one&sort=rating&order=asc&status=ongoing&type=manhwa&genres=action%2Cdrama&author=Cy&min_chapters=30"
        );
        Some(LIST)
    });
    let f: Filters = serde_json::from_str(
        r#"{"sort":{"index":2,"ascending":true},"status":1,"type":1,"genres":{"action":true,"drama":true,"comedy":false},"creator":3,"min_chapters":"30"}"#,
    )
    .unwrap();
    AsuraScans.search(1, "lazy one", &f).unwrap();
    set_filter_data(None);
}

#[test]
fn a_title_page_gives_details_and_chapters_with_the_site_slug_in_the_memo() {
    set_context("en", &[]);
    set_filter_data(None);
    mock(|u| (u == "https://asurascans.com/comics/overgeared-1bb35e56").then_some(DETAIL));
    let stub = Manga { url: "/series/overgeared".into(), memo: Some(json!({ "slug": "overgeared-1bb35e56" })), ..Default::default() };

    let up = AsuraScans.update(&stub, true, true).unwrap();
    let m = up.manga.unwrap();
    assert_eq!(m.title, "Overgeared");
    assert_eq!(m.url, "/series/overgeared");
    assert_eq!(m.memo.as_ref().unwrap()["slug"], "overgeared-1bb35e56");
    assert_eq!(m.status, 1);
    assert!(m.genre.first().is_some_and(|g| g == "Manhwa"), "the type leads the genres: {:?}", m.genre);
    let d = m.description.unwrap();
    // the counts move with the site; the shape of the first line and the description after it do not
    assert!(d.starts_with("Rank: #1 • Rating: 9.19 • Bookmarks: 60.") && d.contains("K\n\nShin Youngwoo"), "{d}");
    assert!(d.contains("Alternative Titles:\n- "), "{d}");

    // premium chapters are hidden by default
    let ch = up.chapters.unwrap();
    assert_eq!(ch.len(), 2, "three chapters, the second is premium");
    assert!(ch[0].url.starts_with("/series/overgeared/chapter/"));
    assert_eq!(ch[0].memo.as_ref().unwrap()["mangaSlug"], "overgeared-1bb35e56");

    set_context("en", &[(HIDE_PREMIUM, json!(false))]);
    let all = AsuraScans.chapters(&stub).unwrap();
    assert_eq!(all.len(), 3);
    assert!(all[1].name.starts_with("🔒 Chapter "), "{}", all[1].name);
}

#[test]
fn a_stale_suffix_falls_back_to_the_address_without_it() {
    set_context("en", &[]);
    mock(|u| (u == "https://asurascans.com/comics/overgeared").then_some(DETAIL));
    let stub = Manga { url: "/series/overgeared".into(), memo: Some(json!({ "slug": "overgeared-deadbeef" })), ..Default::default() };
    assert_eq!(AsuraScans.details(&stub).unwrap().title, "Overgeared");
}

#[test]
fn addresses_for_the_browser_use_the_current_slug() {
    let m = Manga { url: "/series/overgeared".into(), memo: Some(json!({ "slug": "overgeared-1bb35e56" })), ..Default::default() };
    assert_eq!(AsuraScans.manga_url(&m).as_deref(), Some("https://asurascans.com/comics/overgeared-1bb35e56"));
    let no_memo = Manga { url: "/series/overgeared".into(), ..Default::default() };
    assert_eq!(AsuraScans.manga_url(&no_memo).as_deref(), Some("https://asurascans.com/comics/overgeared"));
    let old = Manga { url: "/manga/1234-solo-x/".into(), ..Default::default() };
    assert_eq!(AsuraScans.manga_url(&old).as_deref(), Some("https://asurascans.com/comics/solo-x"));
    let c = Chapter { url: "/series/overgeared/chapter/12.5".into(), memo: Some(json!({ "mangaSlug": "overgeared-1bb35e56" })), ..Default::default() };
    assert_eq!(AsuraScans.chapter_url(&c).as_deref(), Some("https://asurascans.com/comics/overgeared-1bb35e56/chapter/12.5"));
    assert_eq!(AsuraScans.chapter_url(&Chapter::default()), None);
}

#[test]
fn pages_are_read_from_the_page_and_scrambled_ones_get_a_shuffle() {
    set_context("en", &[]);
    mock(|u| (u == "https://asurascans.com/comics/overgeared-1bb35e56/chapter/1").then_some(CHAPTER));
    let ch = Chapter { url: "/series/overgeared/chapter/1".into(), memo: Some(json!({ "mangaSlug": "overgeared-1bb35e56" })), ..Default::default() };
    let pages = AsuraScans.pages(&ch).unwrap();
    assert_eq!(pages.len(), 3);
    assert!(pages[0].image_url.ends_with("/001.webp?v=1791877385"), "{}", pages[0].image_url);
    assert!(pages[0].ops.is_empty());

    mock(|_| Some(SCRAMBLED));
    let pages = AsuraScans.pages(&ch).unwrap();
    assert_eq!(pages[0].ops, [ImageOp::Shuffle { cols: 2, rows: 2, order: vec![1, 0, 3, 2] }]);
}

#[test]
fn filter_data_lists_genres_and_creators() {
    set_context("en", &[]);
    mock(|u| match u {
        "https://asurascans.com/browse" => Some(BROWSE),
        "https://api.asurascans.com/api/creators" => Some(CREATORS),
        _ => None,
    });
    let data = AsuraScans.filter_data().unwrap().unwrap();
    assert_eq!(data["genres"][0]["slug"], "action");
    assert_eq!(data["genres"].as_array().unwrap().len(), 3);
    let f = AsuraScans.filters(Some(&data));
    // sort, status, type, genres, creators, minimum chapters
    assert_eq!(f.len(), 6);
    assert!(matches!(&f[4], FilterDef::Select { options, .. } if options.len() == 1 + 2 + 2));
}

#[test]
fn astro_values_are_unwrapped() {
    let v = unwrap_astro(json!({"a": [0, 5], "b": [1, [[0, {"x": [0, "y"]}], [0, {"x": [0, "z"]}]]], "c": [0], "d": []}));
    assert_eq!(v, json!({"a": 5, "b": [{"x": "y"}, {"x": "z"}], "c": null, "d": null}));
}
