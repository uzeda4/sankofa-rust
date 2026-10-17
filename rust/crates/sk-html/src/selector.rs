//! jsoup-compatible selector parsing and matching.
//!
//! Supported: type, `*`, `#id`, `.class`, `[attr]`, `[^prefix]`, `[a=v]`, `[a!=v]`, `[a^=v]`, `[a$=v]`, `[a*=v]`,
//! `[a~=regex]`, combinators (` `, `>`, `+`, `~`), groups (`,`), and the pseudo-classes `:first-child`,
//! `:last-child`, `:only-child`, `:nth-child`, `:nth-last-child`, the `-of-type` forms, `:empty`, `:root`,
//! `:not`, `:is`, `:has`, `:contains`, `:containsOwn`, `:containsWholeText`, `:containsData`, `:matches`,
//! `:matchesOwn`, `:eq`, `:lt`, `:gt`.
//!
//! Like jsoup, attribute values and class names compare case-insensitively, and an ancestor search stops at the
//! element `select` was called on (inclusive).

use crate::dom;
use ego_tree::NodeId;
use fancy_regex::Regex;
use scraper::node::Node;
use scraper::ElementRef;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Comb {
    Descendant,
    Child,
    Next,
    Later,
}

#[derive(Debug)]
enum AttrOp {
    Exists,
    NamePrefix,
    Eq,
    Ne,
    Starts,
    Ends,
    Contains,
    Regex(Regex),
}

#[derive(Debug)]
enum Simple {
    Any,
    Tag(String),
    Id(String),
    Class(String),
    Attr { name: String, op: AttrOp, value: String },
    Not(Group),
    Is(Group),
    Has(Group),
    /// `:contains` and friends: normalised needle, own text only, whole text (no normalisation).
    Contains { needle: String, own: bool, whole: bool },
    ContainsData(String),
    Matches { re: Regex, own: bool },
    Nth { a: i64, b: i64, last: bool, of_type: bool },
    OnlyChild,
    OnlyOfType,
    Empty,
    Root,
    Scope,
    Eq(usize),
    Lt(usize),
    Gt(usize),
}

#[derive(Debug, Default)]
struct Compound(Vec<Simple>);

/// `parts[0].0` is unused; `parts[i].0` relates `parts[i - 1]` to `parts[i]`.
#[derive(Debug, Default)]
struct Complex(Vec<(Comb, Compound)>);

#[derive(Debug, Default)]
pub struct Group(Vec<Complex>);

// ------------------------------------------------------------------ parsing

struct Parser<'a> {
    chars: Vec<char>,
    pos: usize,
    src: &'a str,
}

type PResult<T> = Result<T, String>;

pub fn parse(selector: &str) -> PResult<Group> {
    let mut p = Parser { chars: selector.chars().collect(), pos: 0, src: selector };
    let g = p.group(false)?;
    p.skip_ws();
    if p.pos < p.chars.len() {
        return Err(p.err("unexpected character"));
    }
    Ok(g)
}

impl Parser<'_> {
    fn err(&self, msg: &str) -> String {
        format!("bad selector `{}` at {}: {msg}", self.src, self.pos)
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn skip_ws(&mut self) -> bool {
        let start = self.pos;
        while self.peek().is_some_and(char::is_whitespace) {
            self.pos += 1;
        }
        self.pos > start
    }

    /// `relative` allows each complex selector to start with a combinator (`:has(> a)`), anchored to `:scope`.
    fn group(&mut self, relative: bool) -> PResult<Group> {
        let mut out = Vec::new();
        loop {
            out.push(self.complex(relative)?);
            self.skip_ws();
            if self.peek() == Some(',') {
                self.pos += 1;
            } else {
                return Ok(Group(out));
            }
        }
    }

    fn complex(&mut self, relative: bool) -> PResult<Complex> {
        let mut parts: Vec<(Comb, Compound)> = Vec::new();
        self.skip_ws();
        let mut comb = Comb::Descendant;
        if relative {
            parts.push((Comb::Descendant, Compound(vec![Simple::Scope])));
            comb = self.combinator().unwrap_or(Comb::Descendant);
            self.skip_ws();
        } else if matches!(self.peek(), Some('>' | '+' | '~')) {
            // A leading combinator at the top level is relative to the element `select` is called on.
            parts.push((Comb::Descendant, Compound(vec![Simple::Scope])));
            comb = self.combinator().unwrap_or(Comb::Descendant);
            self.skip_ws();
        }
        loop {
            let c = self.compound()?;
            if c.0.is_empty() {
                return Err(self.err("expected a selector"));
            }
            parts.push((comb, c));
            let had_ws = self.skip_ws();
            match self.peek() {
                None | Some(',' | ')') => return Ok(Complex(parts)),
                _ => {}
            }
            comb = match self.combinator() {
                Some(c) => c,
                None if had_ws => Comb::Descendant,
                None => return Err(self.err("unexpected character")),
            };
            self.skip_ws();
        }
    }

    fn combinator(&mut self) -> Option<Comb> {
        let c = match self.peek()? {
            '>' => Comb::Child,
            '+' => Comb::Next,
            '~' => Comb::Later,
            _ => return None,
        };
        self.pos += 1;
        Some(c)
    }

    fn ident(&mut self) -> String {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c == '\\' {
                self.pos += 1;
                if let Some(n) = self.peek() {
                    s.push(n);
                    self.pos += 1;
                }
            } else if c.is_alphanumeric() || matches!(c, '-' | '_' | '|') || !c.is_ascii() {
                s.push(c);
                self.pos += 1;
            } else {
                break;
            }
        }
        s
    }

    fn compound(&mut self) -> PResult<Compound> {
        let mut simples = Vec::new();
        if self.peek() == Some('*') {
            self.pos += 1;
            simples.push(Simple::Any);
        } else {
            let tag = self.ident();
            if !tag.is_empty() {
                // `ns|tag` selects by the local name.
                let t = tag.rsplit('|').next().unwrap_or(&tag).to_ascii_lowercase();
                simples.push(Simple::Tag(t));
            }
        }
        loop {
            match self.peek() {
                Some('#') => {
                    self.pos += 1;
                    simples.push(Simple::Id(self.ident()));
                }
                Some('.') => {
                    self.pos += 1;
                    simples.push(Simple::Class(self.ident().to_lowercase()));
                }
                Some('[') => {
                    self.pos += 1;
                    simples.push(self.attribute()?);
                }
                Some(':') => {
                    self.pos += 1;
                    simples.push(self.pseudo()?);
                }
                _ => return Ok(Compound(simples)),
            }
        }
    }

    fn attribute(&mut self) -> PResult<Simple> {
        self.skip_ws();
        if self.peek() == Some('^') {
            self.pos += 1;
            let name = self.ident().to_lowercase();
            self.expect(']')?;
            return Ok(Simple::Attr { name, op: AttrOp::NamePrefix, value: String::new() });
        }
        let name = self.ident().to_lowercase();
        self.skip_ws();
        let op = match self.peek() {
            Some(']') => {
                self.pos += 1;
                return Ok(Simple::Attr { name, op: AttrOp::Exists, value: String::new() });
            }
            Some('=') => {
                self.pos += 1;
                "="
            }
            Some(c @ ('!' | '^' | '$' | '*' | '~')) => {
                self.pos += 1;
                if self.peek() != Some('=') {
                    return Err(self.err("expected `=`"));
                }
                self.pos += 1;
                match c {
                    '!' => "!=",
                    '^' => "^=",
                    '$' => "$=",
                    '*' => "*=",
                    _ => "~=",
                }
            }
            _ => return Err(self.err("bad attribute selector")),
        };
        self.skip_ws();
        let mut value = String::new();
        match self.peek() {
            Some(q @ ('"' | '\'')) => {
                self.pos += 1;
                while let Some(c) = self.peek() {
                    self.pos += 1;
                    if c == q {
                        break;
                    }
                    if c == '\\' {
                        if let Some(n) = self.peek() {
                            value.push(n);
                            self.pos += 1;
                        }
                    } else {
                        value.push(c);
                    }
                }
                self.skip_ws();
            }
            _ => {
                while let Some(c) = self.peek() {
                    if c == ']' {
                        break;
                    }
                    self.pos += 1;
                    // A regex keeps its backslashes; other values use them to escape (`type\=`).
                    if c == '\\' && op != "~=" {
                        if let Some(n) = self.peek() {
                            value.push(n);
                            self.pos += 1;
                        }
                    } else {
                        value.push(c);
                    }
                }
                value = value.trim().to_string();
            }
        }
        // `[attr="v" i]`: the case flag. Values already compare case-insensitively, as in jsoup.
        self.skip_ws();
        if matches!(self.peek(), Some('i' | 'I' | 's' | 'S')) && self.chars.get(self.pos + 1) == Some(&']') {
            self.pos += 1;
        }
        self.expect(']')?;
        let op = match op {
            "=" => AttrOp::Eq,
            "!=" => AttrOp::Ne,
            "^=" => AttrOp::Starts,
            "$=" => AttrOp::Ends,
            "*=" => AttrOp::Contains,
            _ => AttrOp::Regex(Regex::new(&value).map_err(|e| self.err(&e.to_string()))?),
        };
        Ok(Simple::Attr { name, op, value: value.to_lowercase() })
    }

    fn expect(&mut self, c: char) -> PResult<()> {
        if self.peek() == Some(c) {
            self.pos += 1;
            Ok(())
        } else {
            Err(self.err(&format!("expected `{c}`")))
        }
    }

    /// The text between balanced parentheses, quotes respected; the cursor is just after `(`.
    fn balanced(&mut self) -> PResult<String> {
        let mut depth = 1;
        let mut quote: Option<char> = None;
        let mut out = String::new();
        while let Some(c) = self.peek() {
            self.pos += 1;
            match (c, quote) {
                ('\\', _) => {
                    out.push(c);
                    if let Some(n) = self.peek() {
                        out.push(n);
                        self.pos += 1;
                    }
                    continue;
                }
                ('\'' | '"', None) => quote = Some(c),
                (q, Some(open)) if q == open => quote = None,
                ('(', None) => depth += 1,
                (')', None) => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(out);
                    }
                }
                _ => {}
            }
            out.push(c);
        }
        Err(self.err("unbalanced parentheses"))
    }

    fn pseudo(&mut self) -> PResult<Simple> {
        let name = self.ident().to_lowercase();
        let arg = if self.peek() == Some('(') {
            self.pos += 1;
            Some(self.balanced()?)
        } else {
            None
        };
        let need = |a: &Option<String>| a.clone().ok_or_else(|| self.err(&format!(":{name} needs an argument")));
        let sub = |s: &str, relative: bool| -> PResult<Group> {
            let mut p = Parser { chars: s.chars().collect(), pos: 0, src: self.src };
            let g = p.group(relative)?;
            p.skip_ws();
            if p.pos < p.chars.len() {
                return Err(p.err("unexpected character"));
            }
            Ok(g)
        };
        Ok(match name.as_str() {
            "not" => Simple::Not(sub(&need(&arg)?, false)?),
            "is" | "where" | "matches-any" => Simple::Is(sub(&need(&arg)?, false)?),
            "has" => Simple::Has(sub(&need(&arg)?, true)?),
            "contains" | "-soup-contains" => Simple::Contains { needle: dom::normalise(&unquote(&need(&arg)?)), own: false, whole: false },
            "containsown" => Simple::Contains { needle: dom::normalise(&unquote(&need(&arg)?)), own: true, whole: false },
            "containswholetext" => Simple::Contains { needle: unquote(&need(&arg)?), own: false, whole: true },
            "containswholeowntext" => Simple::Contains { needle: unquote(&need(&arg)?), own: true, whole: true },
            "containsdata" => Simple::ContainsData(unquote(&need(&arg)?).to_lowercase()),
            "matches" | "matchesown" => Simple::Matches {
                re: Regex::new(&unquote(&need(&arg)?)).map_err(|e| self.err(&e.to_string()))?,
                own: name == "matchesown",
            },
            "first-child" => Simple::Nth { a: 0, b: 1, last: false, of_type: false },
            "last-child" => Simple::Nth { a: 0, b: 1, last: true, of_type: false },
            "first-of-type" => Simple::Nth { a: 0, b: 1, last: false, of_type: true },
            "last-of-type" => Simple::Nth { a: 0, b: 1, last: true, of_type: true },
            "nth-child" | "nth-last-child" | "nth-of-type" | "nth-last-of-type" => {
                let (a, b) = parse_nth(&need(&arg)?).ok_or_else(|| self.err("bad :nth expression"))?;
                Simple::Nth { a, b, last: name.contains("last"), of_type: name.ends_with("of-type") }
            }
            "only-child" => Simple::OnlyChild,
            "only-of-type" => Simple::OnlyOfType,
            "empty" => Simple::Empty,
            "root" => Simple::Root,
            "scope" => Simple::Scope,
            "eq" | "lt" | "gt" => {
                let n: usize = need(&arg)?.trim().parse().map_err(|_| self.err("expected a number"))?;
                match name.as_str() {
                    "eq" => Simple::Eq(n),
                    "lt" => Simple::Lt(n),
                    _ => Simple::Gt(n),
                }
            }
            other => return Err(self.err(&format!("unsupported pseudo-class :{other}"))),
        })
    }
}

fn unquote(s: &str) -> String {
    let t = s.trim();
    let inner = match (t.chars().next(), t.chars().last()) {
        (Some(a @ ('"' | '\'')), Some(b)) if a == b && t.len() >= 2 => &t[1..t.len() - 1],
        _ => t,
    };
    let mut out = String::new();
    let mut it = inner.chars();
    while let Some(c) = it.next() {
        if c == '\\' {
            if let Some(n) = it.next() {
                out.push(n);
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// `odd`, `even`, `n`, `2n`, `-n+3`, `3`, `2n+1` -> (a, b) for `a*n + b`.
fn parse_nth(s: &str) -> Option<(i64, i64)> {
    let s: String = s.chars().filter(|c| !c.is_whitespace()).collect::<String>().to_lowercase();
    match s.as_str() {
        "odd" => return Some((2, 1)),
        "even" => return Some((2, 0)),
        _ => {}
    }
    match s.split_once('n') {
        None => s.parse().ok().map(|b| (0, b)),
        Some((a, b)) => {
            let a = match a {
                "" | "+" => 1,
                "-" => -1,
                n => n.parse().ok()?,
            };
            let b = if b.is_empty() { 0 } else { b.parse().ok()? };
            Some((a, b))
        }
    }
}

// ------------------------------------------------------------------ matching

#[derive(Clone, Copy)]
struct Ctx {
    /// Ancestor searches stop here (inclusive).
    root: NodeId,
    /// What `:scope` refers to.
    scope: NodeId,
}

impl Group {
    pub(crate) fn matches(&self, el: &ElementRef<'_>, root: NodeId) -> bool {
        let ctx = Ctx { root, scope: root };
        self.0.iter().any(|c| complex_matches(&c.0, c.0.len() - 1, el, ctx))
    }
}

fn element_parent<'a>(el: &ElementRef<'a>) -> Option<ElementRef<'a>> {
    el.parent().and_then(ElementRef::wrap)
}

fn prev_element<'a>(el: &ElementRef<'a>) -> Option<ElementRef<'a>> {
    let mut n = el.prev_sibling();
    while let Some(s) = n {
        if let Some(e) = ElementRef::wrap(s) {
            return Some(e);
        }
        n = s.prev_sibling();
    }
    None
}

fn complex_matches(parts: &[(Comb, Compound)], i: usize, el: &ElementRef<'_>, ctx: Ctx) -> bool {
    if !compound_matches(&parts[i].1, el, ctx) {
        return false;
    }
    if i == 0 {
        return true;
    }
    match parts[i].0 {
        Comb::Child => element_parent(el).is_some_and(|p| complex_matches(parts, i - 1, &p, ctx)),
        Comb::Descendant => {
            if el.id() == ctx.root {
                return false;
            }
            let mut cur = element_parent(el);
            while let Some(p) = cur {
                if complex_matches(parts, i - 1, &p, ctx) {
                    return true;
                }
                if p.id() == ctx.root {
                    break;
                }
                cur = element_parent(&p);
            }
            false
        }
        Comb::Next => prev_element(el).is_some_and(|p| complex_matches(parts, i - 1, &p, ctx)),
        Comb::Later => {
            let mut cur = prev_element(el);
            while let Some(p) = cur {
                if complex_matches(parts, i - 1, &p, ctx) {
                    return true;
                }
                cur = prev_element(&p);
            }
            false
        }
    }
}

fn compound_matches(c: &Compound, el: &ElementRef<'_>, ctx: Ctx) -> bool {
    c.0.iter().all(|s| simple_matches(s, el, ctx))
}

fn group_matches(g: &Group, el: &ElementRef<'_>, ctx: Ctx) -> bool {
    g.0.iter().any(|c| complex_matches(&c.0, c.0.len() - 1, el, ctx))
}

fn siblings_of<'a>(el: &ElementRef<'a>, same_type: bool) -> Vec<ElementRef<'a>> {
    let name = el.value().name();
    match el.parent() {
        Some(p) => p
            .children()
            .filter_map(ElementRef::wrap)
            .filter(|s| !same_type || s.value().name() == name)
            .collect(),
        None => vec![*el],
    }
}

fn simple_matches(s: &Simple, el: &ElementRef<'_>, ctx: Ctx) -> bool {
    match s {
        Simple::Any => true,
        Simple::Tag(t) => dom::name(el).eq_ignore_ascii_case(t),
        Simple::Id(id) => el.value().id() == Some(id.as_str()),
        Simple::Class(c) => el.value().classes().any(|k| k.eq_ignore_ascii_case(c)),
        Simple::Attr { name, op, value } => attr_matches(el, name, op, value),
        Simple::Not(g) => !group_matches(g, el, ctx),
        Simple::Is(g) => group_matches(g, el, ctx),
        Simple::Has(g) => {
            // Each relative selector starts with an implicit `:scope`, so it is anchored to this element.
            let inner = Ctx { root: ctx.root, scope: el.id() };
            let hit = |cand: &ElementRef<'_>| g.0.iter().any(|c| complex_matches(&c.0, c.0.len() - 1, cand, inner));
            el.descendants().skip(1).filter_map(ElementRef::wrap).any(|d| hit(&d))
                || following_siblings(el).any(|s| hit(&s))
        }
        Simple::Contains { needle, own, whole } => {
            let text = match (own, whole) {
                (false, false) => dom::normalise(&dom::text(el)),
                (true, false) => dom::normalise(&dom::own_text(el)),
                (false, true) => dom::whole_text(el),
                (true, true) => dom::whole_text(el),
            };
            text.contains(needle.as_str())
        }
        Simple::ContainsData(needle) => dom::data(el).to_lowercase().contains(needle.as_str()),
        Simple::Matches { re, own } => {
            let text = if *own { dom::own_text(el) } else { dom::text(el) };
            re.is_match(&text).unwrap_or(false)
        }
        Simple::Nth { a, b, last, of_type } => {
            let mut sibs = siblings_of(el, *of_type);
            if *last {
                sibs.reverse();
            }
            let Some(idx) = sibs.iter().position(|e| e.id() == el.id()) else { return false };
            let n = idx as i64 + 1;
            if *a == 0 {
                n == *b
            } else {
                let d = n - *b;
                d % *a == 0 && d / *a >= 0
            }
        }
        Simple::OnlyChild => siblings_of(el, false).len() == 1,
        Simple::OnlyOfType => siblings_of(el, true).len() == 1,
        Simple::Empty => el.children().all(|c| !matches!(c.value(), Node::Text(_) | Node::Element(_))),
        Simple::Root => el.parent().is_some_and(|p| matches!(p.value(), Node::Document)),
        Simple::Scope => el.id() == ctx.scope,
        Simple::Eq(n) => sibling_index(el) == *n,
        Simple::Lt(n) => sibling_index(el) < *n,
        Simple::Gt(n) => sibling_index(el) > *n,
    }
}

/// `:has(+ a)` and `:has(~ a)` look at the element's following siblings.
fn following_siblings<'a>(el: &ElementRef<'a>) -> impl Iterator<Item = ElementRef<'a>> {
    std::iter::successors(el.next_sibling(), |n| n.next_sibling()).filter_map(ElementRef::wrap)
}

fn sibling_index(el: &ElementRef<'_>) -> usize {
    siblings_of(el, false).iter().position(|e| e.id() == el.id()).unwrap_or(0)
}

fn attr_matches(el: &ElementRef<'_>, name: &str, op: &AttrOp, value: &str) -> bool {
    if let AttrOp::NamePrefix = op {
        return el.value().attrs().any(|(k, _)| k.to_ascii_lowercase().starts_with(name));
    }
    let Some(actual) = el.value().attr(name).or_else(|| {
        // attribute names are lower case in the parsed tree; fall back for odd cases
        el.value().attrs().find(|(k, _)| k.eq_ignore_ascii_case(name)).map(|(_, v)| v)
    }) else {
        return matches!(op, AttrOp::Ne);
    };
    let lower = actual.to_lowercase();
    match op {
        AttrOp::Exists | AttrOp::NamePrefix => true,
        AttrOp::Eq => lower == value,
        AttrOp::Ne => lower != value,
        AttrOp::Starts => lower.starts_with(value),
        AttrOp::Ends => lower.ends_with(value),
        AttrOp::Contains => lower.contains(value),
        AttrOp::Regex(re) => re.is_match(actual).unwrap_or(false),
    }
}
