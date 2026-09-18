//! The application database facade.
//!
//! Owns the raw `rusqlite::Connection` and exposes repositories. Every
//! database access goes through this type — no module executes arbitrary SQL
//! outside the repository layer.

use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;
use tracing::info;

use crate::db;
use crate::errors::AppError;

use super::models::{BibleBook, Translation, Verse};

/// Wrapper around a single long-lived SQLite connection.
pub struct Database {
    pub inner: Mutex<Connection>,
}

impl Database {
    /// Opens the database at `path`, creating it and applying migrations if
    /// this is the first run.
    pub fn open(path: &Path) -> Result<Self, crate::AppError> {
        let conn = db::open_connection(path)?;
        db::migrations::run_migrations(&conn)?;
        info!("database initialized at {}", path.display());
        Ok(Self {
            inner: Mutex::new(conn),
        })
    }

    /// Runs `f` with exclusive access to the connection.
    pub fn with_conn<T>(
        &self,
        f: impl FnOnce(&Connection) -> Result<T, AppError>,
    ) -> Result<T, AppError> {
        let conn = self
            .inner
            .lock()
            .map_err(|_| AppError::Internal("database connection lock poisoned".to_string()))?;
        f(&conn)
    }

    /// Lists every book from the canonical registry.
    pub fn list_books(&self) -> Result<Vec<BibleBook>, AppError> {
        self.with_conn(|conn| crate::bible::repository::BibleRepository::new(conn).list_books())
    }

    /// The default translation, if one has been flagged.
    pub fn default_translation(&self) -> Result<Option<Translation>, AppError> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, language, abbreviation, is_default, created_at
                 FROM translations WHERE is_default = 1 LIMIT 1",
            )?;
            let mut rows = stmt.query_map([], row_to_translation)?;
            Ok(rows.next().transpose()?)
        })
    }

    /// Resolves a (possibly abbreviated / alias) book name from the books
    /// table. Returns `None` when the name is unknown.
    pub fn find_book_by_name(&self, name: &str) -> Result<Option<BibleBook>, AppError> {
        self.with_conn(|conn| {
            crate::bible::repository::BibleRepository::new(conn).find_book_by_name(name)
        })
    }

    /// Number of verses stored for a translation. Used by the UI to hint at
    /// whether an installed translation has content.
    pub fn verse_count(&self, translation_id: &str) -> Result<i64, AppError> {
        self.with_conn(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM verses WHERE translation_id = ?1",
                [translation_id],
                |row| row.get(0),
            )
            .map_err(Into::into)
        })
    }
}

impl Verse {
    /// The label of the verse, e.g. "John 3:16". Book names come from the
    /// deterministic scripture registry, never from verse data itself.
    pub fn label(&self) -> String {
        let book = crate::scripture::books::book_by_id(self.book_id)
            .map(|b| b.name.to_string())
            .unwrap_or_else(|| format!("#{}", self.book_id));
        format!("{book} {}:{}", self.chapter, self.verse)
    }
}

fn row_to_translation(row: &rusqlite::Row<'_>) -> rusqlite::Result<Translation> {
    Ok(Translation {
        id: row.get(0)?,
        name: row.get(1)?,
        language: row.get(2)?,
        abbreviation: row.get(3)?,
        is_default: row.get::<_, i64>(4)? != 0,
        created_at: row.get(5)?,
    })
}
