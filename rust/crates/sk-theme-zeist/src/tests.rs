//! The ZeistManga flow against feeds and pages shaped like the real ones.

use super::*;

struct Test;
impl Theme for Test {
    const NAME: &'static str = "Test";
    const BASE_URL: &'static str = "https://blog.test";
    const DETAILS_SELECTOR: &'static str = "main";
    const DETAILS_GENRES: &'static str = "dl.flex:contains(Genre) a[rel=tag]";
    const DETAILS_INFO: &'static str = "dl.flex";
    const DETAILS_INFO_TITLE: &'static str = "dt";
    const DETAILS_INFO_DESCRIPTION: &'static str = "dd";
}
type S = Site<Test>;

fn feed(entries: &str, total: u32) -> String {
    format!(r#"{{"feed":{{"openSearch$totalResults":{{"$t":"{total}"}},"openSearch$itemsPerPage":{{"$t":"2"}},"entry":[{entries}]}}}}"#)
}

fn entry(title: &str, path: &str, category: &str, extra: &str) -> String {
    format!(
        r#"{{"title":{{"$t":"{title}"}},"published":{{"$t":"2024-03-05T10:20:30.000+00:00"}},"category":[{{"term":"{category}"}}],
        "link":[{{"rel":"replies","href":"x"}},{{"rel":"alternate","href":"https://blog.test{path}"}}],{extra}"x":0}}"#
    )
}

fn mock(f: impl Fn(&str) -> Option<String> + 'static) {
    set_mock_http(move |r| match f(&r.url) {
        Some(body) => Ok(HttpResponse { status: 200, body, url: r.url.clone(), ..Default::default() }),
        None => Ok(HttpResponse { status: 404, url: r.url.clone(), ..Default::default() }),
    });
}

#[test]
fn the_latest_feed_is_a_list_with_a_next_page_flag() {
    let thumb = r#""media$thumbnail":{"url":"https://img.test/abc/s72-c/pic.jpg"},"#;
    let mut entries: Vec<String> = (0..21).map(|i| entry(&format!("T{i}"), &format!("/series/t{i}"), "Series", thumb)).collect();
    entries.push(entry("Skipped", "/novel", "Novel", ""));
    let body = feed(&entries.join(","), 99);
    mock(move |u| {
        assert!(u.starts_with("https://blog.test/feeds/posts/default/-/Series?alt=json&orderby=published&max-results=21&start-index=21"), "{u}");
        Some(body.clone())
    });
    let page = S::default().latest(2).unwrap();
    assert_eq!(page.mangas.len(), 20, "the 21st entry only says there is another page");
    assert!(page.has_next);
    assert_eq!(page.mangas[0].url, "/series/t0");
    assert_eq!(page.mangas[0].thumbnail_url.as_deref(), Some("https://img.test/abc/w600/pic.jpg"));
    // a site without a latest list answers popular with it
    assert!(<Site<Test> as Source>::info(&S::default()).supports_latest);
}

#[test]
fn search_puts_the_label_in_the_query() {
    mock(|u| {
        assert!(u.contains("q=label:Series+naruto"), "{u}");
        Some(feed("", 0))
    });
    assert!(S::default().search(1, "naruto", &Filters::default()).unwrap().mangas.is_empty());
}

#[test]
fn details_and_chapters_follow_the_feed_the_page_points_to() {
    let page = r#"<main><img src="/cover.jpg"><h1>Title</h1><div id="synopsis">A story.</div>
        <dl class="flex"><dt>Status</dt><dd>Ongoing</dd></dl>
        <dl class="flex"><dt>Author</dt><dd>Jane</dd></dl>
        <dl class="flex"><dt>Genre</dt><dd><a rel="tag">Action</a><a rel="tag">Drama</a></dd></dl></main>
        <div id="clwd"><script>clwd.run('Title Label')</script></div>"#;
    let first = feed(&format!("{},{}", entry("Chapter 2", "/c/2", "Chapter", ""), entry("Chapter 1", "/c/1", "Chapter", "")), 3);
    let second = feed(&entry("Chapter 0", "/c/0", "Chapter", ""), 3);
    mock(move |u| {
        if u == "https://blog.test/series/title" {
            Some(page.into())
        } else if u.starts_with("https://blog.test/feeds/posts/default/-/Chapter/Title%20Label") && u.contains("start-index=1&") {
            Some(first.clone())
        } else if u.contains("start-index=3&") {
            Some(second.clone())
        } else {
            panic!("unexpected request {u}")
        }
    });
    let manga = Manga { url: "/series/title".into(), title: "Title".into(), ..Default::default() };
    let up = S::default().update(&manga, true, true).unwrap();
    let m = up.manga.unwrap();
    assert_eq!((m.status, m.author.as_deref(), m.genre.as_slice()), (1, Some("Jane"), ["Action".to_string(), "Drama".to_string()].as_slice()));
    assert_eq!(m.thumbnail_url.as_deref(), Some("https://blog.test/cover.jpg"));
    assert_eq!(m.description.as_deref(), Some("A story."));
    assert!(m.memo.as_ref().unwrap()["feedUrl"]["url"].as_str().unwrap().contains("Title%20Label"));
    let ch = up.chapters.unwrap();
    assert_eq!(ch.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(), ["Chapter 2", "Chapter 1", "Chapter 0"]);
    assert_eq!(ch[0].url, "/c/2");
    assert_eq!(ch[0].date_upload, parse_iso("2024-03-05T10:20:30Z").unwrap());

    // the second time the remembered feed address is used and the page is not needed for the chapters
    let again = S::default().update(&Manga { memo: m.memo.clone(), ..manga }, false, true).unwrap();
    assert_eq!(again.chapters.unwrap().len(), 3);
}

#[test]
fn pages_are_the_images_in_the_post() {
    mock(|_| Some(r#"<div class="check-box"><div class="separator"><img src="/1.jpg"><img data-x="y"><img src="https://c.test/2.jpg"></div></div>"#.into()));
    let pages = S::default().pages(&Chapter { url: "/c/1".into(), ..Default::default() }).unwrap();
    assert_eq!(pages.iter().map(|p| p.image_url.as_str()).collect::<Vec<_>>(), ["https://blog.test/1.jpg", "https://c.test/2.jpg"]);
}

#[test]
fn helpers() {
    assert_eq!(replace_last("a?q=1&q=2", "q=", "q=x+"), "a?q=1&q=x+2");
    assert_eq!(set_query("https://x/y?a=1&start-index=5", "start-index", "9").unwrap(), "https://x/y?a=1&start-index=9");
    assert_eq!(parse_status::<Test>(" Em lançamento "), 1);
    assert_eq!(parse_status::<Test>("Dropado"), 5);
}
