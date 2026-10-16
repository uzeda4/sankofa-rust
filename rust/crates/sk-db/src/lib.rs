//! Read access to the SQLite database written by the Kotlin app (`tachiyomi.db`).
//! The Kotlin side owns the schema and its migrations; this crate only reads for now.

use rusqlite::{Connection, OpenFlags};
use sk_model::{Manga, Status};
use std::path::Path;

pub struct Db {
    conn: Connection,
}

impl Db {
    /// Opens an existing database. Fails if the file is missing instead of creating an empty one.
    pub fn open(path: impl AsRef<Path>) -> rusqlite::Result<Self> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        Ok(Self { conn })
    }

    #[cfg(test)]
    fn from_connection(conn: Connection) -> Self {
        Self { conn }
    }

    /// SQLDelight stores the number of applied migrations here.
    pub fn schema_version(&self) -> rusqlite::Result<i64> {
        self.conn.pragma_query_value(None, "user_version", |r| r.get(0))
    }

    /// Library entries (favorites), ordered like the library default: by title.
    pub fn library(&self) -> rusqlite::Result<Vec<Manga>> {
        let mut stmt = self.conn.prepare(
            "SELECT _id, source, url, title, thumbnail_url, favorite, status
             FROM mangas WHERE favorite = 1 ORDER BY title COLLATE NOCASE",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(Manga {
                id: r.get(0)?,
                source: r.get(1)?,
                url: r.get(2)?,
                title: r.get(3)?,
                thumbnail_url: r.get(4)?,
                favorite: r.get::<_, i64>(5)? != 0,
                status: Status::from_i64(r.get(6)?),
            })
        })?;
        rows.collect()
    }

    /// (total chapters, read chapters) for one manga.
    pub fn chapter_counts(&self, manga_id: i64) -> rusqlite::Result<(i64, i64)> {
        self.conn.query_row(
            "SELECT COUNT(*), COALESCE(SUM(read), 0) FROM chapters WHERE manga_id = ?1",
            [manga_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Db {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE mangas(_id INTEGER PRIMARY KEY, source INTEGER, url TEXT, title TEXT,
                thumbnail_url TEXT, favorite INTEGER, status INTEGER);
             CREATE TABLE chapters(_id INTEGER PRIMARY KEY, manga_id INTEGER, read INTEGER);
             INSERT INTO mangas VALUES (1, 7, '/a', 'beta', NULL, 1, 1), (2, 7, '/b', 'Alpha', 'http://x/c.jpg', 1, 2),
                                       (3, 7, '/c', 'gone', NULL, 0, 0);
             INSERT INTO chapters VALUES (1, 2, 1), (2, 2, 0), (3, 2, 1);
             PRAGMA user_version = 47;",
        )
        .unwrap();
        Db::from_connection(conn)
    }

    #[test]
    fn library_lists_only_favorites_sorted_by_title() {
        let db = sample();
        let titles: Vec<_> = db.library().unwrap().into_iter().map(|m| m.title).collect();
        assert_eq!(titles, ["Alpha", "beta"]);
        assert_eq!(db.schema_version().unwrap(), 47);
        assert_eq!(db.chapter_counts(2).unwrap(), (3, 2));
    }

    /// Runs against a real database copied from the Kotlin app, when one is provided.
    #[test]
    fn reads_the_real_app_database() {
        let Ok(path) = std::env::var("SK_TEST_DB") else { return };
        let db = Db::open(&path).unwrap();
        let lib = db.library().unwrap();
        println!("schema {}, {} favorites", db.schema_version().unwrap(), lib.len());
        for m in &lib {
            let (total, read) = db.chapter_counts(m.id).unwrap();
            println!("  {} [{:?}] {read}/{total}", m.title, m.status);
        }
        assert!(db.schema_version().unwrap() > 0);
    }
}
