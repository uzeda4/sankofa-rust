//! Read Comics Online against small pages shaped by the selectors of the Kotlin extension (the live site sits behind a
//! Cloudflare check, so no page could be saved for a fixture).

use super::*;

const LIST: &str = r#"<div class="comic-list-layout"><div class="grid">
<div class="group"><a href="/comic/batman"><img src="/img/no-image.png"></a><a class="block text-sm font-semibold" href="https://readcomicsonline.ru/comic/batman">Batman</a></div>
<div class="group"><a class="block text-sm font-semibold" href="/comic/spider-man%20x"><img src="https://img.test/s.jpg"></a>Spider-Man</div>
<div class="group"><a class="block text-sm font-semibold" href="/comic/batman">Batman</a></div>
</div></div><nav><a rel="next" href="?page=2">2</a></nav>"#;

const DETAILS: &str = r#"<h1 class="text-2xl">Batman</h1><img class="w-full rounded-xl" data-src="https://img.test/b.jpg">
<div class="flex flex-wrap gap-2"><span class="rounded-full">Ongoing</span></div>
<div><div><h3>Synopsis</h3></div><div><p>The Dark Knight.</p></div></div>
<dl><div><dt>Genres:</dt><a>Action</a><a>Crime</a></div></dl>
<div><span>Author:</span><a>Kane</a><a>Finger</a></div>
<div class="overflow-hidden border-ink-600">
<a href="/comic/batman/12"><span class="text-brand-400">Batman 12 : The fall</span><span class="text-slate-500">5 Mar 2024</span></a>
<a href="/comic/batman/11"><span class="text-brand-400">Batman 11</span></a></div>"#;

fn mock(f: impl Fn(&str) -> Option<&'static str> + 'static) {
    set_mock_http(move |r| match f(&r.url) {
        Some(body) => Ok(HttpResponse { status: 200, body: body.into(), url: r.url.clone(), ..Default::default() }),
        None => Ok(HttpResponse { status: 404, url: r.url.clone(), ..Default::default() }),
    });
}

#[test]
fn the_id_is_the_one_the_extension_had_on_the_device() {
    let info = ReadComicsOnline.info();
    assert_eq!(sk_source::extension_id(&info.name, &info.lang, info.version_id), 7_185_601_298_150_078_890);
}

#[test]
fn lists_use_the_placeholder_rule_and_latest_drops_repeats() {
    mock(|u| (u == "https://readcomicsonline.ru/comic-list?sort=views&page=1" || u.ends_with("sort=latest&page=1")).then_some(LIST));
    let page = ReadComicsOnline.popular(1).unwrap();
    assert_eq!(page.mangas.len(), 3);
    assert!(page.has_next);
    assert_eq!(page.mangas[0].url, "/comic/batman");
    assert_eq!(page.mangas[0].thumbnail_url.as_deref(), Some("https://readcomicsonline.ru/uploads/manga/batman/cover/cover_250x350.jpg"));
    assert_eq!(page.mangas[1].url, "/comic/spider-man x", "escapes are decoded the way the extension stored them");
    assert_eq!(page.mangas[1].thumbnail_url.as_deref(), Some("https://img.test/s.jpg"));
    assert_eq!(ReadComicsOnline.latest(1).unwrap().mangas.len(), 2);
}

#[test]
fn search_sends_the_name_and_both_text_filters() {
    mock(|u| {
        assert_eq!(u, "https://readcomicsonline.ru/advanced-search?name=bat%20man&page=2&release=1939&author=Kane");
        Some(r#"<div><a href="/comic/batman"><img src="https://img.test/b.jpg"><p>Batman</p></a></div><span><a rel="next" href="?page=3">3</a></span>"#)
    });
    let f: Filters = serde_json::from_str(r#"{"release":"1939","author":"Kane"}"#).unwrap();
    let page = ReadComicsOnline.search(2, "bat man", &f).unwrap();
    assert_eq!((page.mangas[0].url.as_str(), page.mangas[0].title.as_str()), ("/comic/batman", "Batman"));
    assert!(page.has_next);
}

#[test]
fn a_title_page_gives_details_and_chapters() {
    mock(|u| (u == "https://readcomicsonline.ru/comic/batman").then_some(DETAILS));
    let stub = Manga { url: "/comic/batman".into(), ..Default::default() };
    let up = ReadComicsOnline.update(&stub, true, true).unwrap();
    let m = up.manga.unwrap();
    assert_eq!((m.title.as_str(), m.status, m.author.as_deref()), ("Batman", 1, Some("Kane, Finger")));
    assert_eq!(m.description.as_deref(), Some("The Dark Knight."));
    assert_eq!(m.genre, ["Action", "Crime"]);
    assert_eq!(m.thumbnail_url.as_deref(), Some("https://img.test/b.jpg"));
    let ch = up.chapters.unwrap();
    assert_eq!((ch[0].url.as_str(), ch[0].name.as_str()), ("/comic/batman/12", "12: The fall"));
    assert_eq!(ch[0].date_upload, millis(2024, 3, 5, 0, 0, 0));
    assert_eq!((ch[1].name.as_str(), ch[1].date_upload), ("11", 0));
}

#[test]
fn pages_are_the_reader_images_and_other_sites_links_are_not_ours() {
    mock(|_| Some(r#"<div id="reader-all"><img data-src="https://i.test/1.jpg" src="x.gif"><img src="/2.jpg"></div>"#));
    let pages = ReadComicsOnline.pages(&Chapter { url: "/comic/batman/12".into(), ..Default::default() }).unwrap();
    assert_eq!(pages.iter().map(|p| p.image_url.as_str()).collect::<Vec<_>>(), ["https://i.test/1.jpg", "https://readcomicsonline.ru/2.jpg"]);
    assert!(ReadComicsOnline.by_url("https://other.test/comic/batman").unwrap().is_none());
    assert!(ReadComicsOnline.by_url("https://readcomicsonline.ru/comic").unwrap().is_none());
}

#[test]
fn chapter_names_drop_the_title_and_repeated_halves() {
    assert_eq!(clean_chapter_name("Batman", "Batman 12 : The fall"), "12: The fall");
    assert_eq!(clean_chapter_name("Batman", "Batman 12: 12"), "12");
    assert_eq!(clean_chapter_name("X", "Annual"), "Annual");
}
