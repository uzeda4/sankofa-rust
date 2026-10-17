//! Search filters: the schema a source declares and the state the user picked.
//!
//! The source returns a `Vec<FilterDef>` from `Source::filters`; the app draws them and sends back the picks as
//! [`Filters`], keyed by each filter's `key`:
//!
//! | filter | state |
//! |---|---|
//! | text | the string |
//! | select | index of the chosen option |
//! | checkbox | bool |
//! | tristate | 0 ignore, 1 include, 2 exclude |
//! | group | map of item key to the item's state |
//! | sort | `{"index": n, "ascending": bool}` |

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FilterDef {
    Header { name: String },
    Separator,
    Text { key: String, name: String, #[serde(default)] default: String },
    /// `options` are the labels shown; the state is the index.
    Select { key: String, name: String, options: Vec<String>, default: usize },
    Checkbox { key: String, name: String, #[serde(default)] default: bool },
    Tristate { key: String, name: String },
    /// A titled list of checkboxes or tristates (genres, say).
    Group { key: String, name: String, items: Vec<FilterDef> },
    Sort { key: String, name: String, options: Vec<String>, default_index: usize, default_ascending: bool },
}

impl FilterDef {
    pub fn header(name: &str) -> Self {
        Self::Header { name: name.into() }
    }

    pub fn text(key: &str, name: &str) -> Self {
        Self::Text { key: key.into(), name: name.into(), default: String::new() }
    }

    /// `options` are `(label, value)` pairs, the same slice later given to [`Filters::select_value`].
    pub fn select(key: &str, name: &str, options: &[(&str, &str)], default: usize) -> Self {
        Self::Select { key: key.into(), name: name.into(), options: options.iter().map(|o| o.0.to_string()).collect(), default }
    }

    pub fn checkbox(key: &str, name: &str) -> Self {
        Self::Checkbox { key: key.into(), name: name.into(), default: false }
    }

    pub fn tristate(key: &str, name: &str) -> Self {
        Self::Tristate { key: key.into(), name: name.into() }
    }

    /// A group of checkboxes; `items` are `(label, key)`.
    pub fn checkboxes(key: &str, name: &str, items: &[(String, String)]) -> Self {
        Self::Group {
            key: key.into(),
            name: name.into(),
            items: items.iter().map(|(label, k)| Self::checkbox(k, label)).collect(),
        }
    }

    /// A group of include/exclude tristates; `items` are `(label, key)`.
    pub fn tristates(key: &str, name: &str, items: &[(String, String)]) -> Self {
        Self::Group {
            key: key.into(),
            name: name.into(),
            items: items.iter().map(|(label, k)| Self::tristate(k, label)).collect(),
        }
    }

    pub fn sort(key: &str, name: &str, options: &[(&str, &str)], default_index: usize, default_ascending: bool) -> Self {
        Self::Sort {
            key: key.into(),
            name: name.into(),
            options: options.iter().map(|o| o.0.to_string()).collect(),
            default_index,
            default_ascending,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tri {
    Ignore,
    Include,
    Exclude,
}

/// What the user picked. Anything missing reads as the filter's default (empty, first option, unchecked).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(transparent)]
pub struct Filters(pub Map<String, Value>);

impl Filters {
    pub fn text(&self, key: &str) -> &str {
        self.0.get(key).and_then(Value::as_str).unwrap_or("").trim()
    }

    pub fn index(&self, key: &str) -> usize {
        self.0.get(key).and_then(Value::as_u64).unwrap_or(0) as usize
    }

    /// The value of the chosen option of a select filter whose options were `options` (`(label, value)`).
    pub fn select_value<'a>(&self, key: &str, options: &'a [(&'a str, &'a str)]) -> &'a str {
        options.get(self.index(key)).map(|o| o.1).unwrap_or("")
    }

    pub fn checked(&self, key: &str) -> bool {
        self.0.get(key).and_then(Value::as_bool).unwrap_or(false)
    }

    pub fn tri(&self, key: &str) -> Tri {
        tri(self.0.get(key))
    }

    /// The state of a group's items as a `Filters` of their own, to read nested selects and texts by key.
    pub fn group_state(&self, group: &str) -> Filters {
        match self.0.get(group) {
            Some(Value::Object(items)) => Filters(items.clone()),
            _ => Filters::default(),
        }
    }

    /// Keys of a group's items that are checked or included.
    pub fn included(&self, group: &str) -> Vec<&str> {
        self.group(group, |t| t == Tri::Include)
    }

    /// Keys of a group's items that are excluded.
    pub fn excluded(&self, group: &str) -> Vec<&str> {
        self.group(group, |t| t == Tri::Exclude)
    }

    fn group(&self, group: &str, want: impl Fn(Tri) -> bool) -> Vec<&str> {
        let Some(Value::Object(items)) = self.0.get(group) else { return Vec::new() };
        items.iter().filter(|(_, v)| want(tri(Some(v)))).map(|(k, _)| k.as_str()).collect()
    }

    /// `(option index, ascending)` of a sort filter, if the user touched it.
    pub fn sort(&self, key: &str) -> Option<(usize, bool)> {
        let v = self.0.get(key)?;
        Some((v.get("index")?.as_u64()? as usize, v.get("ascending").and_then(Value::as_bool).unwrap_or(false)))
    }
}

/// Checkbox `true` counts as included, so one helper serves both kinds of group.
fn tri(v: Option<&Value>) -> Tri {
    match v {
        Some(Value::Bool(true)) => Tri::Include,
        Some(Value::Number(n)) => match n.as_u64() {
            Some(1) => Tri::Include,
            Some(2) => Tri::Exclude,
            _ => Tri::Ignore,
        },
        _ => Tri::Ignore,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_reads_back_with_defaults() {
        let f: Filters = serde_json::from_str(
            r#"{"author":" Jane ","status":2,"adult":true,"genres":{"action":1,"drama":2,"romance":0,"comedy":true},"sort":{"index":3,"ascending":true}}"#,
        )
        .unwrap();
        const STATUS: [(&str, &str); 3] = [("All", ""), ("Ongoing", "ongoing"), ("Completed", "completed")];
        assert_eq!(f.text("author"), "Jane");
        assert_eq!(f.text("missing"), "");
        assert_eq!(f.select_value("status", &STATUS), "completed");
        assert_eq!(f.select_value("absent", &STATUS), "");
        assert!(f.checked("adult") && !f.checked("nothing"));
        let mut inc = f.included("genres");
        inc.sort();
        assert_eq!(inc, ["action", "comedy"]);
        assert_eq!(f.excluded("genres"), ["drama"]);
        assert_eq!(f.tri("nope"), Tri::Ignore);
        assert_eq!(f.sort("sort"), Some((3, true)));
        assert_eq!(f.sort("none"), None);
    }

    #[test]
    fn schema_serialises_with_a_type_tag() {
        let defs = vec![
            FilterDef::select("status", "Status", &[("All", ""), ("Ongoing", "ongoing")], 0),
            FilterDef::tristates("genres", "Genres", &[("Action".into(), "action".into())]),
        ];
        let json = serde_json::to_string(&defs).unwrap();
        assert!(json.contains(r#""type":"select""#) && json.contains(r#""options":["All","Ongoing"]"#), "{json}");
        assert!(json.contains(r#""type":"group""#) && json.contains(r#""type":"tristate""#), "{json}");
    }
}
