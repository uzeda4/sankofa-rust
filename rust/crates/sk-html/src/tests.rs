//! Selectors here are taken verbatim from the Kotlin extensions this crate replaces.

use super::*;

fn doc(html: &str) -> Document {
    Document::parse(html, "https://site.org/manga/a/")
}

fn texts(d: &Document, sel: &str) -> Vec<String> {
    d.try_select(sel).unwrap().each_text()
}

#[test]
fn basic_selectors_groups_and_combinators() {
    let d = doc("<div class='bs'><div class='bsx'><a href='/x' title='T'><img data-src='c.jpg'></a></div></div><p id=z class='A b'>p</p>");
    assert_eq!(d.select(".listupd .bs .bsx, .bs .bsx").len(), 1);
    assert_eq!(d.select("div.bs > div.bsx > a[title=t]").len(), 1, "attribute values are case-insensitive");
    assert_eq!(d.select("p#z.a.B").len(), 1, "classes are case-insensitive");
    assert_eq!(d.select("a + p").len(), 0);
    assert_eq!(d.select("div ~ p").len(), 1, "a later sibling");
    assert_eq!(d.select("div.bs + p").len(), 1);
    assert_eq!(d.select("*").len(), 8, "html, head, body and five elements");
}

#[test]
fn attribute_operators() {
    let d = doc("<a href='/title/a?type=x' data-n='one two' rel=tag>A</a><a href='/b'>B</a><a>C</a>");
    assert_eq!(texts(&d, r"a[href*=type\=]"), ["A"]);
    assert_eq!(texts(&d, "a[href^='/title']"), ["A"]);
    assert_eq!(texts(&d, "a[href$=b]"), ["B"]);
    assert_eq!(texts(&d, "a[href]"), ["A", "B"]);
    assert_eq!(texts(&d, "a[href!='/b']"), ["A", "C"]);
    assert_eq!(texts(&d, r"a[data-n~=one\s+two]"), ["A"], "regex match");
    assert_eq!(texts(&d, "a[^data-]"), ["A"], "attribute name prefix");
    assert_eq!(texts(&d, "script[src^=data:text/javascript;base64,dHNf]").len(), 0);
    let n = doc("<i data-type='NOVEL'>n</i><span>Novel</span><span>Not a novel</span>");
    assert_eq!(n.select(r#"[data-type="novel" i], span:matchesOwn((?i)^novel$)"#).len(), 2, "case flag and an inline regex flag");
    assert_eq!(doc("<script src='data:text/javascript;base64,dHNfcmVhZGVy'></script>").select("script[src^=data:text/javascript;base64,dHNf]").len(), 1);
}

#[test]
fn contains_family() {
    let html = "<table class='infotable'><tr><td>Author</td><td>Jane</td></tr><tr><td>Artist</td><td>Joe</td></tr></table>\
        <dl class=flex><dt>Genre</dt><dd><a rel=tag>Action</a></dd></dl><dl class=flex><dt>Type</dt><dd><a rel=tag>Manhwa</a></dd></dl>\
        <ul><li>Status: <span>Ongoing</span></li></ul><div class='text-xs'>Status</div><div><span class='text-sm'>Done</span></div>";
    let d = doc(html);
    assert_eq!(texts(&d, ".infotable tr:contains(author) td:last-child"), ["Jane"], "case-insensitive");
    assert_eq!(texts(&d, "dl.flex:contains(Genre) a[rel=tag], dl.flex:contains(Type) a[rel=tag]"), ["Action", "Manhwa"]);
    assert_eq!(texts(&d, "td:containsOwn(artist) + td"), ["Joe"]);
    assert_eq!(texts(&d, "li:containsOwn(Status)"), ["Status: Ongoing"]);
    assert_eq!(texts(&d, "li:containsOwn(Ongoing)").len(), 0, "own text excludes children");
    assert_eq!(texts(&d, "div.text-xs:containsOwn(Status) + div span.text-sm"), ["Done"]);
    assert_eq!(d.select("li:matchesOwn(^Status:)").len(), 1);
    assert_eq!(d.select("td:matches(^J)").len(), 2);
    assert_eq!(texts(&d, "td:contains('Jane')"), ["Jane"], "quoted needle");
}

#[test]
fn contains_data_finds_script_text() {
    let d = doc("<script>var a = 1;</script><script>var cdns = [\"a\"];</script>");
    let s = d.select("script:containsData(cdns =)");
    assert_eq!(s.len(), 1);
    assert_eq!(s.first().unwrap().data(), "var cdns = [\"a\"];");
    assert_eq!(d.select("script").first().unwrap().text(), "", "script is data, not text");
}

#[test]
fn has_not_is() {
    let html = "<div class='truyen-list'><div class='list-truyen-item-wrap'><a data-id='1'>One</a></div>\
        <div class='list-truyen-item-wrap'><a>Two</a></div></div>\
        <div class='group-page'><a href='1'>1</a><a>2</a><a href='3'>3</a><a href='9'>Last</a></div>\
        <ul><li><div class=chbox></div><div class=eph-num><a>c1</a></div></li><li><div class=eph-num></div></li></ul>\
        <button aria-label='Next recently updated page'>n</button><button aria-label='Next recently updated page' disabled>m</button>";
    let d = doc(html);
    assert_eq!(
        d.select(":is(div.truyen-list > div.list-truyen-item-wrap, div.comic-list > .list-comic-item-wrap):has(a[data-id])").len(),
        1
    );
    assert_eq!(texts(&d, "div.group-page a:not([href]) + a:not(:contains(Last))"), ["3"]);
    assert_eq!(d.select("ul li:has(div.chbox):has(div.eph-num)").len(), 1);
    assert_eq!(texts(&d, "button[aria-label=\"Next recently updated page\"]:not([disabled])"), ["n"]);
    assert_eq!(d.select("li:has(> .eph-num)").len(), 2);
    assert_eq!(d.select("li:has(> a)").len(), 0, "a direct child only");
    assert_eq!(d.select("div:not(.truyen-list):not(.group-page):not(.list-truyen-item-wrap):not(.chbox):not(.eph-num)").len(), 0);
}

#[test]
fn structural_pseudo_classes() {
    let d = doc("<ul><li>a<li>b<li>c<li>d<li>e</ul><p>x</p><p>y</p>");
    assert_eq!(texts(&d, "li:first-child"), ["a"]);
    assert_eq!(texts(&d, "li:last-child"), ["e"]);
    assert_eq!(texts(&d, "li:nth-child(2)"), ["b"]);
    assert_eq!(texts(&d, "li:nth-child(odd)"), ["a", "c", "e"]);
    assert_eq!(texts(&d, "li:nth-child(2n+2)"), ["b", "d"]);
    assert_eq!(texts(&d, "li:nth-last-child(1)"), ["e"]);
    assert_eq!(texts(&d, "li:nth-child(-n+2)"), ["a", "b"]);
    assert_eq!(texts(&d, "p:first-of-type"), ["x"]);
    assert_eq!(texts(&d, "li:eq(2)"), ["c"]);
    assert_eq!(texts(&d, "li:lt(2)"), ["a", "b"]);
    assert_eq!(texts(&d, "li:gt(3)"), ["e"]);
    assert_eq!(d.select("ul:only-child").len(), 0);
    assert_eq!(d.select("html:root").len(), 1);
}

#[test]
fn text_rules_follow_jsoup() {
    let d = doc("<div id=a>  Hello <b>big</b>\n  <i>world</i> <p>para</p>tail<br>next <script>x()</script></div>");
    let a = d.select_first("#a").unwrap();
    assert_eq!(a.text(), "Hello big world para tail next");
    assert_eq!(a.own_text(), "Hello tail next");
    assert_eq!(d.select_first("b").unwrap().own_text(), "big");
    let w = doc("<p>one<br>two</p>").select_first("p").unwrap().whole_text();
    assert_eq!(w, "one\ntwo");
    assert_eq!(doc("<p>a&nbsp;&nbsp;b &amp; c</p>").select_first("p").unwrap().text(), "a b & c");
}

#[test]
fn urls_resolve_against_the_document_location() {
    let d = doc("<a href='/series/x'>x</a><a href='y'>y</a><a href='https://o.org/z'>z</a><img src=''><img>");
    let hrefs: Vec<_> = d.select("a").iter().map(|a| a.abs_url("href")).collect();
    assert_eq!(hrefs, ["https://site.org/series/x", "https://site.org/manga/a/y", "https://o.org/z"]);
    assert_eq!(d.select("img").first().unwrap().abs_url("src"), "");
    assert_eq!(d.select("a").abs_url("href"), "https://site.org/series/x");
}

#[test]
fn noscript_content_is_real_markup() {
    let d = doc("<noscript><img src='real.jpg'></noscript><p>x</p>");
    assert_eq!(d.select("noscript img").first().unwrap().abs_url("src"), "https://site.org/manga/a/real.jpg");
    assert_eq!(d.select("noscript").first().unwrap().tag_name(), "noscript");
}

#[test]
fn scoped_select_and_navigation() {
    let d = doc("<div class=a><p>1</p><p class=k>2</p><p>3</p></div><div class=b><p>4</p></div>");
    let a = d.select_first("div.a").unwrap();
    assert_eq!(a.select("p").each_text(), ["1", "2", "3"]);
    let k = a.select_first("p.k").unwrap();
    assert_eq!(k.next_sibling().unwrap().text(), "3");
    assert_eq!(k.prev_sibling().unwrap().text(), "1");
    assert_eq!(k.parent().unwrap().tag_name(), "div");
    assert_eq!(k.siblings().len(), 2);
    assert_eq!(k.closest("div").unwrap().has_class("a"), true);
    // an ancestor above the scope does not satisfy a descendant combinator
    assert_eq!(a.select("body p").len(), 0);
}

#[test]
fn bad_selectors_are_errors_not_panics() {
    let d = doc("<p>x</p>");
    assert!(d.try_select("p[").is_err());
    assert!(d.try_select("p:bogus").is_err());
    assert!(d.try_select(":contains(").is_err());
    assert_eq!(d.select("p[").len(), 0);
}
