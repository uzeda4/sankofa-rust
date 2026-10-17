//! MangaFire: the request signature, the chapter naming rules, and (ignored by default) the live API through `curl`.

use super::*;

fn mock_json(f: impl Fn(&str) -> Option<String> + 'static) {
    set_mock_http(move |r| match f(&r.url) {
        Some(body) => Ok(HttpResponse { status: 200, body, url: r.url.clone(), ..Default::default() }),
        None => Ok(HttpResponse { status: 404, url: r.url.clone(), ..Default::default() }),
    });
}

#[test]
fn ids_are_the_ones_the_extensions_had_on_the_device() {
    for (lang, id) in [
        ("en", 6_084_907_896_154_116_083_i64),
        ("ja", 524_110_931_382_140_732),
        ("pt", 758_400_875_161_895_515),
        ("es-419", 3_849_324_770_075_062_403),
        ("fr", 6_228_021_941_803_932_286),
        ("pt-BR", 7_842_137_233_968_841_729),
        ("es", 8_098_539_940_032_331_958),
    ] {
        assert_eq!(sk_source::extension_id("MangaFire", lang, 1), id, "{lang}");
    }
}

#[test]
fn the_signature_covers_the_sorted_query_with_indexed_array_keys() {
    // parameters are sorted by name; `[]` keys get 0, 1, ... in order; the signature does not depend on the order given
    let a = signed_url("/api/titles", &[("page", "1".into()), ("genres_in[]", "5".into()), ("genres_in[]", "6".into()), ("limit", "50".into())]);
    let b = signed_url("/api/titles", &[("limit", "50".into()), ("genres_in[]", "5".into()), ("page", "1".into()), ("genres_in[]", "6".into())]);
    assert_eq!(a, b);
    assert!(a.starts_with("https://mangafire.to/api/titles?genres_in%5B%5D=5&genres_in%5B%5D=6&limit=50&page=1&vrf="), "{a}");
    let signed = vrf::sign("/titles?genres_in[0]=5&genres_in[1]=6&limit=50&page=1");
    assert!(a.ends_with(&format!("&vrf={signed}")), "{a}");
    // no query: no `?` in what is signed
    assert!(signed_url("/api/titles/abc", &[]).ends_with(&format!("?vrf={}", vrf::sign("/titles/abc"))));
}

#[test]
fn titles_come_in_pages_and_urls_keep_the_slug() {
    set_context("en", &[]);
    mock_json(|u| {
        assert!(u.starts_with("https://mangafire.to/api/titles?limit=50&order%5Bviews_30d%5D=desc&page=2&vrf="), "{u}");
        Some(r#"{"items":[{"hid":"ab12","slug":"one-piece","title":"One Piece","poster":{"small":"s","large":"l"}},{"hid":"cd34","title":"No Slug"}],"meta":{"lastPage":3,"hasNext":true}}"#.into())
    });
    let page = MangaFire.popular(2).unwrap();
    assert_eq!(page.mangas[0].url, "/title/ab12-one-piece");
    assert_eq!(page.mangas[0].thumbnail_url.as_deref(), Some("l"));
    assert_eq!(page.mangas[1].url, "/title/cd34");
    assert!(page.has_next);
}

#[test]
fn the_content_rating_setting_filters_lists() {
    set_context("en", &[(CONTENT_RATING_PREF, json!(["safe", "erotica"]))]);
    mock_json(|u| {
        assert!(u.contains("content_rating%5B%5D=erotica&content_rating%5B%5D=safe") || u.contains("content_rating%5B%5D=safe&content_rating%5B%5D=erotica"), "{u}");
        Some(r#"{"items":[]}"#.into())
    });
    MangaFire.latest(1).unwrap();
    // and the filter sheet opens with those two ticked
    let f = MangaFire.filters(None);
    let FilterDef::Group { items, .. } = &f[0] else { panic!("first filter is the content rating") };
    let on: Vec<&str> = items.iter().filter_map(|i| match i { FilterDef::Checkbox { key, default: true, .. } => Some(key.as_str()), _ => None }).collect();
    assert_eq!(on, ["safe", "erotica"]);
}

#[test]
fn addresses_and_ids() {
    assert_eq!(hid("/title/ab12-one-piece"), "ab12");
    assert_eq!(hid("/title/one-piece.ab12"), "ab12");
    assert_eq!(hid("/title/ab12/"), "ab12");
    assert_eq!(hid("ab12"), "ab12");
    set_context("pt-BR", &[]);
    assert_eq!(lang_code(), "pt-br");
    set_context("es-419", &[]);
    assert_eq!(lang_code(), "es-la");
}

#[test]
fn chapters_are_named_and_numbered_the_way_the_extension_did() {
    set_context("en", &[]);
    let dto = |json: &str| serde_json::from_str::<ChapterDto>(json).unwrap().to_chapter("/title/ab12-x", "en");
    let plain = dto(r#"{"id":7,"number":12.5,"createdAt":1700000000,"type":"Official"}"#);
    assert_eq!((plain.name.as_str(), plain.chapter_number, plain.url.as_str()), ("Ch. 12.5", 12.5, "/title/ab12-x/7-chapter-12.5-en"));
    assert_eq!((plain.scanlator.as_deref(), plain.date_upload), (Some("Official"), 1_700_000_000_000));
    let titled = dto(r#"{"id":8,"number":13,"name":"The storm"}"#);
    assert_eq!((titled.name.as_str(), titled.chapter_number, titled.scanlator.as_deref()), ("Ch. 13 - The storm", 13.0, Some("Unknown")));
    let stated = dto(r#"{"id":9,"number":14,"name":"Chapter 14.5: Extra"}"#);
    assert_eq!((stated.name.as_str(), stated.chapter_number), ("Chapter 14.5: Extra", 14.5));
    assert_eq!(plain.url.contains("-chapter-12.5-"), true);
}

#[test]
fn all_chapters_are_paged_and_duplicates_can_be_merged() {
    set_context("en", &[]);
    mock_json(|u| {
        if u.contains("/chapters?") && u.contains("page=1&") {
            Some(r#"{"items":[{"id":1,"number":2,"type":"Fan"},{"id":2,"number":1,"type":"Fan"}],"meta":{"lastPage":2}}"#.into())
        } else if u.contains("/chapters?") && u.contains("page=2&") {
            Some(r#"{"items":[{"id":3,"number":2,"type":"Official"}],"meta":{"lastPage":2}}"#.into())
        } else {
            None
        }
    });
    let manga = Manga { url: "/title/ab12-x".into(), ..Default::default() };
    assert_eq!(MangaFire.chapters(&manga).unwrap().len(), 3);

    set_context("en", &[(MERGE_CHAPTERS, json!(true))]);
    let merged = MangaFire.chapters(&manga).unwrap();
    assert_eq!(merged.len(), 2, "one per number");
    assert_eq!(merged[0].scanlator.as_deref(), Some("Official"), "the official one wins and the newest number is first");

    set_context("en", &[(MERGE_CHAPTERS, json!(true)), (PREFER_OFFICIAL, json!(false))]);
    assert_eq!(MangaFire.chapters(&manga).unwrap()[0].scanlator.as_deref(), Some("Fan"));
}

#[test]
fn pages_come_from_the_chapter_or_volume_endpoint() {
    set_context("en", &[]);
    mock_json(|u| {
        assert!(u.starts_with("https://mangafire.to/api/chapters/42?vrf="), "{u}");
        Some(r#"{"data":{"pages":[{"url":"https://i.test/1.jpg"},{"url":"https://i.test/2.jpg"}]}}"#.into())
    });
    let pages = MangaFire.pages(&Chapter { url: "/title/ab12-x/42-chapter-1-en".into(), ..Default::default() }).unwrap();
    assert_eq!(pages.len(), 2);
    mock_json(|u| {
        assert!(u.starts_with("https://mangafire.to/api/volumes/9?vrf="), "{u}");
        Some(r#"{"data":{"pages":[]}}"#.into())
    });
    MangaFire.pages(&Chapter { url: "/title/ab12-x/volume/9".into(), ..Default::default() }).unwrap();
    assert!(MangaFire.pages(&Chapter { url: "/title/ab12-x/nonsense".into(), ..Default::default() }).is_err());
}

#[test]
fn a_captcha_is_reported_not_swallowed() {
    set_context("en", &[]);
    set_mock_http(|r| Ok(HttpResponse { status: 403, body: r#"{"error":"captcha_required"}"#.into(), url: r.url.clone(), ..Default::default() }));
    assert!(MangaFire.popular(1).unwrap_err().starts_with("Shape-selecting captcha detected"));
}

/// Needs network: `cargo test -p sk-source-mangafire live -- --ignored --nocapture`. Proves the signature against the
/// real site (an unsigned or wrongly signed request is refused).
#[test]
#[ignore]
fn live_api_accepts_the_signature() {
    use sk_source::testing::Curl;
    use sk_source::{Http, HttpRequest};
    let curl = Curl::default();
    set_context("en", &[]);
    set_mock_http(move |r| {
        let reply = curl.request(&HttpRequest { method: r.method.clone(), url: r.url.clone(), headers: r.headers.clone(), follow_redirects: true, ..Default::default() });
        match reply.error {
            Some(e) => Err(e),
            None => Ok(HttpResponse { status: reply.status, headers: reply.headers, body: reply.body, url: reply.url, ..Default::default() }),
        }
    });
    let page = MangaFire.popular(1).unwrap();
    println!("popular: {} titles, first {:?}", page.mangas.len(), page.mangas.first().map(|m| (&m.title, &m.url)));
    assert!(!page.mangas.is_empty());
    let first = &page.mangas[0];
    let details = MangaFire.details(first).unwrap();
    println!("details: {} / {:?} / status {}", details.title, details.genre, details.status);
    let chapters = MangaFire.chapters(first).unwrap();
    println!("chapters: {}, newest {:?}", chapters.len(), chapters.first().map(|c| &c.name));
    assert!(!chapters.is_empty());
    let pages = MangaFire.pages(&chapters[0]).unwrap();
    println!("pages: {}", pages.len());
    assert!(!pages.is_empty());
}
