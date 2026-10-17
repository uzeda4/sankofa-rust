//! Madara against small pages shaped like the real theme's output, plus the chapter protector round trip.

use super::*;
use cbc::cipher::BlockEncryptMut;

struct Demo;
impl Theme for Demo {
    const NAME: &'static str = "Demo";
    const BASE_URL: &'static str = "https://demo.test";
    const CHAPTER_MODE: ChapterMode = ChapterMode::MangaAjax;
}
type DemoSite = Site<Demo>;

const ARCHIVE: &str = r#"
<div class="page-item-detail manga"><div class="item-thumb"><a href="https://demo.test/manga/one-piece/"><img data-src="https://demo.test/c1.jpg" src="x.gif"></a></div>
<div class="post-title"><h3><a href="https://demo.test/manga/one-piece/">One Piece</a></h3></div><div data-post-id="77"></div></div>
<div class="page-item-detail manga"><div class="post-title"><h3><a href="https://demo.test/manga/naruto/">Naruto</a></h3></div><div data-post-id="78"></div></div>"#;

const DETAILS: &str = r#"<html><head><link rel="shortlink" href="https://demo.test/?p=77"></head><body>
<div class="post-title"><h1><span class="manga-title-badges">HOT</span> One Piece</h1></div>
<div class="summary_image"><img data-src="https://demo.test/cover.jpg"></div>
<div class="author-content"><a>Oda</a></div><div class="artist-content"><a>Updating</a></div>
<div class="genres-content"><a href="https://demo.test/manga-genre/action/">Action</a>, <a href="https://demo.test/manga-genre/manhwa/">Manhwa</a></div>
<div class="summary-heading"><h5>Status</h5></div><div class="summary-content">OnGoing</div>
<div class="description-summary"><div class="summary__content"><p>Pirates.</p><p>More.</p></div></div></body></html>"#;

const CHAPTERS: &str = r#"<ul><li class="wp-manga-chapter"><a href="https://demo.test/manga/one-piece/chapter-2/">Chapter 2</a><span class="chapter-release-date"><i>March 5, 2024</i></span></li>
<li class="wp-manga-chapter"><a href="https://demo.test/manga/one-piece/chapter-1/">Chapter 1</a><span class="chapter-release-date"><a title="3 days ago"></a></span></li></ul>"#;

fn mock(f: impl Fn(&HttpRequest) -> Option<String> + 'static) {
    set_mock_http(move |r| match f(r) {
        Some(body) => Ok(HttpResponse { status: 200, body, url: r.url.clone(), ..Default::default() }),
        None => Ok(HttpResponse { status: 404, url: r.url.clone(), ..Default::default() }),
    });
}

#[test]
fn lists_post_the_ajax_form_and_titles_are_post_ids() {
    set_context("en", &[]);
    mock(|r| {
        assert_eq!(r.url, "https://demo.test/wp-admin/admin-ajax.php");
        let body = r.body.clone().unwrap();
        assert!(body.contains("action=madara_load_more&page=1&"), "{body}");
        assert!(body.contains("vars%5Bmeta_key%5D=_wp_manga_views"), "{body}");
        Some(ARCHIVE.into())
    });
    let page = DemoSite::default().popular(2).unwrap();
    assert_eq!(page.mangas.len(), 2);
    assert!(!page.has_next, "two of twenty-five is the last page");
    let m = &page.mangas[0];
    assert_eq!((m.url.as_str(), m.title.as_str()), ("77", "One Piece"));
    assert_eq!(m.thumbnail_url.as_deref(), Some("https://demo.test/c1.jpg"));
    assert_eq!(m.memo.as_ref().unwrap()["path"], "/manga/one-piece/");
}

#[test]
fn search_filters_become_tax_and_meta_queries() {
    set_context("en", &[]);
    mock(|r| {
        let body = r.body.clone().unwrap();
        for part in [
            "vars%5Bs%5D=luffy",
            "vars%5Btax_query%5D%5B0%5D%5Bterms%5D=Oda",
            "vars%5Bmeta_query%5D%5B1%5D%5Bvalue%5D%5B0%5D=end",
            "vars%5Btax_query%5D%5B1%5D%5Boperation%5D=AND",
            "vars%5Btax_query%5D%5B1%5D%5Bterms%5D%5B0%5D=action",
        ] {
            assert!(body.contains(part), "{part} missing from {body}");
        }
        Some(ARCHIVE.into())
    });
    let f: Filters = serde_json::from_str(r#"{"author":"Oda","status":{"end":true},"genre_condition":1,"genres":{"action":true}}"#).unwrap();
    DemoSite::default().search(1, "luffy", &f).unwrap();
}

#[test]
fn details_and_chapters_use_the_stored_path_and_the_ajax_endpoint() {
    set_context("en", &[]);
    mock(|r| match (r.method.as_str(), r.url.as_str()) {
        ("GET", "https://demo.test/manga/one-piece/") => Some(DETAILS.into()),
        ("POST", "https://demo.test/manga/one-piece/ajax/chapters/") => Some(CHAPTERS.into()),
        _ => None,
    });
    let stub = Manga { url: "77".into(), memo: Some(json!({ "path": "/manga/one-piece/" })), ..Default::default() };
    let up = DemoSite::default().update(&stub, true, true).unwrap();
    let m = up.manga.unwrap();
    assert_eq!((m.url.as_str(), m.title.as_str(), m.status), ("77", "One Piece", 1));
    assert_eq!((m.author.as_deref(), m.artist), (Some("Oda"), None), "'Updating' is not an artist");
    assert_eq!(m.description.as_deref(), Some("Pirates.\n\nMore."));
    assert_eq!(m.genre, ["Action", "Manhwa"]);
    let ch = up.chapters.unwrap();
    assert_eq!(ch.len(), 2);
    assert_eq!(ch[0].url, "chapter-2");
    assert_eq!(ch[0].date_upload, parse_date("March 5, 2024", "MMMM d, yyyy", "en").unwrap());
    assert_eq!(DemoSite::default().chapter_url(&ch[0]).as_deref(), Some("https://demo.test/manga/one-piece/chapter-2/"));
    assert_eq!(DemoSite::default().manga_url(&stub).as_deref(), Some("https://demo.test/manga/one-piece/"));
    // chapters alone must not load the title page
    mock(|r| (r.method == "POST").then(|| CHAPTERS.into()));
    assert_eq!(DemoSite::default().update(&stub, false, true).unwrap().chapters.unwrap().len(), 2);
}

#[test]
fn a_title_stored_by_path_gets_its_id_from_the_page() {
    set_context("en", &[]);
    mock(|r| match r.url.as_str() {
        "https://demo.test/manga/one-piece/" => Some(DETAILS.into()),
        "https://demo.test/manga/one-piece/ajax/chapters/" => Some(CHAPTERS.into()),
        _ => None,
    });
    let old = Manga { url: "/manga/one-piece/".into(), ..Default::default() };
    let up = DemoSite::default().update(&old, true, true).unwrap();
    let m = up.manga.unwrap();
    assert_eq!(m.url, "/manga/one-piece/", "the stored address stays, the id goes to the memo");
    assert_eq!(m.memo.unwrap()["id"], "77");
    assert_eq!(up.chapters.unwrap().len(), 2);
}

#[test]
fn pages_are_plain_images_or_decrypted_from_the_protector() {
    set_context("en", &[]);
    let ch = Chapter { url: "chapter-1".into(), memo: Some(json!({ "mangaPath": "/manga/one-piece/" })), ..Default::default() };
    mock(|_| Some(r#"<div class="page-break"><img data-src=" https://i.test/1.jpg "></div><div class="page-break"><img src="https://i.test/2.jpg"></div>"#.into()));
    let pages = DemoSite::default().pages(&ch).unwrap();
    assert_eq!(pages.len(), 2);
    assert_eq!(pages[0].image_url, "https://i.test/1.jpg");
    assert_eq!(pages[0].headers[0].0, "Referer");

    // encrypt the way the site's CryptoJS does, then read it back through the page
    let (password, salt) = ("pw-123", [1u8, 2, 3, 4, 5, 6, 7, 8]);
    let inner = serde_json::to_string(&json!(["https://i.test/a.jpg", "https://i.test/b.jpg"])).unwrap();
    let plain = serde_json::to_string(&inner).unwrap();
    let mut key_iv = Vec::new();
    let mut prev = Vec::new();
    while key_iv.len() < 48 {
        let mut h = Md5::new();
        h.update(&prev);
        h.update(password.as_bytes());
        h.update(salt);
        prev = h.finalize().to_vec();
        key_iv.extend_from_slice(&prev);
    }
    let mut buf = plain.clone().into_bytes();
    let len = buf.len();
    buf.resize(len + 16, 0);
    let enc = cbc::Encryptor::<Aes256>::new_from_slices(&key_iv[..32], &key_iv[32..48]).unwrap().encrypt_padded_mut::<Pkcs7>(&mut buf, len).unwrap().to_vec();
    let ct = base64::engine::general_purpose::STANDARD.encode(enc);
    let salt_hex: String = salt.iter().map(|b| format!("{b:02x}")).collect();
    let script = format!("var wpmangaprotectornonce='{password}';var chapter_data='{{\"ct\":\"{ct}\",\"s\":\"{salt_hex}\"}}';");
    let page = format!(r#"<script id="chapter-protector-data" src="data:text/javascript;base64,{}"></script>"#, base64::engine::general_purpose::STANDARD.encode(script));
    mock(move |_| Some(page.clone()));
    let pages = DemoSite::default().pages(&ch).unwrap();
    assert_eq!(pages.iter().map(|p| p.image_url.as_str()).collect::<Vec<_>>(), ["https://i.test/a.jpg", "https://i.test/b.jpg"]);
}

#[test]
fn relative_dates_in_several_languages() {
    let now = parse_date("June 15, 2024", "MMMM d, yyyy", "en").unwrap() + 15 * 3_600_000;
    let day = 86_400_000;
    let midnight = now - 15 * 3_600_000;
    assert_eq!(parse_chapter_date(Some("Today"), "MMMM d, yyyy", now), midnight);
    assert_eq!(parse_chapter_date(Some("ontem"), "MMMM d, yyyy", now), midnight - day);
    assert_eq!(parse_chapter_date(Some("3 days ago"), "MMMM d, yyyy", now), now - 3 * day);
    assert_eq!(parse_chapter_date(Some("2 horas atrás"), "MMMM d, yyyy", now), now - 2 * 3_600_000);
    assert_eq!(parse_chapter_date(Some("gibberish"), "MMMM d, yyyy", now), 0);
    assert_eq!(parse_chapter_date(None, "MMMM d, yyyy", now), 0);
}

#[test]
fn statuses_read_in_the_languages_sites_use() {
    for (s, v) in [("OnGoing", 1), ("Completed", 2), ("end", 2), ("Em Lançamento", 1), ("Canceled", 5), ("On Hold", 6), ("???", 0)] {
        assert_eq!(to_status(s), v, "{s}");
    }
}
