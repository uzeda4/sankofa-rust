//! The MangaBox flow against pages shaped like the real ones (the markup the Kotlin selectors were written for).

use super::*;

struct Test;
impl Theme for Test {
    const NAME: &'static str = "Test";
    const BASE_URL: &'static str = "https://site.test";
}

type S = Site<Test>;

const LIST: &str = r#"<div class="truyen-list">
  <div class="list-truyen-item-wrap"><a data-id="1" href="https://site.test/manga/one-piece"><img src="https://img.test/1.jpg"></a>
    <h3><a href="https://site.test/manga/one-piece">One Piece</a></h3></div>
  <div class="list-truyen-item-wrap"><a data-id="2" href="https://site.test/manga/naruto"><img src="/2.jpg"></a>
    <h3><a href="https://site.test/manga/naruto">Naruto</a></h3></div>
  <div class="list-truyen-item-wrap"><h3>no data-id</h3></div>
</div>
<div class="group-page"><a href="?page=1">1</a><a>2</a><a href="?page=3">3</a><a href="?page=9">Last</a></div>"#;

const DETAILS: &str = r#"<div class="manga-info-top">
  <div class="manga-info-pic"><img src="https://img.test/cover.jpg"></div>
  <ul class="manga-info-text"><li><h1>One Piece</h1></li>
    <li>Author(s) : <a href="/a/1">Oda</a>, <a href="/a/2">Eiichiro</a></li>
    <li>Status : Ongoing</li>
    <li class="genres">Genres : <a>Action</a><a>Adventure</a></li></ul>
</div>
<div id="noidungm">One Piece summary: A pirate story.<br>More</div>
<h2 class="story-alternative">Alternative : ワンピース</h2>"#;

const CHAPTER_PAGE: &str = r#"<script>var cdns = ["https://cdn1.test/", "https:\/\/cdn2.test"]; var backupImage = ["https://cdn3.test"];
var chapterImages = ["one-piece/1/001.webp", "/one-piece/1/002.webp"];</script>"#;

fn mock(f: impl Fn(&str) -> Option<&'static str> + 'static) {
    set_mock_http(move |r| match f(&r.url) {
        Some(body) => Ok(HttpResponse { status: 200, body: body.into(), url: r.url.clone(), ..Default::default() }),
        None => Ok(HttpResponse { status: 404, url: r.url.clone(), ..Default::default() }),
    });
}

#[test]
fn lists_keep_paths_slugs_and_the_next_page_flag() {
    set_context("en", &[]);
    mock(|u| (u == "https://site.test/manga-list/hot-manga?page=1").then_some(LIST));
    let page = S::default().popular(1).unwrap();
    assert_eq!(page.mangas.len(), 2, "the entry without data-id is skipped");
    assert_eq!(page.mangas[0].url, "/manga/one-piece");
    assert_eq!(page.mangas[0].title, "One Piece");
    assert_eq!(page.mangas[0].thumbnail_url.as_deref(), Some("https://img.test/1.jpg"));
    assert_eq!(page.mangas[1].thumbnail_url.as_deref(), Some("https://site.test/2.jpg"));
    assert_eq!(page.mangas[0].memo.as_ref().unwrap()["slug"], "one-piece");
    assert!(page.has_next);
}

#[test]
fn search_uses_the_alias_url_or_the_genre_filters() {
    set_context("en", &[]);
    mock(|u| (u == "https://site.test/search/story/one_piece_a?page=2").then_some(LIST));
    // the search selector matches all three wraps; the one without a link is dropped
    let r = S::default().search(2, "One-Piece à!", &Filters::default()).unwrap();
    assert_eq!(r.mangas.len(), 2);

    set_mock_http(|r| {
        assert_eq!(r.url, "https://site.test/genre/action?filter=9&page=3");
        Ok(HttpResponse { status: 200, body: "<p>none</p>".into(), url: r.url.clone(), ..Default::default() })
    });
    let f: Filters = serde_json::from_str(&format!(r#"{{"sort":2,"status":2,"genre":{}}}"#, GENRES.iter().position(|g| g.1 == "action").unwrap())).unwrap();
    assert!(S::default().search(3, "", &f).unwrap().mangas.is_empty());
}

#[test]
fn details_read_the_title_page() {
    set_context("en", &[]);
    mock(|u| (u == "https://site.test/manga/one-piece").then_some(DETAILS));
    let m = S::default().details(&Manga { url: "/manga/one-piece".into(), title: "x".into(), ..Default::default() }).unwrap();
    assert_eq!(m.title, "One Piece");
    assert_eq!(m.author.as_deref(), Some("Oda, Eiichiro"));
    assert_eq!(m.status, 1);
    assert_eq!(m.genre, ["Action", "Adventure"]);
    assert_eq!(m.thumbnail_url.as_deref(), Some("https://img.test/cover.jpg"));
    let d = m.description.unwrap();
    assert!(d.starts_with("A pirate story.") && d.ends_with("Alternative Name: Alternative : ワンピース"), "{d}");
    assert!(m.initialized);
    // the stored address is `/manga/<slug>` with the slug kept in the memo
    assert_eq!(m.url, "/manga/one-piece");
}

#[test]
fn chapters_come_from_the_api() {
    set_context("en", &[]);
    mock(|u| {
        (u == "https://site.test/api/manga/one-piece/chapters?limit=-1").then_some(
            r#"{"success":true,"data":{"chapters":[
              {"chapter_name":"Chapter 2","chapter_slug":"chapter-2","chapter_num":2,"updated_at":"2024-03-05T10:20:30.000Z"},
              {"chapter_name":null,"chapter_slug":"chapter-1","chapter_num":null,"updated_at":null},
              {"chapter_name":"no slug","chapter_slug":null}]}}"#,
        )
    });
    let manga = Manga { url: "/manga/one-piece".into(), title: "One Piece".into(), ..Default::default() };
    let ch = S::default().chapters(&manga).unwrap();
    assert_eq!(ch.len(), 2);
    assert_eq!((ch[0].name.as_str(), ch[0].url.as_str()), ("Chapter 2", "https://site.test/manga/one-piece/chapter-2"));
    assert_eq!(ch[0].date_upload, parse_iso("2024-03-05T10:20:30Z").unwrap());
    assert_eq!(ch[0].scanlator.as_deref(), Some("site.test"));
    assert_eq!((ch[1].name.as_str(), ch[1].chapter_number, ch[1].date_upload), ("Chapter", 0.0, 0));
}

#[test]
fn pages_build_cdn_urls_with_the_other_cdns_as_mirrors() {
    set_context("en", &[]);
    mock(|u| (u == "https://site.test/manga/one-piece/chapter-1").then_some(CHAPTER_PAGE));
    let chapter = Chapter { url: "https://site.test/manga/one-piece/chapter-1".into(), ..Default::default() };
    let pages = S::default().pages(&chapter).unwrap();
    assert_eq!(pages.len(), 2);
    assert_eq!(pages[0].image_url, "https://cdn1.test/one-piece/1/001.webp");
    assert_eq!(pages[1].image_url, "https://cdn1.test/one-piece/1/002.webp", "a leading slash does not double up");
    assert_eq!(pages[0].mirrors, ["https://cdn2.test", "https://cdn3.test"]);
    assert_eq!(pages[0].url, "https://site.test/manga/one-piece/chapter-1");
    assert!(pages[0].ops.is_empty());
}

#[test]
fn split_strips_are_merged_when_the_setting_is_on() {
    // widths 800: 400 + 400 + 400 fit while 3*800 > 2*h_so_far + h; the fourth is wider and stands alone
    let sizes = [(800u32, 400u32), (800, 400), (800, 400), (600, 300), (800, 1500)];
    let bytes: Vec<Vec<u8>> = sizes
        .iter()
        .map(|&(w, h)| {
            let mut b = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 13, b'I', b'H', b'D', b'R'];
            b.extend(w.to_be_bytes());
            b.extend(h.to_be_bytes());
            b
        })
        .collect();
    set_mock_http(move |r| {
        if r.url.ends_with(".png") {
            let n: usize = r.url.trim_end_matches(".png").rsplit('/').next().unwrap().parse().unwrap();
            use base64::Engine;
            return Ok(HttpResponse { status: 206, body_b64: Some(base64::engine::general_purpose::STANDARD.encode(&bytes[n])), url: r.url.clone(), ..Default::default() });
        }
        Ok(HttpResponse {
            status: 200,
            body: r#"<script>var cdns = ["https://cdn.test"]; var chapterImages = ["0.png","1.png","2.png","3.png","4.png"];</script>"#.into(),
            url: r.url.clone(),
            ..Default::default()
        })
    });
    set_context("en", &[(MERGE_PREF, json!(true))]);
    let pages = S::default().pages(&Chapter { url: "https://site.test/c".into(), ..Default::default() }).unwrap();
    assert_eq!(pages.len(), 3);
    assert_eq!(
        pages[0].ops,
        [ImageOp::Merge { urls: vec!["https://cdn.test/0.png".into(), "https://cdn.test/1.png".into(), "https://cdn.test/2.png".into()], width: 800, height: 1200 }]
    );
    assert!(pages[1].ops.is_empty() && pages[2].ops.is_empty());
}

#[test]
fn query_alias_matches_the_sites_function() {
    assert_eq!(normalize_query("One-Piece à!"), "one_piece_a");
    assert_eq!(normalize_query("  Đại  Chiến "), "dai_chien");
    assert_eq!(normalize_query("a__b"), "a_b");
}

#[test]
fn the_source_declares_its_filters_and_settings() {
    let info = S::default().info();
    assert!(info.prefs.iter().any(|p| matches!(p, PrefDef::Switch { key, .. } if key == MERGE_PREF)));
    let f = S::default().filters(None);
    assert_eq!(f.len(), 5);
    assert!(matches!(&f[4], FilterDef::Select { options, .. } if options.len() == GENRES.len()));
}
