//! Webtoons.com, written in Rust. Port of the Kotlin extension `all.webtoons` (one source per language: en, id, th, es,
//! fr, zh-Hant, de). Lists and details are HTML pages of the desktop site, the episode list comes from the mobile
//! site's JSON API, and the pages are the images of the viewer (plus, optionally, the author's note drawn as a page).

use fancy_regex::Regex;
use serde::Deserialize;
use sk_source_sdk::html::Document;
use sk_source_sdk::*;

const BASE: &str = "https://www.webtoons.com";
const MOBILE: &str = "https://m.webtoons.com";

const SHOW_NOTES: &str = "showAuthorsNotes";
const MAX_QUALITY: &str = "useMaxQuality";
const SEQUENTIAL: &str = "useSequentialNumbering";

const SEARCH_TYPES: [(&str, &str); 3] = [("ALL", ""), ("Originals", "originals"), ("Canvas", "canvas")];

/// `zh-Hant` is spelled `zh-hant` in the site's addresses.
fn lang_code() -> String {
    match lang().as_str() {
        "zh-Hant" => "zh-hant".into(),
        l => l.into(),
    }
}

#[derive(Default)]
pub struct Webtoons;

impl Source for Webtoons {
    fn info(&self) -> Info {
        let locale = if lang() == "zh-Hant" { "zh_TW".to_string() } else { lang() };
        Info {
            name: "Webtoons.com".into(),
            lang: lang(),
            base_url: BASE.into(),
            // Two languages keep ids from before their names changed.
            id: match lang().as_str() {
                "id" => 8_749_627_068_478_740_298,
                "zh-Hant" => 2_959_982_438_613_576_472,
                _ => 0,
            },
            supports_latest: true,
            rate_limit: Some(RateLimit { permits: 1, period_ms: 1000, host: "m.webtoons.com".into(), skip_path: String::new() }),
            cookies: vec![
                CookieDef::new("webtoons.com", "ageGatePass", "true"),
                CookieDef::new("webtoons.com", "locale", &locale),
                CookieDef::new("webtoons.com", "needGDPR", "false"),
            ],
            prefs: vec![
                PrefDef::switch(SHOW_NOTES, "Show author's notes", "Enable to see the author's notes at the end of chapters (if they're there).", false),
                PrefDef::switch(MAX_QUALITY, "Use maximum quality images", "Enable to load images in maximum quality.", false),
                PrefDef::switch(SEQUENTIAL, "Use sequential chapter numbering", "Enable to use sequential numbering instead of official episode numbers.", false),
            ],
            ..Default::default()
        }
    }

    fn popular(&self, page: u32) -> Result<MangaPage> {
        let ranking = match page {
            1 => "trending",
            2 => "popular",
            3 => "originals",
            4 => "canvas",
            _ => return Err("page > 4 not available".into()),
        };
        let doc = get(&format!("{BASE}/{}/ranking/{ranking}", lang_code())).send()?.html();
        Ok(MangaPage { mangas: parse_list(&doc), has_next: ranking != "canvas" })
    }

    fn latest(&self, _page: u32) -> Result<MangaPage> {
        let day = ["thursday", "friday", "saturday", "sunday", "monday", "tuesday", "wednesday"][(now_ms().div_euclid(86_400_000)).rem_euclid(7) as usize];
        let doc = get(&format!("{BASE}/{}/originals/{day}?sortOrder=UPDATE", lang_code())).send()?.html();
        Ok(MangaPage { mangas: parse_list(&doc), has_next: false })
    }

    fn search(&self, page: u32, query: &str, filters: &Filters) -> Result<MangaPage> {
        let kind = filters.select_value("type", &SEARCH_TYPES);
        let mut url = format!("{BASE}/{}/search", lang_code());
        if !kind.is_empty() {
            url.push_str(&format!("/{kind}"));
        }
        url.push_str(&format!("?keyword={}", url_encode(query)));
        if page > 1 && !kind.is_empty() {
            url.push_str(&format!("&page={page}"));
        }
        let doc = get(&url).send()?.html();
        Ok(MangaPage { mangas: parse_list(&doc), has_next: doc.select_first("a.pagination[aria-current=true] + a").is_some() })
    }

    fn filters(&self, _data: Option<&Value>) -> Vec<FilterDef> {
        vec![FilterDef::select("type", "Search Type", &SEARCH_TYPES, 0)]
    }

    fn by_url(&self, url: &str) -> Result<Option<Manga>> {
        let u = Url::parse(url).map_err(|e| e.to_string())?;
        if u.host_str() != Url::parse(BASE).ok().and_then(|b| b.host_str().map(str::to_string)).as_deref() {
            return Ok(None);
        }
        let Some(title_no) = u.query_pairs().find(|(k, _)| k == "title_no").and_then(|(_, v)| v.parse::<u32>().ok()) else { return Ok(None) };
        let path: Vec<&str> = u.path_segments().map(|s| s.collect()).unwrap_or_default();
        // Every language ships as its own source; only the matching one resolves the link.
        if path.len() < 3 || path[0] != lang_code() {
            return Ok(None);
        }
        let prefix = if path[1] == "canvas" { "/challenge" } else { "" };
        let stub = Manga { url: format!("{prefix}/episodeList?titleNo={title_no}"), ..Default::default() };
        self.update(&stub, true, false)?.manga.map(Some).ok_or_else(|| "no details".into())
    }

    fn update(&self, manga: &Manga, fetch_details: bool, fetch_chapters: bool) -> Result<Update> {
        // Details and the episode list live on different hosts, so ask for both together.
        let mut requests = Vec::new();
        if fetch_details {
            requests.push(get(&absolute(BASE, &manga.url)));
        }
        if fetch_chapters {
            requests.push(episodes_request(manga)?);
        }
        let mut answers = send_all(requests).into_iter();
        let mut update = Update::default();
        if fetch_details {
            let resp = answers.next().ok_or("no answer")??;
            update.manga = Some(parse_details(&resp.html(), manga)?);
        }
        if fetch_chapters {
            update.chapters = Some(parse_episodes(answers.next().ok_or("no answer")??)?);
        }
        Ok(update)
    }

    fn details(&self, manga: &Manga) -> Result<Manga> {
        self.update(manga, true, false)?.manga.ok_or_else(|| "no details".into())
    }

    fn chapters(&self, manga: &Manga) -> Result<Vec<Chapter>> {
        self.update(manga, false, true)?.chapters.ok_or_else(|| "no chapters".into())
    }

    fn pages(&self, chapter: &Chapter) -> Result<Vec<Page>> {
        let resp = get(&absolute(BASE, &chapter.url)).send()?;
        let doc = resp.html();
        let max_quality = pref_bool(MAX_QUALITY, false);
        let mut pages: Vec<Page> = doc
            .select("div#_imageList > img")
            .iter()
            .enumerate()
            .filter_map(|(i, img)| {
                let mut url = Url::parse(&img.attr("data-url")).ok()?;
                if max_quality && url.query_pairs().any(|(k, v)| k == "type" && v == "q90") {
                    let kept: Vec<(String, String)> = url.query_pairs().filter(|(k, _)| k != "type").map(|(k, v)| (k.into_owned(), v.into_owned())).collect();
                    url.set_query(None);
                    if !kept.is_empty() {
                        url.query_pairs_mut().extend_pairs(kept);
                    }
                }
                Some(Page::new(i as u32, url.to_string()))
            })
            .collect();

        if pages.is_empty() {
            pages = motion_toon_pages(&resp.body)?;
        }

        if pref_bool(SHOW_NOTES, false) {
            let note = doc.select("div.creator_note p.author_text").text();
            if !note.is_empty() {
                let creator = doc.select("div.creator_note .author_name span").text().trim().to_string();
                let mut page = Page::new(pages.len() as u32, format!("{BASE}/author-note"));
                page.ops = vec![ImageOp::Text { title: format!("Author's Notes from {creator}"), body: note }];
                pages.push(page);
            }
        }
        Ok(pages)
    }
}

fn parse_list(doc: &Document) -> Vec<Manga> {
    doc.select(".webtoon_list li a")
        .iter()
        .filter_map(|a| {
            Some(Manga {
                url: path_of(&a.abs_url("href")),
                title: a.select_first(".title")?.text(),
                thumbnail_url: a.select_first("img").map(|i| i.abs_url("src")).filter(|s| !s.is_empty()),
                ..Default::default()
            })
        })
        .collect()
}

fn parse_details(doc: &Document, previous: &Manga) -> Result<Manga> {
    let header = doc.select_first(".detail_header .info");
    let aside = doc.select_first("#_asideDetail");
    let author = header
        .as_ref()
        .and_then(|h| h.select_first(".author:nth-of-type(1)").map(|a| a.own_text()))
        .or_else(|| header.as_ref().and_then(|h| h.select_first(".author_area").map(|a| a.own_text())));
    let artist = header
        .as_ref()
        .and_then(|h| h.select_first(".author:nth-of-type(2)").map(|a| a.own_text()))
        .or_else(|| header.as_ref().and_then(|h| h.select_first(".author_area").map(|a| a.own_text())))
        .or_else(|| author.clone());
    let day_info = aside.as_ref().and_then(|a| a.select_first("p.day_info").map(|d| d.text())).unwrap_or_default();
    let status = if ["UP", "EVERY", "NOUVEAU"].iter().any(|w| day_info.contains(w)) {
        1
    } else if ["END", "COMPLETED", "TERMINÉ"].iter().any(|w| day_info.contains(w)) {
        2
    } else {
        0
    };

    // A title kept in a library keeps its cover unless the site's was changed; the banner is not a cover.
    let last = |u: &str| Url::parse(u).ok().and_then(|u| u.path_segments().and_then(|s| s.last().map(str::to_string)));
    let banner = doc.select_first(".detail_header .thmb img").and_then(|i| last(&i.abs_url("src")));
    let old = previous.thumbnail_url.as_deref().and_then(last);
    let og = doc.select_first("head meta[property=\"og:image\"]").map(|m| m.attr("content"));
    let thumbnail_url = match old {
        Some(o) if Some(&o) != banner.as_ref() => previous.thumbnail_url.clone(),
        _ => og,
    };

    Ok(Manga {
        url: path_of(doc.location()),
        title: doc.select_first("h1.subj, h3.subj").ok_or("the page has no title (the site changed?)")?.text(),
        author,
        artist,
        genre: header.as_ref().map(|h| h.select(".genre").each_text()).unwrap_or_default(),
        description: aside.as_ref().and_then(|a| a.select_first("p.summary").map(|s| s.text())),
        status,
        thumbnail_url,
        initialized: true,
        ..Default::default()
    })
}

// ------------------------------------------------------------------ episodes

#[derive(Deserialize)]
struct EpisodesResponse {
    result: EpisodeList,
}

#[derive(Deserialize)]
struct EpisodeList {
    #[serde(rename = "episodeList")]
    episode_list: Vec<Episode>,
}

#[derive(Deserialize)]
struct Episode {
    #[serde(rename = "episodeTitle")]
    title: String,
    #[serde(rename = "viewerLink")]
    viewer_link: String,
    #[serde(rename = "exposureDateMillis")]
    date_millis: i64,
    #[serde(rename = "hasBgm", default)]
    has_bgm: bool,
}

fn episodes_request(manga: &Manga) -> Result<Request> {
    let url = Url::parse(&absolute(BASE, &manga.url)).map_err(|e| e.to_string())?;
    let title_id = url
        .query_pairs()
        .find(|(k, _)| k == "title_no" || k == "titleNo")
        .map(|(_, v)| v.into_owned())
        .ok_or_else(|| "Migrate from Webtoons.com to Webtoons.com".to_string())?;
    let path: Vec<&str> = url.path_segments().map(|s| s.filter(|p| !p.is_empty()).collect()).unwrap_or_default();
    let kind = if url.path().contains("episodeList") {
        // the older address forms people still have in their library
        match path.first() {
            Some(&"episodeList") => "webtoon",
            Some(&"challenge") => "canvas",
            _ => return Err("Migrate from Webtoons.com to Webtoons.com".into()),
        }
    } else if path.get(1) == Some(&"canvas") {
        "canvas"
    } else {
        "webtoon"
    };
    let mut api = format!("{MOBILE}/api/v1/{kind}/{title_id}/episodes?pageSize=99999");
    if kind == "canvas" {
        api.push_str(&format!("&readingLanguageCode={}", lang_code()));
    }
    Ok(get(&api).header("Referer", &format!("{MOBILE}/")))
}

/// season (group 4), bonus/mini/special (6), episode number (11)
fn episode_regex() -> Regex {
    Regex::new(
        r"(?i)(?:(s(eason)?|saison|part|vol(ume)?)\s*\.?\s*(\d+).*?)?(.*?(mini|bonus|special).*?)?(e(p(isode)?)?|ch(apter)?)\s*\.?\s*(\d+(\.\d+)?)",
    )
    .expect("a valid pattern")
}

struct Numbered {
    ep: Episode,
    number: f32,
    season: i32,
}

fn parse_episodes(resp: Response) -> Result<Vec<Chapter>> {
    let list: EpisodesResponse = resp.json()?;
    let re = episode_regex();
    let (mut recognized, mut unrecognized) = (0, 0);
    let mut chapters: Vec<Numbered> = list
        .result
        .episode_list
        .into_iter()
        .map(|ep| {
            let caps = re.captures(ep.title.as_str()).ok().flatten();
            // bonus, mini and special episodes get no number of their own
            let matched = caps.as_ref().filter(|c| c.get(6).is_none_or(|m| m.as_str().is_empty()));
            let number = matched.and_then(|c| c.get(11)).and_then(|m| m.as_str().parse::<f32>().ok()).unwrap_or(-1.0);
            let season = matched.and_then(|c| c.get(4)).and_then(|m| m.as_str().parse::<i32>().ok()).unwrap_or(1);
            if number == -1.0 {
                unrecognized += 1;
            } else {
                recognized += 1;
            }
            Numbered { ep, number, season }
        })
        .collect();

    if pref_bool(SEQUENTIAL, false) || unrecognized > recognized {
        for (i, c) in chapters.iter_mut().enumerate() {
            c.number = (i + 1) as f32;
        }
    } else {
        let (mut max_number, mut current_season, mut season_offset) = (0f32, 1, 0f32);
        for i in 0..chapters.len() {
            if chapters[i].number != -1.0 {
                let original = chapters[i].number;
                // a new season restarts the count: keep numbers rising
                if chapters[i].season > current_season {
                    current_season = chapters[i].season;
                    if original <= max_number {
                        season_offset = max_number;
                    }
                }
                chapters[i].number = season_offset + original;
                max_number = max_number.max(chapters[i].number);
            } else {
                chapters[i].number = if i == 0 { 0.0 } else { chapters[i - 1].number + 0.01 };
            }
        }
    }

    Ok(chapters
        .into_iter()
        .map(|c| {
            let mut name = unescape(&c.ep.title);
            name.push_str(&format!(" (ch. {})", format_number(c.number)));
            if c.ep.has_bgm {
                name.push_str(" ♫");
            }
            Chapter { url: c.ep.viewer_link, name, date_upload: c.ep.date_millis, chapter_number: c.number, ..Default::default() }
        })
        .rev()
        .collect())
}

/// `DecimalFormat("#.##")`: at most two decimals, no trailing zeros.
fn format_number(n: f32) -> String {
    let s = format!("{:.2}", n);
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// HTML entities in a title (`&amp;`, `&#39;`) to the characters they stand for.
fn unescape(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    Document::parse(&format!("<p>{s}</p>"), "").select_first("p").map(|p| p.whole_text()).unwrap_or_else(|| s.to_string())
}

// ------------------------------------------------------------------ pages

/// A motion toon has no images in the page: its layers are listed in a JSON file the page points to.
fn motion_toon_pages(html: &str) -> Result<Vec<Page>> {
    let doc_url = Regex::new(r"documentURL:.*?'(.*?)'").unwrap().captures(html).ok().flatten().map(|c| c[1].to_string());
    let path = Regex::new(r"jpg:.*?'(.*?)\{").unwrap().captures(html).ok().flatten().map(|c| c[1].to_string());
    let (Some(doc_url), Some(path)) = (doc_url, path) else { return Ok(Vec::new()) };

    #[derive(Deserialize)]
    struct Motion {
        assets: Assets,
    }
    #[derive(Deserialize)]
    struct Assets {
        images: std::collections::BTreeMap<String, String>,
    }
    let motion: Motion = get(&doc_url).send()?.json()?;
    Ok(motion
        .assets
        .images
        .iter()
        .filter(|(k, _)| k.contains("layer"))
        .enumerate()
        .map(|(i, (_, v))| Page::new(i as u32, format!("{path}{v}")))
        .collect())
}

export_source!(Webtoons);

#[cfg(test)]
mod tests;
