use sk_source_sdk::*;

#[derive(Default)]
struct Bench;

impl Source for Bench {
    fn info(&self) -> Info {
        Info { name: "Bench".into(), lang: "en".into(), base_url: "https://x.org".into(), ..Default::default() }
    }
    fn popular(&self, _page: u32) -> Result<MangaPage> {
        let body = get("https://x.org/").send()?.body;
        let doc = sk_html::Document::parse(&body, "https://x.org/");
        let n = doc.select("a[href]").len();
        let m = doc.select("div:has(img) a:not([href^=#])").len();
        let t = doc.root().text().len();
        Ok(MangaPage { mangas: vec![Manga { url: format!("{n}/{m}/{t}"), title: "bench".into(), ..Default::default() }], has_next: false })
    }
    fn details(&self, _: &Manga) -> Result<Manga> { Err("x".into()) }
    fn chapters(&self, _: &Manga) -> Result<Vec<Chapter>> { Ok(vec![]) }
    fn pages(&self, _: &Chapter) -> Result<Vec<Page>> { Ok(vec![]) }
}
export_source!(Bench);
