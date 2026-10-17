//! Text extraction with jsoup's rules (`text`, `ownText`, `wholeText`, `data`).

use ego_tree::iter::Edge;
use ego_tree::NodeRef;
use scraper::node::Node;
use scraper::ElementRef;

/// Parsing replaces `<noscript>` with this element so its content becomes real elements (as in jsoup, which parses
/// with scripting off) instead of raw text; the name is mapped back by [`name`].
pub(crate) const NOSCRIPT_ALIAS: &str = "x-noscript";

/// Tag name as the page wrote it.
pub(crate) fn name<'a>(el: &'a ElementRef<'_>) -> &'a str {
    match el.value().name() {
        NOSCRIPT_ALIAS => "noscript",
        n => n,
    }
}

fn is_block(tag: &str) -> bool {
    matches!(
        tag,
        "address" | "article" | "aside" | "blockquote" | "body" | "center" | "dd" | "details" | "dialog" | "div" | "dl"
            | "dt" | "fieldset" | "figcaption" | "figure" | "footer" | "form" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6"
            | "head" | "header" | "hgroup" | "hr" | "html" | "li" | "main" | "nav" | "ol" | "option" | "p" | "pre"
            | "section" | "summary" | "table" | "tbody" | "td" | "tfoot" | "th" | "thead" | "title" | "tr" | "ul"
    )
}

fn is_data_tag(tag: &str) -> bool {
    matches!(tag, "script" | "style")
}

/// jsoup keeps `<script>` and `<style>` contents as data, not text.
fn text_of_node<'a>(n: &NodeRef<'a, Node>) -> Option<&'a str> {
    match n.value() {
        Node::Text(t) => {
            let in_data = n
                .parent()
                .and_then(ElementRef::wrap)
                .is_some_and(|p| is_data_tag(p.value().name()));
            (!in_data).then_some(&**t)
        }
        _ => None,
    }
}

fn push_normalised(out: &mut String, text: &str) {
    let mut last_ws = out.chars().last().is_none_or(char::is_whitespace);
    for c in text.chars() {
        if c.is_whitespace() {
            if !last_ws {
                out.push(' ');
                last_ws = true;
            }
        } else {
            out.push(c);
            last_ws = false;
        }
    }
}

fn trimmed(mut s: String) -> String {
    let t = s.trim_end().len();
    s.truncate(t);
    s.trim_start().to_string()
}

/// `Element.text()`: all descendant text, whitespace collapsed, block elements and `<br>` separated by a space.
pub(crate) fn text(el: &ElementRef<'_>) -> String {
    let mut out = String::new();
    let root = el.id();
    for edge in el.traverse() {
        match edge {
            Edge::Open(n) => match n.value() {
                Node::Text(_) => {
                    if let Some(t) = text_of_node(&n) {
                        push_normalised(&mut out, t);
                    }
                }
                Node::Element(e) if n.id() != root => {
                    let tag = if e.name() == NOSCRIPT_ALIAS { "noscript" } else { e.name() };
                    if (is_block(tag) || tag == "br") && !out.is_empty() && !out.ends_with(' ') {
                        out.push(' ');
                    }
                }
                _ => {}
            },
            Edge::Close(n) => {
                if let Node::Element(e) = n.value() {
                    if n.id() != root
                        && is_block(e.name())
                        && n.next_sibling().is_some_and(|s| matches!(s.value(), Node::Text(_)))
                        && !out.is_empty()
                        && !out.ends_with(' ')
                    {
                        out.push(' ');
                    }
                }
            }
        }
    }
    trimmed(out)
}

/// `Element.ownText()`: only the element's direct text nodes, normalised and trimmed.
pub(crate) fn own_text(el: &ElementRef<'_>) -> String {
    let mut out = String::new();
    for c in el.children() {
        if let Some(t) = text_of_node(&c) {
            push_normalised(&mut out, t);
        } else if let Node::Element(e) = c.value() {
            if e.name() == "br" && !out.is_empty() && !out.ends_with(' ') {
                out.push(' ');
            }
        }
    }
    trimmed(out)
}

/// `Element.wholeText()`: text exactly as written, `<br>` as a newline.
pub(crate) fn whole_text(el: &ElementRef<'_>) -> String {
    let mut out = String::new();
    for edge in el.traverse() {
        if let Edge::Open(n) = edge {
            match n.value() {
                Node::Text(_) => {
                    if let Some(t) = text_of_node(&n) {
                        out.push_str(t);
                    }
                }
                Node::Element(e) if e.name() == "br" => out.push('\n'),
                _ => {}
            }
        }
    }
    out
}

/// `Element.data()`: the contents of `<script>`/`<style>` below this element (and its own, for those tags).
pub(crate) fn data(el: &ElementRef<'_>) -> String {
    let mut out = String::new();
    for n in el.descendants() {
        if let Node::Text(t) = n.value() {
            if n.parent().and_then(ElementRef::wrap).is_some_and(|p| is_data_tag(p.value().name())) {
                out.push_str(t);
            }
        }
    }
    out
}

/// jsoup's normalisation for `:contains`: lower case, whitespace collapsed, trimmed.
pub(crate) fn normalise(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    push_normalised(&mut out, &s.to_lowercase());
    trimmed(out)
}
