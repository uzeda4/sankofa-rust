"""Ports the English sources of the Keiyoushi extension repository that are pure configuration of a shared theme
(Madara, MangaThemesia, ...) into Rust crates under rust/sources/, and says why it left the others out.

  python port.py --index index.pb --kei <checkout of keiyoushi/extensions-source> --sources ../../sources \
                 --theme madara [--only name1,name2] [--dry] [--report report.json]

Inputs are read, never executed. A Kotlin class is translated only when every statement in its body is one of the
known "set a constant" shapes; anything else (override fun with logic, helper classes, `when (lang)`, ...) makes the
source "manual" and is reported with the first statement that stopped the translation. Crates it wrote carry a
`.generated` marker, so rerunning replaces them and never touches a hand-written port.
"""

import argparse
import gzip
import json
import os
import re

# ------------------------------------------------------------------ the Keiyoushi index (gzip + protobuf)


def _varint(b, i):
    r = s = 0
    while True:
        c = b[i]
        i += 1
        r |= (c & 0x7F) << s
        s += 7
        if c < 0x80:
            return r, i


def _fields(b):
    i, out = 0, []
    while i < len(b):
        k, i = _varint(b, i)
        f, t = k >> 3, k & 7
        if t == 0:
            v, i = _varint(b, i)
        elif t == 2:
            n, i = _varint(b, i)
            v, i = b[i : i + n], i + n
        elif t == 1:
            v, i = b[i : i + 8], i + 8
        elif t == 5:
            v, i = b[i : i + 4], i + 4
        else:
            raise ValueError("wire type")
        out.append((f, v))
    return out


def load_index(path):
    top = _fields(gzip.decompress(open(path, "rb").read()))
    exts = []
    for f, v in top:
        if f != 101:
            continue
        for _, v1 in _fields(v):
            e = {"sources": []}
            for f2, v2 in _fields(v1):
                if f2 == 1:
                    e["name"] = v2.decode()
                elif f2 == 2:
                    e["pkg"] = v2.decode()
                elif f2 == 8:
                    s = {}
                    for f3, v3 in _fields(v2):
                        if f3 == 2:
                            s["name"] = v3.decode()
                        elif f3 == 3:
                            s["lang"] = v3.decode()
                        elif f3 == 4:
                            s["url"] = v3.decode()
                    e["sources"].append(s)
            exts.append(e)
    return exts


# ------------------------------------------------------------------ reading the Kotlin of one extension


def strip_comments(code):
    out, i, n = [], 0, len(code)
    while i < n:
        c = code[i]
        if code.startswith('"""', i):
            j = code.find('"""', i + 3)
            j = n if j < 0 else j + 3
            out.append(code[i:j])
            i = j
        elif c == '"':
            j = i + 1
            while j < n and code[j] != '"':
                j += 2 if code[j] == "\\" else 1
            out.append(code[i : j + 1])
            i = j + 1
        elif code.startswith("//", i):
            while i < n and code[i] != "\n":
                i += 1
        elif code.startswith("/*", i):
            end = code.find("*/", i)
            i = n if end < 0 else end + 2
        else:
            out.append(c)
            i += 1
    return "".join(out)


def class_body(code):
    m = re.search(r"class\s+(\w+)\s*:\s*(\w+)\(\s*\)\s*(\{)?", code)
    if not m:
        return None
    if not m.group(3):
        return m.group(1), m.group(2), ""  # no body: the theme as it is
    depth, i = 1, m.end()
    start = i
    while i < len(code) and depth:
        c = code[i]
        if code.startswith('"""', i):
            j = code.find('"""', i + 3)
            i = len(code) if j < 0 else j + 3
            continue
        if c == '"':
            i += 1
            while i < len(code) and code[i] != '"':
                i += 2 if code[i] == "\\" else 1
        elif c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
        i += 1
    return m.group(1), m.group(2), code[start : i - 1]


def statements(body):
    """Splits a class body into logical statements (continuation lines are joined)."""
    out, cur, depth = [], "", 0
    for line in body.split("\n"):
        t = line.strip()
        if not t:
            continue
        cont = depth > 0 or t.startswith((".", "?", "+", "&&", "||", "else", ")"))
        if cur and not cont:
            out.append(cur)
            cur = ""
        cur = (cur + " " + t).strip() if cur else t
        k = 0
        while k < len(t):
            ch = t[k]
            if t.startswith('"""', k):
                j = t.find('"""', k + 3)
                k = len(t) if j < 0 else j + 3
                continue
            if ch == '"':
                k += 1
                while k < len(t) and t[k] != '"':
                    k += 2 if t[k] == "\\" else 1
            elif ch in "({[":
                depth += 1
            elif ch in ")}]":
                depth -= 1
            k += 1
    if cur:
        out.append(cur)
    return out


def unescape(s):
    return re.sub(r"\\(.)", lambda m: {"n": "\n", "t": "\t"}.get(m.group(1), m.group(1)), s)


LIT = r'(?:"""[^$]*?"""|"(?:[^"\\$]|\\.)*")'
LITS = LIT + r"(?:\s*\+\s*" + LIT + r")*"


def lit_value(expr):
    """The text of a Kotlin string expression made only of literals joined by `+`, or None."""
    expr = expr.strip()
    if not re.fullmatch(LITS, expr):
        return None
    out = []
    for m in re.finditer(LIT, expr):
        t = m.group(0)
        out.append(t[3:-3] if t.startswith('"""') else unescape(t[1:-1]))
    return "".join(out)


def rs(s):
    """A Rust raw string literal for `s`."""
    h = "#"
    while '"' + h in s:
        h += "#"
    return f'r{h}"{s}"{h}'


# ------------------------------------------------------------------ the shapes every theme shares

RATE = re.compile(
    r"override fun OkHttpClient\.Builder\.configureClient\(\)\s*=\s*rateLimit\((\d+)(?:,\s*(\d+)\.seconds)?\)"
    r"(?:\s*\{\s*!it\.encodedPath\.startsWith\((" + LIT + r")\)\s*\})?"
)
DATE = re.compile(r"override val (?:chapterDateFormat|dateFormat)\s*=\s*DateTimeFormatter\.ofPattern\((" + LIT + r")(?:,\s*Locale\.(?:US|ROOT|ENGLISH))?\)")


class Parsed:
    def __init__(self):
        self.consts = []  # (name, rust type, rust expression)
        self.fns = []  # complete Rust items for the impl block
        self.uses = set()  # items to import from the theme crate
        self.sdk_uses = set()  # items to import from sk_source_sdk


def rate_limit(st, out, skip_const):
    m = RATE.fullmatch(st)
    if not m:
        return False
    permits, secs = m.group(1), m.group(2) or "1"
    out.sdk_uses.add("RateLimit")
    out.consts.append(("RATE_LIMIT", "Option<RateLimit>", f"Some(RateLimit {{ permits: {permits}, period_ms: {int(secs) * 1000}, host: String::new(), skip_path: String::new() }})"))
    if m.group(3):
        if not skip_const:
            raise ValueError("this theme cannot skip a path")
        out.consts.append(("RATE_LIMIT_SKIP_PATH", "&'static str", rs(lit_value(m.group(3)))))
    return True


# ------------------------------------------------------------------ Madara

MADARA_STR = {
    "mangaSubString": "MANGA_SUB_STRING",
    "genreDirectory": "GENRE_DIRECTORY",
    "mangaDetailsSelectorTitle": "DETAILS_TITLE",
    "mangaDetailsSelectorAuthor": "DETAILS_AUTHOR",
    "mangaDetailsSelectorArtist": "DETAILS_ARTIST",
    "mangaDetailsSelectorStatus": "DETAILS_STATUS",
    "mangaDetailsSelectorDescription": "DETAILS_DESCRIPTION",
    "mangaDetailsSelectorThumbnail": "DETAILS_THUMBNAIL",
    "mangaDetailsSelectorGenre": "DETAILS_GENRE",
    "mangaDetailsSelectorTag": "DETAILS_TAG",
    "seriesTypeSelector": "SERIES_TYPE",
    "altNameSelector": "ALT_NAME",
    "archiveUrlSelector": "ARCHIVE_URL_SELECTOR",
    "chapterUrlSelector": "CHAPTER_URL_SELECTOR",
    "chapterDateSelector": "CHAPTER_DATE_SELECTOR",
    "pageListParseSelector": "PAGE_LIST_SELECTOR",
    "filterGenresSelector": "FILTER_GENRES_SELECTOR",
    "ajaxTemplate": "AJAX_TEMPLATE",
}
MADARA_OPT = {"archiveTitleSelector": "ARCHIVE_TITLE_SELECTOR", "chapterNameSelector": "CHAPTER_NAME_SELECTOR"}
MADARA_FUN = {"archiveSelector": "ARCHIVE_SELECTOR", "chapterListSelector": "CHAPTER_LIST_SELECTOR"}
MADARA_BOOL = {
    "filterNonMangaItems": "FILTER_NON_MANGA_ITEMS",
    "supportsPostId": "SUPPORTS_POST_ID",
    "supportsLatest": "SUPPORTS_LATEST",
    "supportsFilterFetching": "SUPPORTS_FILTER_FETCHING",
}
# Properties the Rust theme has no use for (the view-count ping, the card layout of searches).
MADARA_IGNORED = {"sendViewCount", "supportsRelatedMangas"}
MADARA_IGNORED_FUN = {"searchCardSelector"}
TRIM = re.compile(r"override fun imageFromElement\(element: Element\): String\? = super\.imageFromElement\(element\)\?\.trim\(\)")


def madara(stmts, parent="Madara"):
    out = Parsed()
    if parent == "MadaraNoAjax":
        out.consts.append(("NO_AJAX", "bool", "true"))
    for st in stmts:
        m = re.fullmatch(r"override val chapterMode\s*=\s*ChapterMode\.(\w+)", st)
        if m:
            out.uses.add("ChapterMode")
            out.consts.append(("CHAPTER_MODE", "ChapterMode", f"ChapterMode::{m.group(1)}"))
            continue
        m = re.fullmatch(r"override val (\w+)(?:\s*:\s*String\??)?\s*=\s*(.+)", st)
        if m:
            name, v = m.group(1), lit_value(m.group(2))
            if v is not None and name in MADARA_STR:
                out.consts.append((MADARA_STR[name], "&'static str", rs(v)))
                continue
            if v is not None and name in MADARA_OPT:
                out.consts.append((MADARA_OPT[name], "Option<&'static str>", f"Some({rs(v)})"))
                continue
            b = re.fullmatch(r"(true|false)", m.group(2).strip())
            if b and name in MADARA_BOOL:
                out.consts.append((MADARA_BOOL[name], "bool", b.group(1)))
                continue
            if b and name in MADARA_IGNORED:
                continue
        m = re.fullmatch(r"override fun (\w+)\(\)(?:\s*:\s*String\??)?\s*=\s*(.+)", st)
        if m and lit_value(m.group(2)) is not None:
            if m.group(1) == "nextPageSelector" and parent == "MadaraNoAjax":
                out.consts.append(("NEXT_PAGE_SELECTOR", "&'static str", rs(lit_value(m.group(2)))))
                continue
            if m.group(1) in MADARA_IGNORED_FUN or (m.group(1) == "nextPageSelector"):
                continue
            if m.group(1) in MADARA_FUN:
                out.consts.append((MADARA_FUN[m.group(1)], "&'static str", rs(lit_value(m.group(2)))))
                continue
            if m.group(1) in MADARA_IGNORED_FUN:
                continue
        m = DATE.fullmatch(st)
        if m:
            out.consts.append(("CHAPTER_DATE_PATTERN", "&'static str", rs(lit_value(m.group(1)))))
            continue
        if rate_limit(st, out, skip_const=True):
            continue
        if TRIM.fullmatch(st):
            out.uses.add("default_image")
            out.sdk_uses.add("html::El")
            out.fns.append("    fn image_from_element(el: &El) -> Option<String> {\n        default_image(el).map(|s| s.trim().to_string())\n    }")
            continue
        return None, f"unsupported statement: {st[:110]}"
    return out, None


# ------------------------------------------------------------------ MangaThemesia

MTS_STR = {
    "mangaUrlDirectory": "MANGA_URL_DIRECTORY",
    "datePattern": "DATE_PATTERN",
    "seriesDetailsSelector": "DETAILS_SELECTOR",
    "seriesTitleSelector": "TITLE_SELECTOR",
    "seriesDescriptionSelector": "DESCRIPTION_SELECTOR",
    "seriesThumbnailSelector": "THUMBNAIL_SELECTOR",
    "pageSelector": "PAGE_SELECTOR",
    "projectPageString": "PROJECT_PAGE",
}
MTS_FUN_CONST = {"chapterListSelector": "CHAPTER_LIST_SELECTOR", "searchMangaSelector": "LIST_SELECTOR", "searchMangaNextPageSelector": "NEXT_PAGE_SELECTOR"}
# Selectors the Rust theme builds with a function (so a site can override them).
MTS_VAL_FN = {
    "seriesGenreSelector": "genre_selector",
    "seriesStatusSelector": "status_selector",
    "seriesTypeSelector": "type_selector",
    "seriesArtistSelector": "artist_selector",
    "seriesAuthorSelector": "author_selector",
    "seriesAltNameSelector": "alt_name_selector",
}


def mangathemesia(stmts, parent=None):
    out = Parsed()
    for st in stmts:
        m = re.fullmatch(r"override val (\w+)(?:\s*:\s*String\??)?\s*=\s*(.+)", st)
        if m:
            name, v = m.group(1), lit_value(m.group(2))
            if v is not None and name in MTS_STR:
                out.consts.append((MTS_STR[name], "&'static str", rs(v)))
                continue
            if v is not None and name in MTS_VAL_FN:
                out.fns.append(f"    fn {MTS_VAL_FN[name]}() -> String {{\n        {rs(v)}.into()\n    }}")
                continue
            if re.fullmatch(r"(true|false)", m.group(2).strip()) and name == "hasProjectPage":
                out.consts.append(("HAS_PROJECT_PAGE", "bool", m.group(2).strip()))
                continue
            if name == "sendViewCount":
                continue
        m = re.fullmatch(r"override fun (\w+)\(\)(?:\s*:\s*String\??)?\s*=\s*(.+)", st)
        if m and lit_value(m.group(2)) is not None and m.group(1) in MTS_FUN_CONST:
            out.consts.append((MTS_FUN_CONST[m.group(1)], "&'static str", rs(lit_value(m.group(2)))))
            continue
        if rate_limit(st, out, skip_const=False):
            continue
        return None, f"unsupported statement: {st[:110]}"
    return out, None


# ------------------------------------------------------------------ MangaCatalog

PAIR = re.compile(r'Pair\(\s*"((?:[^"\\]|\\.)*)"\s*,\s*"((?:[^"\\]|\\.)*)"\s*\)')


def mangacatalog(stmts, parent=None):
    out = Parsed()
    for st in stmts:
        m = re.fullmatch(r"override val sourceList\s*=\s*listOf\((.*)\)(?:\.\w+\(.*\))*", st)
        if m:
            pairs = [(unescape(a), unescape(b).replace("$baseUrl", "")) for a, b in PAIR.findall(m.group(1))]
            if not pairs or any("$" in a or "$" in b or not (b.startswith("/") or b.startswith("http")) for a, b in pairs):
                return None, f"sourceList the generator cannot read: {st[:90]}"
            body = ", ".join(f"({rs(a)}, {rs(b)})" for a, b in pairs)
            out.consts.append(("SOURCES", "&'static [(&'static str, &'static str)]", f"&[{body}]"))
            continue
        m = re.fullmatch(r"override fun chapterListSelector\(\)(?:\s*:\s*String)?\s*=\s*(.+)", st)
        if m and lit_value(m.group(1)) is not None:
            out.consts.append(("CHAPTER_LIST_SELECTOR", "&'static str", rs(lit_value(m.group(1)))))
            continue
        return None, f"unsupported statement: {st[:110]}"
    if not any(c[0] == "SOURCES" for c in out.consts):
        return None, "no sourceList"
    return out, None


# ------------------------------------------------------------------ Yiimanga


def yiimanga(stmts, parent=None):
    out = Parsed()
    for st in stmts:
        m = re.fullmatch(r"override val (\w+)(?:\s*:\s*String\??)?\s*=\s*(.+)", st)
        if m:
            name, raw = m.group(1), m.group(2).strip()
            v = lit_value(raw)
            if v is not None and name == "mangaPath":
                out.consts.append(("MANGA_PATH", "&'static str", rs(v)))
                continue
            if v is not None and name == "searchParam":
                out.consts.append(("SEARCH_PARAM", "&'static str", rs(v)))
                continue
            if v is not None and name == "popularPath":
                out.consts.append(("POPULAR_PATH", "Option<&'static str>", f"Some({rs(v)})"))
                continue
            if raw in ("true", "false") and name == "supportsFilterFetching":
                out.consts.append(("SUPPORTS_FILTER_FETCHING", "bool", raw))
                continue
        return None, f"unsupported statement: {st[:110]}"
    if not any(c[0] == "MANGA_PATH" for c in out.consts):
        return None, "no mangaPath"
    return out, None


THEMES = {
    "yiimanga": dict(bases={"Yiimanga"}, parse=yiimanga, crate="sk-theme-yiimanga", mod="sk_theme_yiimanga", label="A Yiimanga site"),
    "madara": dict(bases={"Madara", "MadaraNoAjax"}, parse=madara, crate="sk-theme-madara", mod="sk_theme_madara", label="A Madara site"),
    "mangacatalog": dict(bases={"MangaCatalog"}, parse=mangacatalog, crate="sk-theme-mangacatalog", mod="sk_theme_mangacatalog", label="A MangaCatalog site"),
    "mangathemesia": dict(bases={"MangaThemesia"}, parse=mangathemesia, crate="sk-theme-mangathemesia", mod="sk_theme_mangathemesia", label="A MangaThemesia site"),
}

# ------------------------------------------------------------------ writing a crate


def emit(theme, name, struct, tail, lang_dir, base_url, p):
    t = THEMES[theme]
    uses = sorted(p.uses | {"Site", "Theme"})
    lines = [
        f"//! {name}, written in Rust. {t['label']}: everything comes from `{t['crate']}`.",
        f"//! Port of the Keiyoushi extension `{lang_dir}.{tail}` (Apache-2.0).",
        "",
        "use sk_source_sdk::export_source;",
    ]
    for u in sorted(p.sdk_uses):
        lines.append(f"use sk_source_sdk::{u};")
    lines.append(f"use {t['mod']}::{{{', '.join(uses)}}};")
    lines += ["", f"pub struct {struct};", "", f"impl Theme for {struct} {{"]
    lines.append(f"    const NAME: &'static str = {rs(name)};")
    lines.append(f"    const BASE_URL: &'static str = {rs(base_url)};")
    for k, ty, v in p.consts:
        lines.append(f"    const {k}: {ty} = {v};")
    for f in p.fns:
        lines += ["", f]
    lines += ["}", "", f"export_source!(Site<{struct}>);", ""]
    return "\n".join(lines)


def camel(s):
    return "".join(x[:1].upper() + x[1:] for x in re.split(r"[^A-Za-z0-9]+", s) if x) or "Source"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--index", required=True)
    ap.add_argument("--kei", required=True)
    ap.add_argument("--sources", required=True)
    ap.add_argument("--theme", required=True, choices=sorted(THEMES))
    ap.add_argument("--only", default="")
    ap.add_argument("--report", default="")
    ap.add_argument("--dry", action="store_true", help="only report, write no crate")
    a = ap.parse_args()
    t = THEMES[a.theme]
    only = {x for x in a.only.split(",") if x}
    done, manual = [], []
    for e in load_index(a.index):
        en = [s for s in e["sources"] if s.get("lang") == "en"]
        if not en:
            continue
        lang_dir, tail = e["pkg"].split(".")[-2], e["pkg"].split(".")[-1]
        kdir = os.path.join(a.kei, "src", lang_dir, tail)
        gradle = os.path.join(kdir, "build.gradle.kts")
        if not os.path.exists(gradle):
            continue
        g = open(gradle, encoding="utf-8", errors="ignore").read()
        th = re.search(r'theme\s*=\s*"(\w+)"', g)
        if not th or th.group(1) != a.theme or (only and tail not in only):
            continue
        s0 = en[0]
        item = {"dir": tail, "name": s0["name"]}
        code = ""
        for root, _, files in os.walk(kdir):
            for f in files:
                if f.endswith(".kt"):
                    code += open(os.path.join(root, f), encoding="utf-8", errors="ignore").read() + "\n"
        cb = class_body(strip_comments(code))
        if not cb:
            manual.append({**item, "why": "no `class X : Theme()` found"})
            continue
        cls, parent, body = cb
        if parent not in t["bases"]:
            manual.append({**item, "why": f"extends {parent}, not {sorted(t['bases'])}"})
            continue
        try:
            parsed, why = t["parse"](statements(body), parent)
        except ValueError as err:
            parsed, why = None, str(err)
        if why:
            manual.append({**item, "why": why})
            continue
        slug = re.sub(r"[^a-z0-9]", "", tail.lower())
        crate = os.path.join(a.sources, slug)
        if os.path.isdir(crate) and not os.path.exists(os.path.join(crate, ".generated")):
            manual.append({**item, "why": "a hand-written port already exists"})
            continue
        rec = {"dir": slug, "name": s0["name"], "base": s0["url"], "theme": a.theme}
        if not a.dry:
            os.makedirs(os.path.join(crate, "src"), exist_ok=True)
            with open(os.path.join(crate, ".generated"), "w") as f:
                f.write("written by tools/port/port.py; rerun it instead of editing\n")
            with open(os.path.join(crate, "Cargo.toml"), "w", encoding="utf-8", newline="\n") as f:
                f.write(
                    f'[package]\nname = "sk-source-{slug}"\nversion.workspace = true\nedition.workspace = true\nlicense.workspace = true\n\n'
                    f'[lib]\ncrate-type = ["cdylib", "rlib"]\n\n[dependencies]\n'
                    f'sk-source-sdk = {{ version = "0.1.0", path = "../../crates/sk-source-sdk" }}\n'
                    f'{t["crate"]} = {{ version = "0.1.0", path = "../../crates/{t["crate"]}" }}\n'
                )
            with open(os.path.join(crate, "src", "lib.rs"), "w", encoding="utf-8", newline="\n") as f:
                f.write(emit(a.theme, s0["name"], camel(cls), tail, lang_dir, s0["url"].rstrip("/"), parsed))
            icon = os.path.join(kdir, "res", "mipmap-xxxhdpi", "ic_launcher.png")
            if os.path.exists(icon):
                with open(os.path.join(crate, "icon.png"), "wb") as f:
                    f.write(open(icon, "rb").read())
            rec["icon"] = os.path.exists(icon)
        done.append(rec)
    if a.report:
        json.dump({"done": done, "manual": manual}, open(a.report, "w", encoding="utf-8"), indent=1)
    print(f"{a.theme}: {len(done)} ported, {len(manual)} left for hand work")
    reasons = {}
    for m in manual:
        k = re.sub(r"\d+", "N", m["why"])[:72]
        reasons[k] = reasons.get(k, 0) + 1
    for k, v in sorted(reasons.items(), key=lambda x: -x[1])[:12]:
        print(f"  {v:3d}  {k}")


if __name__ == "__main__":
    main()
