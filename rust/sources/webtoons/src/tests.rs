//! Webtoons against pages and API answers saved from the live site (English, trimmed).

use super::*;

const RANK: &str = include_str!("../tests/fixtures/rank.html");
const SEARCH: &str = include_str!("../tests/fixtures/search.html");
const LATEST: &str = include_str!("../tests/fixtures/latest.html");
const DETAIL: &str = include_str!("../tests/fixtures/detail.html");
const EPISODES: &str = include_str!("../tests/fixtures/episodes.json");
const VIEWER: &str = include_str!("../tests/fixtures/viewer.html");

fn mock(f: impl Fn(&str) -> Option<&'static str> + 'static) {
    set_mock_http(move |r| match f(&r.url) {
        Some(body) => Ok(HttpResponse { status: 200, body: body.into(), url: r.url.clone(), ..Default::default() }),
        None => Ok(HttpResponse { status: 404, url: r.url.clone(), ..Default::default() }),
    });
}

fn mock_owned(body: String) {
    set_mock_http(move |r| Ok(HttpResponse { status: 200, body: body.clone(), url: r.url.clone(), ..Default::default() }));
}

#[test]
fn ids_are_the_ones_the_extensions_had_on_the_device() {
    // generated for en, de, th, fr, es; the other two keep fixed ids
    for (lang, id) in [("en", 2_522_335_540_328_470_744_i64), ("de", 1_486_822_823_904_297_379), ("th", 3_330_861_233_813_615_504), ("fr", 8_251_758_253_314_457_060), ("es", 8_664_735_734_660_636_295)] {
        set_context(lang, &[]);
        let info = Webtoons.info();
        assert_eq!(sk_source::extension_id(&info.name, &info.lang, info.version_id), id, "{lang}");
    }
    set_context("id", &[]);
    assert_eq!(Webtoons.info().id, 8_749_627_068_478_740_298);
    set_context("zh-Hant", &[]);
    let info = Webtoons.info();
    assert_eq!(info.id, 2_959_982_438_613_576_472);
    assert!(info.cookies.iter().any(|c| c.name == "locale" && c.value == "zh_TW"));
    set_context("en", &[]);
    assert_eq!(Webtoons.info().id, 0, "derived by the host from name, language and version");
}

#[test]
fn rankings_are_pages_one_to_four() {
    set_context("en", &[]);
    mock(|u| (u == "https://www.webtoons.com/en/ranking/trending").then_some(RANK));
    let page = Webtoons.popular(1).unwrap();
    assert!(!page.mangas.is_empty());
    assert!(page.mangas[0].url.starts_with("/en/") && page.mangas[0].url.contains("title_no="), "{}", page.mangas[0].url);
    assert!(!page.mangas[0].title.is_empty());
    assert!(page.has_next);
    assert!(Webtoons.popular(5).unwrap_err().contains("page > 4"));
    mock(|u| (u == "https://www.webtoons.com/zh-hant/ranking/canvas").then_some(RANK));
    set_context("zh-Hant", &[]);
    assert!(!Webtoons.popular(4).unwrap().has_next);
}

#[test]
fn search_goes_to_the_site_search_and_pages_only_for_a_typed_search() {
    set_context("en", &[]);
    mock(|u| {
        assert_eq!(u, "https://www.webtoons.com/en/search?keyword=tower%20of");
        Some(SEARCH)
    });
    Webtoons.search(3, "tower of", &Filters::default()).unwrap();
    mock(|u| {
        assert_eq!(u, "https://www.webtoons.com/en/search/canvas?keyword=x&page=2");
        Some(SEARCH)
    });
    let f: Filters = serde_json::from_str(r#"{"type":2}"#).unwrap();
    Webtoons.search(2, "x", &f).unwrap();
}

#[test]
fn latest_reads_todays_originals() {
    set_context("en", &[]);
    mock(|u| {
        assert!(u.starts_with("https://www.webtoons.com/en/originals/") && u.ends_with("?sortOrder=UPDATE"), "{u}");
        Some(LATEST)
    });
    assert!(!Webtoons.latest(1).unwrap().mangas.is_empty());
}

#[test]
fn details_read_the_desktop_title_page() {
    set_context("en", &[]);
    mock_owned(DETAIL.into());
    let stub = Manga { url: "/en/fantasy/tower-of-god/list?title_no=95".into(), ..Default::default() };
    let m = Webtoons.details(&stub).unwrap();
    assert_eq!(m.title, "Tower of God");
    assert_eq!(m.author.as_deref(), Some("SIU"));
    assert_eq!(m.genre, ["Fantasy"]);
    assert_eq!(m.status, 1, "EVERY MONDAY");
    assert!(m.description.as_deref().unwrap().starts_with("What do you desire?"));
    assert!(m.thumbnail_url.as_deref().unwrap().starts_with("https://"), "{:?}", m.thumbnail_url);

    // a cover already in the library stays unless it was the site's banner
    let kept = Manga { thumbnail_url: Some("https://x.test/covers/my-cover.jpg".into()), ..stub.clone() };
    assert_eq!(Webtoons.details(&kept).unwrap().thumbnail_url.as_deref(), Some("https://x.test/covers/my-cover.jpg"));
}

#[test]
fn episodes_are_numbered_across_seasons_and_listed_newest_first() {
    set_context("en", &[]);
    mock(|u| {
        assert_eq!(u, "https://m.webtoons.com/api/v1/webtoon/95/episodes?pageSize=99999");
        Some(EPISODES)
    });
    let stub = Manga { url: "/en/fantasy/tower-of-god/list?title_no=95".into(), ..Default::default() };
    let ch = Webtoons.chapters(&stub).unwrap();
    assert_eq!(ch.len(), 6);
    // season 1 ep 0 and 1, season 2 ep 0 and 1 (its count restarts: offset 1), season 3 ep 234 and 235 (offset stays 1)
    let by_name: Vec<(&str, f32)> = ch.iter().map(|c| (c.name.as_str(), c.chapter_number)).collect();
    assert_eq!(by_name[0].0, "[Season 3] Ep. 235 (Season 3 Finale) (ch. 236)");
    assert_eq!(ch.last().unwrap().name, "[Season 1] Ep. 0 (ch. 0)");
    assert!(ch.windows(2).all(|w| w[0].chapter_number >= w[1].chapter_number), "{by_name:?}");
    assert!(ch[0].url.contains("/viewer?title_no=95&episode_no="));
    assert!(ch[0].date_upload > 1_400_000_000_000);

    set_context("en", &[(SEQUENTIAL, json!(true))]);
    let seq = Webtoons.chapters(&stub).unwrap();
    assert_eq!(seq[0].chapter_number, 6.0);
    assert_eq!(seq.last().unwrap().chapter_number, 1.0);
}

#[test]
fn old_address_forms_still_find_their_episodes() {
    set_context("en", &[]);
    mock(|u| {
        assert_eq!(u, "https://m.webtoons.com/api/v1/canvas/304446/episodes?pageSize=99999&readingLanguageCode=en");
        Some(EPISODES)
    });
    let old = Manga { url: "/challenge/episodeList?titleNo=304446".into(), ..Default::default() };
    assert_eq!(Webtoons.chapters(&old).unwrap().len(), 6);
    let bad = Manga { url: "/en/x/list".into(), ..Default::default() };
    assert!(Webtoons.chapters(&bad).unwrap_err().starts_with("Migrate"));
}

#[test]
fn pages_are_the_viewer_images_and_the_authors_note_is_optional() {
    set_context("en", &[]);
    mock_owned(VIEWER.into());
    let ch = Chapter { url: "/en/fantasy/tower-of-god/season-1-ep-0/viewer?title_no=95&episode_no=1".into(), ..Default::default() };
    let pages = Webtoons.pages(&ch).unwrap();
    assert_eq!(pages.len(), 3);
    assert!(pages[0].image_url.contains("?type=q90"), "{}", pages[0].image_url);

    set_context("en", &[(MAX_QUALITY, json!(true))]);
    let hq = Webtoons.pages(&ch).unwrap();
    assert!(!hq[0].image_url.contains("type="), "{}", hq[0].image_url);

    mock_owned(
        r#"<div id="_imageList"><img data-url="https://i.test/1.jpg"></div>
        <div class="creator_note"><div class="author_name"><span> Siu </span></div><p class="author_text">Thanks for reading!</p></div>"#
            .into(),
    );
    set_context("en", &[(SHOW_NOTES, json!(true))]);
    let with_note = Webtoons.pages(&ch).unwrap();
    assert_eq!(with_note.len(), 2);
    assert_eq!(with_note[1].ops, [ImageOp::Text { title: "Author's Notes from Siu".into(), body: "Thanks for reading!".into() }]);
}

#[test]
fn number_formatting_and_entities() {
    assert_eq!(format_number(5.0), "5");
    assert_eq!(format_number(5.5), "5.5");
    assert_eq!(format_number(5.01), "5.01");
    assert_eq!(unescape("Tom &amp; Jerry&#39;s"), "Tom & Jerry's");
}
