//! Bible translation import (the seed/import mechanism).
//!
//! No Bible text ships with the application. Everything is offline: an
//! operator imports a translation they have legal rights to distribute from
//! a JSON file. The documented format lives in `/data/bible/README.md`.

use std::fs;

use rusqlite::Connection;
use serde::Deserialize;

use crate::errors::AppError;

use super::models::TranslationInfo;
use super::repository::BibleRepository;

/// The accepted JSON envelope for a translation import.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportDocument {
    pub translation: TranslationInfo,
    /// Flat list of verses. Book ids refer to the canonical registry.
    #[serde(default)]
    pub verses: Vec<ImportVerse>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportVerse {
    pub book_id: i32,
    pub chapter: u32,
    pub verse: u32,
    pub text: String,
}

/// Parses an import document from a UTF-8 JSON file on disk.
pub fn parse_import(path: &std::path::Path) -> Result<ImportDocument, AppError> {
    let raw =
        fs::read(path).map_err(|e| AppError::Media(format!("cannot read import file: {e}")))?;
    let doc: ImportDocument = serde_json::from_slice(&raw).map_err(|e| {
        AppError::InvalidConfiguration(format!("invalid translation import JSON: {e}"))
    })?;
    if doc.translation.id.trim().is_empty() || doc.translation.name.trim().is_empty() {
        return Err(AppError::InvalidConfiguration(
            "translation import requires a non-empty id and name".to_string(),
        ));
    }
    Ok(doc)
}

/// Imports an entire translation document inside a transaction.
pub fn apply_import(conn: &Connection, doc: &ImportDocument) -> Result<(usize, String), AppError> {
    let tx = conn.unchecked_transaction()?;
    {
        // Validate every book id before mutating anything.
        let mut stmt = tx.prepare("SELECT 1 FROM books WHERE id = ?1")?;
        for v in &doc.verses {
            let found: bool = stmt
                .query_row([v.book_id], |row| row.get::<_, i64>(0))
                .map(|n| n == 1)
                .unwrap_or(false);
            if !found {
                return Err(AppError::InvalidScriptureReference(format!(
                    "unknown book id {} in import",
                    v.book_id
                )));
            }
        }
    }

    {
        let repo = BibleRepository::new(&tx);
        repo.upsert_translation(&doc.translation)?;
        for v in &doc.verses {
            repo.insert_verse(&super::models::Verse {
                translation_id: doc.translation.id.clone(),
                book_id: v.book_id,
                chapter: v.chapter as i32,
                verse: v.verse as i32,
                text: v.text.clone(),
            })?;
        }
    }

    let count = doc.verses.len();
    tx.commit()?;
    Ok((count, doc.translation.id.clone()))
}

/// Table shape Selah expects inside an imported SQLite Bible file.
const SOURCE_TABLE: &str = "verses";

/// Highest book id in the canonical registry (66 books).
const MAX_BOOK_ID: i64 = 66;

/// Result of a SQLite translation import.
#[derive(Debug, Clone)]
pub struct SqliteImportOutcome {
    pub verses_imported: usize,
    pub translation_id: String,
}

/// Imports a translation from a SQLite file.
///
/// The file must expose a `verses` table shaped like the common public
/// downloads:
///
/// ```sql
/// CREATE TABLE verses (
///   book_id INTEGER,   -- 1..=66, matching Selah's canonical book registry
///   chapter INTEGER,
///   number  INTEGER,   -- the verse number
///   text    TEXT
/// );
/// ```
///
/// The whole copy runs inside one transaction as a single
/// `INSERT … SELECT`, so SQLite moves all ~31 000 verses in C rather than
/// through a Rust loop. The source database is attached read-only and
/// detached again, so it is never modified.
pub fn import_sqlite_translation(
    conn: &Connection,
    path: &std::path::Path,
    info: &TranslationInfo,
) -> Result<SqliteImportOutcome, AppError> {
    let path_str = path.to_string_lossy().to_string();

    // `mode=ro` keeps the operator's file untouched; the URI form also stops
    // SQLite creating a journal beside it.
    conn.execute(
        "ATTACH DATABASE ?1 AS source",
        [format!("file:{path_str}?mode=ro")],
    )
    .map_err(|e| {
        AppError::Media(format!(
            "cannot open {} as a Bible database: {e}",
            path.display()
        ))
    })?;

    // Everything after the attach must detach again, whatever happens.
    let result = copy_verses(conn, info);
    let _ = conn.execute("DETACH DATABASE source", []);

    result
}

/// Copies every row of the attached `verses` table into Selah's schema.
fn copy_verses(conn: &Connection, info: &TranslationInfo) -> Result<SqliteImportOutcome, AppError> {
    ensure_source_is_usable(conn)?;

    let tx = conn.unchecked_transaction()?;
    {
        let repo = BibleRepository::new(&tx);
        repo.upsert_translation(info)?;
    }

    // Verse numbers arrive in a column named `number`; Selah's column is
    // `verse`. Book ids are shared, so no mapping table is needed.
    let inserted = tx.execute(
        "INSERT INTO verses (translation_id, book_id, chapter, verse, text)
         SELECT ?1, book_id, chapter, number, text FROM source.verses
         WHERE book_id BETWEEN 1 AND ?2 AND chapter >= 1 AND number >= 1
         ON CONFLICT(translation_id, book_id, chapter, verse)
         DO UPDATE SET text = excluded.text",
        rusqlite::params![info.id, MAX_BOOK_ID],
    )?;

    tx.commit()?;

    tracing::info!(
        translation = %info.id,
        verses = inserted,
        "bible translation imported from sqlite"
    );

    Ok(SqliteImportOutcome {
        verses_imported: inserted,
        translation_id: info.id.clone(),
    })
}

/// Verifies the attached file really holds the expected table and that its
/// book ids fit the canonical registry.
fn ensure_source_is_usable(conn: &Connection) -> Result<(), AppError> {
    let table_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM source.sqlite_master
                           WHERE type = 'table' AND name = ?1)",
            [SOURCE_TABLE],
            |row| row.get(0),
        )
        .map_err(|e| {
            AppError::InvalidConfiguration(format!("not a readable Bible database: {e}"))
        })?;

    if !table_exists {
        return Err(AppError::InvalidConfiguration(format!(
            "the file has no `{SOURCE_TABLE}` table, so it is not a Bible database \
             Selah can read"
        )));
    }

    for column in ["book_id", "chapter", "number", "text"] {
        let has_column: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM pragma_table_info('verses', 'source') WHERE name = ?1)",
            [column],
            |row| row.get(0),
        )?;
        if !has_column {
            return Err(AppError::InvalidConfiguration(format!(
                "the `{SOURCE_TABLE}` table is missing a `{column}` column"
            )));
        }
    }

    let out_of_range: i64 = conn.query_row(
        "SELECT COUNT(*) FROM source.verses WHERE book_id < 1 OR book_id > ?1",
        [MAX_BOOK_ID],
        |row| row.get(0),
    )?;
    if out_of_range > 0 {
        return Err(AppError::InvalidConfiguration(format!(
            "the file contains {out_of_range} rows whose book number is outside 1–{MAX_BOOK_ID}"
        )));
    }

    let total: i64 = conn.query_row("SELECT COUNT(*) FROM source.verses", [], |row| row.get(0))?;
    if total == 0 {
        return Err(AppError::InvalidConfiguration(
            "the Bible database is empty".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use std::path::Path;

    /// Creates a source database shaped like the published downloads.
    fn write_source(dir: &Path, name: &str, rows: &[(i32, i32, i32, &str)]) -> std::path::PathBuf {
        let path = dir.join(name);
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            "CREATE TABLE verses (
                 id      INTEGER PRIMARY KEY AUTOINCREMENT,
                 book_id INTEGER NOT NULL,
                 chapter INTEGER NOT NULL,
                 number  INTEGER NOT NULL,
                 text    TEXT    NOT NULL
             );",
        )
        .unwrap();
        for (book, chapter, verse, text) in rows {
            conn.execute(
                "INSERT INTO verses (book_id, chapter, number, text) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![book, chapter, verse, text],
            )
            .unwrap();
        }
        path
    }

    /// Selah's own database, with the canonical books already present.
    fn memory_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        db::migrations::run_migrations(&conn).unwrap();
        conn
    }

    fn translation(id: &str, is_default: bool) -> TranslationInfo {
        TranslationInfo {
            id: id.to_string(),
            name: format!("{id} translation"),
            language: "en".to_string(),
            abbreviation: Some(id.to_uppercase()),
            is_default,
        }
    }

    /// Unique scratch directory for a test.
    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("selah-import-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn imports_every_verse_from_a_sqlite_bible() {
        let dir = scratch("basic");
        let source = write_source(
            &dir,
            "web.sqlite",
            &[
                (43, 3, 16, "For God so loved the world"),
                (43, 3, 17, "For God sent not his Son"),
                (19, 23, 1, "The Lord is my shepherd"),
            ],
        );

        let conn = memory_db();
        let outcome = import_sqlite_translation(&conn, &source, &translation("web", true)).unwrap();

        assert_eq!(outcome.verses_imported, 3);
        assert_eq!(BibleRepository::new(&conn).verse_count("web").unwrap(), 3);

        // The verse text and numbering must survive the copy.
        let stored: String = conn
            .query_row(
                "SELECT text FROM verses WHERE translation_id = 'web' AND book_id = 43
                 AND chapter = 3 AND verse = 16",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(stored, "For God so loved the world");

        // The translation row itself is registered and marked default.
        let registered = BibleRepository::new(&conn)
            .get_translation("web")
            .unwrap()
            .expect("translation should exist");
        assert!(registered.is_default);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn rejects_a_file_that_is_not_a_bible() {
        let dir = scratch("not-bible");
        let source = dir.join("random.sqlite");
        let conn = Connection::open(&source).unwrap();
        conn.execute_batch("CREATE TABLE notes (body TEXT);")
            .unwrap();
        drop(conn);

        let conn = memory_db();
        let err = import_sqlite_translation(&conn, &source, &translation("x", false)).unwrap_err();
        assert!(err.to_string().contains("verses"), "got: {err}");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn rejects_book_numbers_outside_the_canon() {
        let dir = scratch("out-of-range");
        let source = write_source(&dir, "odd.sqlite", &[(99, 1, 1, "nowhere")]);

        let conn = memory_db();
        let err =
            import_sqlite_translation(&conn, &source, &translation("odd", false)).unwrap_err();
        assert!(err.to_string().contains("book number"), "got: {err}");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn re_importing_updates_in_place() {
        let dir = scratch("reimport");
        let source = write_source(&dir, "web.sqlite", &[(43, 3, 16, "first wording")]);
        let conn = memory_db();
        import_sqlite_translation(&conn, &source, &translation("web", false)).unwrap();

        // Same translation id, corrected text.
        let corrected = write_source(&dir, "corrected.sqlite", &[(43, 3, 16, "second wording")]);
        let outcome =
            import_sqlite_translation(&conn, &corrected, &translation("web", false)).unwrap();

        assert_eq!(outcome.verses_imported, 1);
        assert_eq!(BibleRepository::new(&conn).verse_count("web").unwrap(), 1);
        let stored: String = conn
            .query_row(
                "SELECT text FROM verses WHERE translation_id = 'web' AND book_id = 43
                 AND chapter = 3 AND verse = 16",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(stored, "second wording");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn imported_verses_are_searchable() {
        // The search index is kept current by triggers, so a bulk copy has to
        // feed it too — otherwise search silently returns nothing.
        let dir = scratch("search");
        let source = write_source(
            &dir,
            "web.sqlite",
            &[(19, 23, 1, "The Lord is my shepherd")],
        );
        let conn = memory_db();
        import_sqlite_translation(&conn, &source, &translation("web", true)).unwrap();

        let hits = BibleRepository::new(&conn)
            .search("web", "shepherd", 10)
            .unwrap();
        assert_eq!(hits.len(), 1);
        assert!(hits[0].text.contains("shepherd"));

        std::fs::remove_dir_all(&dir).ok();
    }
}
