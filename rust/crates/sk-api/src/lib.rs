//! The surface the Kotlin app sees: one small, blocking API over the `sk-*` crates, exported with UniFFI.
//! Call everything from a background dispatcher; nothing here touches the UI.

uniffi::setup_scaffolding!();

mod sources;

#[derive(Debug, uniffi::Error)]
pub enum SkError {
    Db { reason: String },
    Net { reason: String },
    Package { reason: String },
}

impl std::fmt::Display for SkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Db { reason } | Self::Net { reason } | Self::Package { reason } => f.write_str(reason),
        }
    }
}

#[derive(Debug, uniffi::Record)]
pub struct LibraryEntry {
    pub id: i64,
    pub title: String,
    /// Same numeric values as `SManga` (0 unknown, 1 ongoing, 2 completed, ...).
    pub status: u8,
    pub chapters_total: i64,
    pub chapters_read: i64,
}

#[derive(Debug, uniffi::Record)]
pub struct CoreInfo {
    pub version: String,
    /// `PRAGMA user_version` of the database, i.e. how many migrations the Kotlin side applied.
    pub schema_version: i64,
    pub favorites: u32,
}

/// Version of the Rust core, so the app can show which native library it loaded.
#[uniffi::export]
pub fn core_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// The library (favorites) read straight from the app's database file, by title.
#[uniffi::export]
pub fn library(db_path: String) -> Result<Vec<LibraryEntry>, SkError> {
    let db = sk_db::Db::open(&db_path).map_err(db_err)?;
    db.library()
        .map_err(db_err)?
        .into_iter()
        .map(|m| {
            let (chapters_total, chapters_read) = db.chapter_counts(m.id).map_err(db_err)?;
            Ok(LibraryEntry { id: m.id, title: m.title, status: m.status as u8, chapters_total, chapters_read })
        })
        .collect()
}

#[uniffi::export]
pub fn core_info(db_path: String) -> Result<CoreInfo, SkError> {
    let db = sk_db::Db::open(&db_path).map_err(db_err)?;
    Ok(CoreInfo {
        version: core_version(),
        schema_version: db.schema_version().map_err(db_err)?,
        favorites: db.library().map_err(db_err)?.len() as u32,
    })
}

/// GET a page as text. `headers` are `(name, value)` pairs, e.g. a `Referer`.
#[uniffi::export]
pub fn fetch_text(url: String, headers: Vec<Header>) -> Result<String, SkError> {
    let pairs: Vec<(&str, &str)> = headers.iter().map(|h| (h.name.as_str(), h.value.as_str())).collect();
    let bytes = sk_net::Client::new().get_bytes_with(&url, &pairs).map_err(|e| SkError::Net { reason: e.to_string() })?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

#[derive(Debug, uniffi::Record)]
pub struct Header {
    pub name: String,
    pub value: String,
}

fn db_err(e: sk_db::Error) -> SkError {
    SkError::Db { reason: e.to_string() }
}

