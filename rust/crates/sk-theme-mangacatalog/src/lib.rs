//! MangaCatalog: a network of sites that each read one franchise (a main series and its spin-offs). The "popular" list is
//! the site's own list of titles; details and chapters sit in one page; the pages of a chapter are `img[data-src]`.
//!
//! Port of the `mangacatalog` multisrc of the Kotlin extensions. Titles and chapters are stored as absolute addresses.

use sk_source_sdk::html::Document;
use sk_source_sdk::*;
use std::marker::PhantomData;

pub trait Theme: 'static {
    const NAME: &'static str;
    const LANG: &'static str = "en";
    const BASE_URL: &'static str;
    const VERSION_ID: u32 = 1;

    /// `(title, address)` of every title the site has; an address that starts with `/` is relative to `BASE_URL`.
    const SOURCES: &'static [(&'static str, &'static str)];
    const CHAPTER_LIST_SELECTOR: &'static str = "div.w-full > div.bg-bg-secondary > div.grid";
}

pub struct Site<T>(PhantomData<T>);

impl<T> Default for Site<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

fn absolute<T: Theme>(address: &str) -> String {
    if address.starts_with('/') {
        format!("{}{address}", base_url(T::BASE_URL))
    } else {
        address.to_string()
    }
}

fn title_of<T: Theme>(name: &str, address: &str) -> Manga {
    Manga { url: absolute::<T>(address), title: name.to_string(), ..Default::default() }
}

fn titles<T: Theme>(query: &str) -> MangaPage {
    let q = query.to_lowercase();
    let mut all: Vec<(&str, &str)> = T::SOURCES.to_vec();
    all.sort_by(|a, b| a.0.cmp(b.0));
    all.dedup_by(|a, b| a.1 == b.1);
    MangaPage {
        mangas: all.into_iter().filter(|(name, _)| name.to_lowercase().contains(&q)).map(|(n, a)| title_of::<T>(n, a)).collect(),
        has_next: false,
    }
}

fn page_of(url: &str) -> Result<Document> {
    Ok(get(url).send()?.html())
}

impl<T: Theme> Source for Site<T> {
    fn info(&self) -> Info {
        Info {
            name: T::NAME.into(),
            lang: T::LANG.into(),
            base_url: T::BASE_URL.into(),
            version_id: T::VERSION_ID,
            custom_urls: true,
            ..Default::default()
        }
    }

    fn popular(&self, _page: u32) -> Result<MangaPage> {
        Ok(titles::<T>(""))
    }

    fn search(&self, _page: u32, query: &str, _filters: &Filters) -> Result<MangaPage> {
        Ok(titles::<T>(query))
    }

    fn details(&self, manga: &Manga) -> Result<Manga> {
        let doc = page_of(&manga.url)?;
        let info = doc.select("div.bg-bg-secondary > div.px-6 > div.flex-col").text();
        let description = match info.split_once("Description") {
            Some((_, after)) => after.trim().to_string(),
            None => info,
        };
        Ok(Manga {
            url: manga.url.clone(),
            title: doc.select("div.container > h1").text(),
            description: Some(description).filter(|d| !d.is_empty()),
            thumbnail_url: Some(doc.select("div.flex > img").abs_url("src")).filter(|u| !u.is_empty()),
            ..Default::default()
        })
    }

    fn chapters(&self, manga: &Manga) -> Result<Vec<Chapter>> {
        let doc = page_of(&manga.url)?;
        Ok(doc
            .select(T::CHAPTER_LIST_SELECTOR)
            .iter()
            .map(|el| {
                let (first, second) = (el.select(".col-span-4 > a").text(), el.select(".text-xs:not(a)").text());
                Chapter {
                    url: el.select(".col-span-4 > a").abs_url("href"),
                    name: if second.is_empty() { first } else { format!("{first} - {second}") },
                    chapter_number: -1.0,
                    scanlator: None,
                    date_upload: 0,
                    memo: None,
                }
            })
            .collect())
    }

    fn update(&self, manga: &Manga, fetch_details: bool, fetch_chapters: bool) -> Result<Update> {
        // one page holds both: fetch it once
        Ok(Update {
            manga: if fetch_details { Some(self.details(manga)?) } else { None },
            chapters: if fetch_chapters { Some(self.chapters(manga)?) } else { None },
        })
    }

    fn pages(&self, chapter: &Chapter) -> Result<Vec<Page>> {
        let doc = page_of(&chapter.url)?;
        Ok(doc
            .select("img[data-src]")
            .iter()
            .enumerate()
            .map(|(i, img)| Page { index: i as u32, url: String::new(), image_url: img.abs_url("data-src"), headers: Vec::new(), ops: Vec::new(), mirrors: Vec::new() })
            .collect())
    }

    fn manga_url(&self, manga: &Manga) -> Option<String> {
        Some(manga.url.clone())
    }

    fn chapter_url(&self, chapter: &Chapter) -> Option<String> {
        Some(chapter.url.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Demo;
    impl Theme for Demo {
        const NAME: &'static str = "Demo";
        const BASE_URL: &'static str = "https://demo.test";
        const SOURCES: &'static [(&'static str, &'static str)] = &[("Zeta", "/manga/zeta/"), ("Alpha", "/manga/alpha/"), ("Alpha again", "/manga/alpha/")];
    }

    #[test]
    fn the_title_list_is_sorted_deduplicated_and_searchable() {
        let all = titles::<Demo>("");
        assert_eq!(all.mangas.iter().map(|m| m.title.as_str()).collect::<Vec<_>>(), ["Alpha", "Zeta"]);
        assert_eq!(all.mangas[0].url, "https://demo.test/manga/alpha/");
        assert_eq!(titles::<Demo>("zet").mangas.len(), 1);
    }
}
