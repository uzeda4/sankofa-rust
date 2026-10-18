//! Luscious against API answers shaped like the real ones (the live API was checked by hand for the shapes).

use super::*;

fn query_of(url: &str, key: &str) -> String {
    Url::parse(url).unwrap().query_pairs().find(|(k, _)| k == key).map(|(_, v)| v.into_owned()).unwrap_or_default()
}

fn variables(url: &str) -> Value {
    serde_json::from_str(&query_of(url, "variables")).unwrap()
}

fn mock(f: impl Fn(&str) -> Option<String> + 'static) {
    set_mock_http(move |r| match f(&r.url) {
        Some(body) => Ok(HttpResponse { status: 200, body, url: r.url.clone(), ..Default::default() }),
        None => Ok(HttpResponse { status: 404, url: r.url.clone(), ..Default::default() }),
    });
}

fn album_json(pictures: u32) -> String {
    format!(
        r#"{{"data":{{"album":{{"get":{{"url":"/albums/demo_55/","title":"Demo","cover":{{"url":"https://c.test/c.jpg","width":1,"height":1}},"description":"About","language":{{"title":"English"}},
        "labels":["hot"],"genres":[{{"title":"Humor"}}],"audiences":[{{"title":"Straight Sex"}}],"tags":[{{"text":"Artist: Some Body"}},{{"text":"Tag"}}],
        "number_of_pictures":{pictures},"number_of_animated_pictures":2,"content":{{"title":"Hentai"}},"created":1700000000.5}}}}}}}}"#
    )
}

fn pictures_json(first: u32, count: u32) -> String {
    let items: Vec<String> = (first..first + count)
        .map(|p| {
            format!(
                r#"{{"created":1700000001.2,"title":"t{p}","url_to_original":null,"url_to_video":null,"position":{p},"thumbnails":[{{"width":10,"height":10,"url":"https://other.test/a/{p}.small.jpg"}},{{"width":100,"height":200,"url":"//other.test/a/{p}.jpg"}}]}}"#
            )
        })
        .collect();
    format!(r#"{{"data":{{"picture":{{"list":{{"info":{{"has_next_page":true}},"items":[{}]}}}}}}}}"#, items.join(","))
}

#[test]
fn the_portuguese_source_keeps_the_fixed_id_and_the_others_derive_theirs() {
    set_context("pt-BR", &[]);
    assert_eq!(Luscious.info().id, 5_826_725_746_643_311_801);
    set_context("en", &[]);
    let info = Luscious.info();
    assert_eq!((info.id, info.version_id), (0, 1));
    assert_eq!(info.mirrors.len(), 2);
}

#[test]
fn default_lists_send_the_extensions_filters() {
    set_context("en", &[]);
    mock(|u| {
        assert!(u.starts_with("https://www.luscious.net/graphql/nobatch/?operationName=AlbumList&query="), "{u}");
        let v = variables(u);
        assert_eq!(v["input"]["display"], "rating_all_time");
        assert_eq!(v["input"]["page"], 3);
        assert_eq!(
            v["input"]["filters"],
            json!([{"name": "selection", "value": "all"}, {"name": "audience_ids", "value": "+1+10+2+3+5+6+8+9"}, {"name": "language_ids", "value": "+1"}])
        );
        Some(r#"{"data":{"album":{"list":{"info":{"page":3,"has_next_page":true},"items":[{"url":"/albums/a_1/","title":"A","cover":{"url":"https://c.test/1.jpg"}}]}}}}"#.into())
    });
    let page = Luscious.popular(3).unwrap();
    assert!(page.has_next);
    assert_eq!((page.mangas[0].url.as_str(), page.mangas[0].title.as_str()), ("/albums/a_1/", "A"));
    // the "all" language sends no language filter, and latest sorts by date
    set_context("all", &[]);
    mock(|u| {
        let v = variables(u);
        assert_eq!(v["input"]["display"], "date_newest");
        assert!(!v["input"]["filters"].to_string().contains("language_ids"));
        Some(r#"{"data":{"album":{"list":{"info":{"has_next_page":false},"items":[]}}}}"#.into())
    });
    assert!(!Luscious.latest(1).unwrap().has_next);
}

#[test]
fn search_turns_filters_into_the_sites_syntax() {
    set_context("en", &[]);
    mock(|u| {
        let v = variables(u);
        let f = v["input"]["filters"].to_string();
        for part in [
            r#"{"name":"album_type","value":"manga"}"#,
            r#"{"name":"audience_ids","value":"+1+2"}"#,
            r#"{"name":"language_ids","value":"+1+2+3"}"#,
            r#"{"name":"tagged","value":"+dark_fantasy-yuri"}"#,
            r#"{"name":"genre_ids","value":"+25+45-27"}"#,
            r#"{"name":"search_query","value":"hello"}"#,
        ] {
            assert!(f.contains(part), "{part} missing from {f}");
        }
        assert_eq!(v["input"]["display"], "date_newest");
        Some(r#"{"data":{"album":{"list":{"info":{"has_next_page":false},"items":[]}}}}"#.into())
    });
    let state: Filters = serde_json::from_str(
        r#"{"sort":6,"album_type":1,"interests":{"1":true,"10":false,"2":true},"languages":{"2":true,"3":true},"tags":"Dark Fantasy, -Yuri","genres":{"25":1,"45":1,"27":2,"45b":0}}"#,
    )
    .unwrap();
    Luscious.search(1, "hello", &state).unwrap();

    let none: Filters = serde_json::from_str(r#"{"interests":{"1":false}}"#).unwrap();
    assert_eq!(Luscious.search(1, "", &none).unwrap_err(), "Please select an Interest");
    assert_eq!(tag_filter("a b,c d"), "+a_b+c_d");
}

#[test]
fn a_tankobon_genre_sends_the_shared_id() {
    set_context("en", &[]);
    let state: Filters = serde_json::from_str(r#"{"genres":{"45b":1}}"#).unwrap();
    let sent = album_list_filters("en", &state, "").unwrap();
    assert!(sent.contains(&("genre_ids".to_string(), "+45".to_string())), "{sent:?}");
}

#[test]
fn details_and_picture_chapters_page_through_the_album() {
    set_context("en", &[]);
    mock(|u| match query_of(u, "operationName").as_str() {
        "AlbumGet" => {
            assert_eq!(variables(u)["id"], "55");
            Some(album_json(70))
        }
        "AlbumListOwnPictures" => {
            let v = variables(u);
            assert_eq!(v["input"]["display"], "position");
            match v["input"]["page"].as_u64() {
                Some(1) => Some(pictures_json(1, 50)),
                Some(2) => Some(pictures_json(51, 20)),
                _ => None,
            }
        }
        _ => None,
    });
    let stub = Manga { url: "/albums/demo_55/".into(), ..Default::default() };
    let up = Luscious.update(&stub, true, true).unwrap();
    let m = up.manga.unwrap();
    assert_eq!(m.url, "/albums/demo_55/");
    assert_eq!((m.artist.as_deref(), m.author.as_deref()), (Some("Some Body"), Some("Some Body")));
    assert_eq!(m.genre, ["English", "hot", "Humor", "Straight Sex", "Artist: Some Body", "Tag", "Hentai"]);
    assert_eq!(m.description.as_deref(), Some("About\n\nPictures: 70\nAnimated Pictures: 2"));
    let ch = up.chapters.unwrap();
    assert_eq!(ch.len(), 70);
    assert_eq!((ch[0].name.as_str(), ch[0].url.as_str()), ("70 - t70", "/a/70.jpg"));
    assert_eq!(ch[69].name, "1 - t1");
    assert_eq!(ch[0].date_upload, 1_700_000_001_000);
    assert_eq!(Luscious.chapter_url(&ch[0]).as_deref(), Some("https://ah-img.luscious.net/a/70.jpg"));
    let page = Luscious.pages(&ch[0]).unwrap();
    assert_eq!(page[0].image_url, "https://ah-img.luscious.net/a/70.jpg");
}

#[test]
fn merged_chapters_are_parts_of_a_thousand_and_pages_read_the_chunk() {
    set_context("en", &[(MERGE_CHAPTERS, json!(true)), (RESOLUTION, json!("1"))]);
    mock(|u| match query_of(u, "operationName").as_str() {
        "AlbumGet" => Some(album_json(2500)),
        "AlbumListOwnPictures" => {
            // chunk 2 starts at API page 21
            let page = variables(u)["input"]["page"].as_u64().unwrap();
            match page {
                21 => Some(pictures_json(1001, 50)),
                22 => Some(pictures_json(1051, 3)),
                _ => Some(pictures_json(0, 0)),
            }
        }
        _ => None,
    });
    let stub = Manga { url: "/albums/demo_55/".into(), ..Default::default() };
    let ch = Luscious.chapters(&stub).unwrap();
    assert_eq!(ch.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(), ["Merged Chapter (Part 3)", "Merged Chapter (Part 2)", "Merged Chapter (Part 1)"]);
    assert_eq!(ch[1].url, "/albums/demo_55/?chunk=2");
    assert_eq!(Luscious.chapter_url(&ch[1]).as_deref(), Some("https://www.luscious.net/albums/demo_55/"));
    let pages = Luscious.pages(&ch[1]).unwrap();
    assert_eq!(pages.len(), 53, "a short page ends the chunk");
    // resolution "1" picks the second thumbnail, made protocol-absolute, on the image host
    assert_eq!(pages[0].image_url, "https://ah-img.luscious.net/a/1001.jpg");
}

#[test]
fn related_albums_come_from_the_three_lists() {
    set_context("en", &[]);
    mock(|u| {
        assert_eq!(variables(u)["id"], "55");
        Some(r#"{"data":{"album":{"list_related":{"more_like_this":[{"url":"/albums/a_1/","title":"A","cover":{"url":"x"}}],"items_liked_like_this":null,"items_created_by_this_user":[{"url":"/albums/b_2/","title":"B","cover":{"url":"y"}}]}}}}"#.into())
    });
    let related = Luscious.related(&Manga { url: "/albums/demo_55/".into(), ..Default::default() }).unwrap();
    assert_eq!(related.iter().map(|m| m.title.as_str()).collect::<Vec<_>>(), ["A", "B"]);
}

#[test]
fn links_to_other_hosts_or_pages_are_not_albums() {
    set_context("en", &[]);
    assert!(Luscious.by_url("https://example.com/albums/x_1/").unwrap().is_none());
    assert!(Luscious.by_url("https://www.luscious.net/users/x/").unwrap().is_none());
    assert_eq!(album_id("/albums/demo_55/?chunk=2"), "55");
}

/// Needs network: `cargo test -p sk-source-luscious live -- --ignored --nocapture`. Runs the whole flow on the real API.
#[test]
#[ignore]
fn live_api_flow() {
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
    let page = Luscious.popular(1).unwrap();
    println!("popular: {} albums, next {}", page.mangas.len(), page.has_next);
    assert!(!page.mangas.is_empty());
    let up = Luscious.update(&page.mangas[0], true, true).unwrap();
    let chapters = up.chapters.unwrap();
    println!("details: genres {}, chapters {}", up.manga.unwrap().genre.len(), chapters.len());
    assert!(!chapters.is_empty());
    println!("pages: {}", Luscious.pages(&chapters[0]).unwrap().len());
    println!("related: {}", Luscious.related(&page.mangas[0]).unwrap().len());
    assert!(!Luscious.search(1, "love", &Filters::default()).unwrap().mangas.is_empty());
}

#[test]
fn a_refusal_from_the_site_is_shown_as_the_sites_message() {
    set_context("all", &[]);
    set_mock_http(|r| {
        Ok(HttpResponse {
            status: 400,
            body: r#"{"errors":[{"message":"Invalid Facet Value: 6 for facet: Content Type. Reason: This item cannot be viewed in your location. [Error BK-03]"}]}"#.into(),
            url: r.url.clone(),
            ..Default::default()
        })
    });
    let err = Luscious.popular(1).err().unwrap();
    assert!(err.contains("cannot be viewed in your location") && !err.contains("https://"), "{err}");
}
