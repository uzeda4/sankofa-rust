//! Luscious search filters: the option tables of the Kotlin extension and the album-list request they turn into.

use sk_source_sdk::*;

/// A value that means "do not send this filter".
pub const IGNORE: &str = "<ignore>";

pub const POPULAR_SORT: &str = "rating_all_time";
pub const LATEST_SORT: &str = "date_newest";

/// The site's number for a source language; `all` sends no language filter.
pub fn site_lang(lang: &str) -> &'static str {
    match lang {
        "all" => IGNORE,
        "en" => "1",
        "ja" => "2",
        "es" => "3",
        "it" => "4",
        "de" => "5",
        "fr" => "6",
        "zh" => "8",
        "ko" => "9",
        "pt-BR" => "100",
        "th" => "101",
        _ => "99",
    }
}

pub const ALBUM_TYPE: [(&str, &str); 3] = [("All", IGNORE), ("Manga", "manga"), ("Pictures", "pictures")];
pub const RESTRICT_GENRES: [(&str, &str); 3] = [("None", IGNORE), ("Loose", "loose"), ("Strict", "strict")];
pub const SELECTION: [(&str, &str); 7] = [
    ("All", "all"),
    ("No Votes", "not_voted"),
    ("Downvoted", "downvoted"),
    ("Animated", "animated"),
    ("Banned", "banned"),
    ("Made by People You Follow", "made_by_following"),
    ("Faved by People You Follow", "faved_by_following"),
];
pub const CONTENT_TYPE: [(&str, &str); 4] = [("All", IGNORE), ("Hentai", "2"), ("Non-Erotic", "5"), ("Real People", "6")];
pub const ALBUM_SIZE: [(&str, &str); 8] = [
    ("All", IGNORE),
    ("0-25", "0"),
    ("0-50", "1"),
    ("50-100", "2"),
    ("100-200", "3"),
    ("200-800", "4"),
    ("800-3200", "5"),
    ("3200-12800", "6"),
];

/// `(label, id)`; every one is ticked by default.
pub const INTERESTS: [(&str, &str); 8] = [
    ("Straight Sex", "1"),
    ("Trans x Girl", "10"),
    ("Gay / Yaoi", "2"),
    ("Lesbian / Yuri", "3"),
    ("Trans", "5"),
    ("Solo Girl", "6"),
    ("Trans x Trans", "8"),
    ("Trans x Guy", "9"),
];

pub const LANGUAGES: [(&str, &str); 11] = [
    ("English", "1"),
    ("Japanese", "2"),
    ("Spanish", "3"),
    ("Italian", "4"),
    ("German", "5"),
    ("French", "6"),
    ("Chinese", "8"),
    ("Korean", "9"),
    ("Others", "99"),
    ("Portuguese", "100"),
    ("Thai", "101"),
];

/// `(label, key)`. The key is the site's genre id; "Tankōbon" shares id 45 with "Music" on the site, so its key carries a
/// `b` that [`genre_id`] drops.
pub const GENRES: &[(&str, &str)] = &[
    ("3D / Digital Art", "25"),
    ("Amateurs", "20"),
    ("Artist Collection", "19"),
    ("Asian Girls", "12"),
    ("BDSM", "27"),
    ("Bestiality Hentai", "5"),
    ("Casting", "44"),
    ("Celebrity Fakes", "16"),
    ("Cosplay", "22"),
    ("Cross-Dressing", "30"),
    ("Cumshot", "26"),
    ("Defloration / First Time", "59"),
    ("Ebony Girls", "32"),
    ("European Girls", "46"),
    ("Extreme Gore", "60"),
    ("Extreme Scat", "61"),
    ("Fantasy / Monster Girls", "10"),
    ("Fetish", "2"),
    ("Furries", "8"),
    ("Futanari", "31"),
    ("Group Sex", "36"),
    ("Harem", "56"),
    ("Humor", "41"),
    ("Incest", "24"),
    ("Interracial", "28"),
    ("Kemonomimi / Animal Ears", "39"),
    ("Latina Girls", "33"),
    ("Lolicon", "3"),
    ("Mature", "13"),
    ("Members: Original Art", "18"),
    ("Members: Verified Selfies", "21"),
    ("Military", "48"),
    ("Mind Control", "34"),
    ("Monsters & Tentacles", "38"),
    ("Music", "45"),
    ("Netorare / Cheating", "40"),
    ("No Genre Given", "1"),
    ("Nonconsent / Reluctance", "37"),
    ("Other Ethnicity Girls", "57"),
    ("Private to Luscious", "55"),
    ("Public Sex", "43"),
    ("Romance", "42"),
    ("School / College", "35"),
    ("Sex Workers", "47"),
    ("SFW", "23"),
    ("Shotacon", "4"),
    ("Softcore / Ecchi", "9"),
    ("Superheroes", "17"),
    ("Swimsuit", "49"),
    ("Tankōbon", "45b"),
    ("Trans", "14"),
    ("TV / Movies", "51"),
    ("Video Games", "15"),
    ("Vintage", "58"),
    ("Western", "11"),
    ("Workplace Sex", "50"),
];

fn genre_id(key: &str) -> &str {
    key.strip_suffix('b').unwrap_or(key)
}

/// The sort options: ratings, dates (every year since 2013), then first letters. `(label, value)`.
pub fn sort_options(current_year: i64) -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = [
        ("Rating - All Time", "rating_all_time"),
        ("Rating - Last 7 Days", "rating_7_days"),
        ("Rating - Last 14 Days", "rating_14_days"),
        ("Rating - Last 30 Days", "rating_30_days"),
        ("Rating - Last 90 Days", "rating_90_days"),
        ("Rating - Last Year", "rating_1_year"),
        ("Date - Newest First", "date_newest"),
        ("Date - Oldest First", "date_oldest"),
        ("Date - Upcoming", "date_upcoming"),
        ("Date - Trending", "date_trending"),
        ("Date - Featured", "date_featured"),
        ("Date - Last Viewed", "date_last_interaction"),
        ("Other - Search Score", "search_score"),
    ]
    .iter()
    .map(|(l, v)| (l.to_string(), v.to_string()))
    .collect();
    for year in (2013..=current_year).rev() {
        v.push((format!("Date - {year}"), format!("date_{year}")));
    }
    v.push(("First Letter - Any".into(), "alpha_any".into()));
    for c in 'a'..='z' {
        v.push((format!("First Letter - {}", c.to_ascii_uppercase()), format!("alpha_{c}")));
    }
    v
}

pub fn current_year() -> i64 {
    civil_from_days(now_ms().div_euclid(86_400_000)).0
}

fn pairs(a: &[(&str, &str)]) -> Vec<(String, String)> {
    a.iter().map(|(l, v)| (l.to_string(), v.to_string())).collect()
}

fn checkbox_group(key: &str, name: &str, items: &[(&str, &str)], default: bool) -> FilterDef {
    FilterDef::Group {
        key: key.into(),
        name: name.into(),
        items: items.iter().map(|(label, k)| FilterDef::Checkbox { key: k.to_string(), name: label.to_string(), default }).collect(),
    }
}

pub fn filter_defs(lang: &str) -> Vec<FilterDef> {
    let sort = sort_options(current_year());
    let sort_pairs: Vec<(&str, &str)> = sort.iter().map(|(l, v)| (l.as_str(), v.as_str())).collect();
    let own = site_lang(lang);
    let languages: Vec<(&str, &str)> = LANGUAGES.iter().copied().filter(|(_, v)| *v != own).collect();
    vec![
        FilterDef::select("sort", "Sort By", &sort_pairs, 0),
        FilterDef::select("album_type", "Album Type", &ALBUM_TYPE, 0),
        FilterDef::select("content_type", "Content Type", &CONTENT_TYPE, 0),
        FilterDef::select("album_size", "Album Size", &ALBUM_SIZE, 0),
        FilterDef::select("selection", "Selection", &SELECTION, 0),
        FilterDef::select("restrict_genres", "Restrict Genres", &RESTRICT_GENRES, 0),
        checkbox_group("interests", "Interests", &INTERESTS, true),
        checkbox_group("languages", "Languages", &languages, false),
        FilterDef::tristates("genres", "Genres", &pairs(GENRES)),
        FilterDef::header("Separate tags with commas (,)"),
        FilterDef::header("Prepend with dash (-) to exclude"),
        FilterDef::text("tags", "Tags"),
        FilterDef::header("The following require username or ID"),
        FilterDef::text("creator", "Uploader"),
        FilterDef::text("favorite", "Favorite by User"),
    ]
}

/// `+id` for each ticked option (in the table's order), as the site wants them.
fn plus_list(filters: &Filters, group: &str, options: &[(&str, &str)], default_all: bool) -> String {
    let all_on = default_all && !filters.0.contains_key(group);
    let state = filters.group_state(group);
    options.iter().filter(|(_, k)| all_on || state.tri(k) == Tri::Include).map(|(_, k)| format!("+{k}")).collect()
}

/// "dark fantasy, -yuri" -> "+dark_fantasy-yuri": the site's tag syntax.
pub fn tag_filter(text: &str) -> String {
    format!("+{}", text.to_lowercase())
        .replace(' ', "_")
        .replace("_,", "+")
        .replace(",_", "+")
        .replace(',', "+")
        .replace("+-", "-")
        .replace("-_", "-")
        .trim()
        .to_string()
}

/// The `filters` array of an `AlbumList` request, in the order the extension sent them. `Err` when no interest is ticked.
pub fn album_list_filters(lang: &str, filters: &Filters, query: &str) -> Result<Vec<(String, String)>> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut put = |name: &str, value: String| out.push((name.to_string(), value));
    for (key, name, options) in [
        ("content_type", "content_id", &CONTENT_TYPE[..]),
        ("album_type", "album_type", &ALBUM_TYPE[..]),
        ("selection", "selection", &SELECTION[..]),
        ("album_size", "picture_count_rank", &ALBUM_SIZE[..]),
        ("restrict_genres", "restrict_genres", &RESTRICT_GENRES[..]),
    ] {
        let value = filters.select_value(key, options);
        if value != IGNORE {
            put(name, value.to_string());
        }
    }

    let interests = plus_list(filters, "interests", &INTERESTS, true);
    if interests.is_empty() {
        return Err("Please select an Interest".into());
    }
    put("audience_ids", interests);

    if site_lang(lang) != IGNORE {
        // the source's own language, then the other languages ticked ("+1" or "+1+2+3"); a bare "+" lists every language
        let ticked = plus_list(filters, "languages", &LANGUAGES, false);
        put("language_ids", format!("+{}{ticked}", site_lang(lang)));
    }

    let tags = filters.text("tags");
    if !tags.is_empty() {
        put("tagged", tag_filter(tags));
    }
    if !filters.text("creator").is_empty() {
        put("created_by_id", filters.text("creator").to_string());
    }
    if !filters.text("favorite").is_empty() {
        put("favorite_by_user_id", filters.text("favorite").to_string());
    }

    let genres = filters.group_state("genres");
    let included: String = GENRES.iter().filter(|(_, k)| genres.tri(k) == Tri::Include).map(|(_, k)| format!("+{}", genre_id(k))).collect();
    let excluded: String = GENRES.iter().filter(|(_, k)| genres.tri(k) == Tri::Exclude).map(|(_, k)| format!("-{}", genre_id(k))).collect();
    if !included.is_empty() || !excluded.is_empty() {
        put("genre_ids", format!("{included}{excluded}"));
    }

    if !query.is_empty() {
        put("search_query", query.to_string());
    }
    Ok(out)
}
