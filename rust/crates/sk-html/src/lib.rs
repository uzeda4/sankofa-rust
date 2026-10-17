//! HTML parsing with jsoup's selector dialect and text rules, for sources that scrape web pages.
//!
//! ```
//! let doc = sk_html::Document::parse("<ul><li><a href='/a'>One</a><li><a href='/b'>Two</a></ul>", "https://x.org/");
//! let links = doc.select("li:has(a[href$=b]) a");
//! assert_eq!(links.text(), "Two");
//! assert_eq!(links.first().unwrap().abs_url("href"), "https://x.org/b");
//! ```
//!
//! The dialect matches jsoup (what the Kotlin extensions use): `:contains`, `:containsOwn`, `:has`, `:matchesOwn`,
//! case-insensitive attribute values, `text()`/`ownText()` whitespace rules and `abs:` URL resolution.

mod dom;
mod selector;

use ego_tree::NodeId;
use scraper::{ElementRef, Html};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use url::Url;

pub use selector::Group as Selector;

thread_local! {
    static CACHE: RefCell<HashMap<String, Rc<Selector>>> = RefCell::new(HashMap::new());
}

/// Parses (or fetches from the cache) a selector. Sources call `select` in loops, so parsing once pays off.
pub fn compile(selector: &str) -> Result<Rc<Selector>, String> {
    CACHE.with(|c| {
        if let Some(s) = c.borrow().get(selector) {
            return Ok(s.clone());
        }
        let parsed = Rc::new(selector::parse(selector)?);
        let mut cache = c.borrow_mut();
        if cache.len() > 512 {
            cache.clear();
        }
        cache.insert(selector.to_string(), parsed.clone());
        Ok(parsed)
    })
}

pub struct Document {
    html: Html,
    base: Option<Url>,
    location: String,
}

impl Document {
    /// Parses `html`; relative URLs resolve against `base_url` (what jsoup calls the document location).
    pub fn parse(html: &str, base_url: &str) -> Self {
        Self { html: Html::parse_document(&alias_noscript(html)), base: Url::parse(base_url).ok(), location: base_url.to_string() }
    }

    /// The URL the document was loaded from, as given to [`Document::parse`].
    pub fn location(&self) -> &str {
        &self.location
    }

    /// The `<html>` element.
    pub fn root(&self) -> El<'_> {
        El { el: self.html.root_element(), doc: self }
    }

    /// Elements matching `selector`, in document order. A malformed selector matches nothing; use
    /// [`Document::try_select`] to see the error (the selectors in a source are constants, so a unit test is the
    /// right place to catch them).
    pub fn select(&self, selector: &str) -> Elements<'_> {
        self.try_select(selector).unwrap_or_default()
    }

    pub fn try_select(&self, selector: &str) -> Result<Elements<'_>, String> {
        let root = self.html.tree.root().id();
        let group = compile(selector)?;
        let items = self
            .html
            .tree
            .root()
            .descendants()
            .filter_map(ElementRef::wrap)
            .filter(|e| group.matches(e, root))
            .map(|el| El { el, doc: self })
            .collect();
        Ok(Elements(items))
    }

    pub fn select_first(&self, selector: &str) -> Option<El<'_>> {
        self.select(selector).0.into_iter().next()
    }

    pub fn title(&self) -> String {
        self.select_first("title").map(|t| t.text()).unwrap_or_default()
    }

    /// The `<body>` text, as jsoup's `body().text()`.
    pub fn text(&self) -> String {
        self.root().text()
    }

    /// The whole document serialised as HTML.
    pub fn html(&self) -> String {
        self.html.html()
    }
}

/// An element and the document it belongs to.
#[derive(Clone, Copy)]
pub struct El<'a> {
    el: ElementRef<'a>,
    doc: &'a Document,
}

impl PartialEq for El<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.el.id() == other.el.id()
    }
}

impl<'a> El<'a> {
    fn wrap(&self, el: ElementRef<'a>) -> El<'a> {
        El { el, doc: self.doc }
    }

    pub fn id_node(&self) -> NodeId {
        self.el.id()
    }

    /// Descendants (and this element itself, like jsoup) matching `selector`.
    pub fn select(&self, selector: &str) -> Elements<'a> {
        self.try_select(selector).unwrap_or_default()
    }

    pub fn try_select(&self, selector: &str) -> Result<Elements<'a>, String> {
        let group = compile(selector)?;
        let root = self.el.id();
        Ok(Elements(
            self.el
                .descendants()
                .filter_map(ElementRef::wrap)
                .filter(|e| group.matches(e, root))
                .map(|el| self.wrap(el))
                .collect(),
        ))
    }

    pub fn select_first(&self, selector: &str) -> Option<El<'a>> {
        self.select(selector).0.into_iter().next()
    }

    /// True when this element matches `selector` (as a root of its own subtree).
    pub fn is(&self, selector: &str) -> bool {
        compile(selector).map(|g| g.matches(&self.el, self.doc.html.tree.root().id())).unwrap_or(false)
    }

    /// The nearest ancestor-or-self that matches `selector`.
    pub fn closest(&self, selector: &str) -> Option<El<'a>> {
        let group = compile(selector).ok()?;
        let root = self.doc.html.tree.root().id();
        let mut cur = Some(self.el);
        while let Some(e) = cur {
            if group.matches(&e, root) {
                return Some(self.wrap(e));
            }
            cur = e.parent().and_then(ElementRef::wrap);
        }
        None
    }

    pub fn tag_name(&self) -> String {
        dom::name(&self.el).to_string()
    }

    /// Attribute value, or an empty string when absent (jsoup's `attr`). Use [`El::attr_opt`] to tell them apart.
    pub fn attr(&self, name: &str) -> String {
        self.attr_opt(name).unwrap_or_default()
    }

    pub fn attr_opt(&self, name: &str) -> Option<String> {
        self.el
            .value()
            .attr(name)
            .or_else(|| self.el.value().attrs().find(|(k, _)| k.eq_ignore_ascii_case(name)).map(|(_, v)| v))
            .map(str::to_string)
    }

    pub fn has_attr(&self, name: &str) -> bool {
        self.attr_opt(name).is_some()
    }

    /// `abs:attr`: the attribute resolved against the document location, or empty when it cannot be.
    pub fn abs_url(&self, name: &str) -> String {
        let value = self.attr(name);
        resolve(self.doc.base.as_ref(), value.trim())
    }

    pub fn id(&self) -> String {
        self.el.value().id().unwrap_or_default().to_string()
    }

    pub fn class_names(&self) -> Vec<String> {
        self.el.value().classes().map(str::to_string).collect()
    }

    pub fn has_class(&self, class: &str) -> bool {
        self.el.value().classes().any(|c| c.eq_ignore_ascii_case(class))
    }

    /// All descendant text, whitespace-normalised (jsoup `text()`).
    pub fn text(&self) -> String {
        dom::text(&self.el)
    }

    /// Only this element's own text nodes (jsoup `ownText()`).
    pub fn own_text(&self) -> String {
        dom::own_text(&self.el)
    }

    /// Text exactly as written, `<br>` as a newline (jsoup `wholeText()`).
    pub fn whole_text(&self) -> String {
        dom::whole_text(&self.el)
    }

    /// Contents of `<script>`/`<style>` (jsoup `data()`).
    pub fn data(&self) -> String {
        dom::data(&self.el)
    }

    /// `text()` or `None` when blank.
    pub fn text_or_none(&self) -> Option<String> {
        Some(self.text()).filter(|t| !t.is_empty())
    }

    pub fn inner_html(&self) -> String {
        self.el.inner_html().replace("x-noscript", "noscript")
    }

    pub fn outer_html(&self) -> String {
        self.el.html().replace("x-noscript", "noscript")
    }

    pub fn parent(&self) -> Option<El<'a>> {
        self.el.parent().and_then(ElementRef::wrap).map(|e| self.wrap(e))
    }

    pub fn children(&self) -> Vec<El<'a>> {
        self.el.children().filter_map(ElementRef::wrap).map(|e| self.wrap(e)).collect()
    }

    pub fn first_child(&self) -> Option<El<'a>> {
        self.el.children().find_map(ElementRef::wrap).map(|e| self.wrap(e))
    }

    pub fn last_child(&self) -> Option<El<'a>> {
        self.el.children().rev().find_map(ElementRef::wrap).map(|e| self.wrap(e))
    }

    pub fn next_sibling(&self) -> Option<El<'a>> {
        std::iter::successors(self.el.next_sibling(), |n| n.next_sibling()).find_map(ElementRef::wrap).map(|e| self.wrap(e))
    }

    pub fn prev_sibling(&self) -> Option<El<'a>> {
        std::iter::successors(self.el.prev_sibling(), |n| n.prev_sibling()).find_map(ElementRef::wrap).map(|e| self.wrap(e))
    }

    /// Element siblings (not including this element), in document order.
    pub fn siblings(&self) -> Vec<El<'a>> {
        self.parent()
            .map(|p| p.children().into_iter().filter(|c| c != self).collect())
            .unwrap_or_default()
    }
}

/// The result of a `select`: jsoup's `Elements`.
#[derive(Default)]
pub struct Elements<'a>(pub Vec<El<'a>>);

impl<'a> Elements<'a> {
    pub fn first(&self) -> Option<&El<'a>> {
        self.0.first()
    }

    pub fn last(&self) -> Option<&El<'a>> {
        self.0.last()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, El<'a>> {
        self.0.iter()
    }

    /// Text of every element joined by a space (jsoup `Elements.text()`).
    pub fn text(&self) -> String {
        self.0.iter().map(El::text).collect::<Vec<_>>().join(" ")
    }

    pub fn each_text(&self) -> Vec<String> {
        self.0.iter().map(El::text).collect()
    }

    pub fn own_text(&self) -> String {
        self.0.iter().map(El::own_text).collect::<Vec<_>>().join(" ")
    }

    /// The attribute of the first element that has it, or empty.
    pub fn attr(&self, name: &str) -> String {
        self.0.iter().find_map(|e| e.attr_opt(name)).unwrap_or_default()
    }

    pub fn abs_url(&self, name: &str) -> String {
        self.0.iter().find(|e| e.has_attr(name)).map(|e| e.abs_url(name)).unwrap_or_default()
    }

    pub fn select(&self, selector: &str) -> Elements<'a> {
        let mut out: Vec<El<'a>> = Vec::new();
        for e in &self.0 {
            for m in e.select(selector).0 {
                if !out.contains(&m) {
                    out.push(m);
                }
            }
        }
        Elements(out)
    }
}

impl<'a> IntoIterator for Elements<'a> {
    type Item = El<'a>;
    type IntoIter = std::vec::IntoIter<El<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, 'b> IntoIterator for &'b Elements<'a> {
    type Item = &'b El<'a>;
    type IntoIter = std::slice::Iter<'b, El<'a>>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

/// Resolves `reference` against `base` (jsoup `absUrl`); empty when there is no base or the result is not a URL.
pub fn resolve(base: Option<&Url>, reference: &str) -> String {
    if reference.is_empty() {
        return String::new();
    }
    match base {
        Some(b) => b.join(reference).map(|u| u.to_string()).unwrap_or_default(),
        None => Url::parse(reference).map(|u| u.to_string()).unwrap_or_default(),
    }
}

/// `<noscript>` -> `<x-noscript>` so its content is parsed as elements (see `dom::NOSCRIPT_ALIAS`).
fn alias_noscript(html: &str) -> std::borrow::Cow<'_, str> {
    let lower = html.to_ascii_lowercase();
    if !lower.contains("noscript") {
        return html.into();
    }
    let mut out = String::with_capacity(html.len() + 32);
    let mut last = 0;
    let mut from = 0;
    while let Some(i) = lower[from..].find("noscript") {
        let at = from + i;
        let before = &lower[..at];
        if before.ends_with('<') || before.ends_with("</") {
            out.push_str(&html[last..at]);
            out.push_str("x-");
            last = at;
        }
        from = at + "noscript".len();
    }
    out.push_str(&html[last..]);
    out.into()
}

#[cfg(test)]
mod tests;
