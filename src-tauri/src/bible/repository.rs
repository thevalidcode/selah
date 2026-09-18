//! Persistence repositories.
//!
//! All SQL lives here. Application services call repositories; they never
//! embed SQL themselves.

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::errors::AppError;
use crate::media::MediaItem;
use crate::models::presentation::{Presentation, PresentationItemRecord};
use crate::scripture::reference::ScriptureReference;

use super::models::{BibleBook, Passage, Translation, TranslationInfo, Verse};

/// Access to translation + verse data.
pub struct BibleRepository<'a> {
    conn: &'a Connection,
}

impl<'a> BibleRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Every book in canonical order, with chapter counts filled from the
    /// deterministic Scripture registry.
    pub fn list_books(&self) -> Result<Vec<BibleBook>, AppError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, testament, abbreviation FROM books ORDER BY id")?;
        let rows = stmt.query_map([], |row| {
            Ok(BibleBook::new(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
            ))
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Looks a book up by its exact display name (case-insensitive).
    pub fn find_book_by_name(&self, name: &str) -> Result<Option<BibleBook>, AppError> {
        self.conn
            .query_row(
                "SELECT id, name, testament, abbreviation FROM books
                 WHERE lower(name) = lower(?1) LIMIT 1",
                [name],
                |row| {
                    Ok(BibleBook::new(
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                    ))
                },
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_translations(&self) -> Result<Vec<Translation>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, language, abbreviation, is_default, created_at
             FROM translations ORDER BY is_default DESC, name",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Translation {
                id: row.get(0)?,
                name: row.get(1)?,
                language: row.get(2)?,
                abbreviation: row.get(3)?,
                is_default: row.get::<_, i64>(4)? != 0,
                created_at: row.get(5)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn get_translation(&self, id: &str) -> Result<Option<Translation>, AppError> {
        self.conn
            .query_row(
                "SELECT id, name, language, abbreviation, is_default, created_at
                 FROM translations WHERE id = ?1",
                [id],
                |row| {
                    Ok(Translation {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        language: row.get(2)?,
                        abbreviation: row.get(3)?,
                        is_default: row.get::<_, i64>(4)? != 0,
                        created_at: row.get(5)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    /// Registers translation metadata. Does not require verse data yet.
    pub fn upsert_translation(&self, info: &TranslationInfo) -> Result<(), AppError> {
        self.conn.execute(
            "INSERT INTO translations (id, name, language, abbreviation, is_default, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                language = excluded.language,
                abbreviation = excluded.abbreviation,
                is_default = excluded.is_default",
            params![
                info.id,
                info.name,
                info.language,
                info.abbreviation,
                info.is_default as i64,
                Utc::now().to_rfc3339()
            ],
        )?;

        if info.is_default {
            self.conn.execute(
                "UPDATE translations SET is_default = 0 WHERE id != ?1",
                [&info.id],
            )?;
        }
        Ok(())
    }

    pub fn set_default_translation(&self, id: &str) -> Result<(), AppError> {
        let affected = self
            .conn
            .execute("UPDATE translations SET is_default = 1 WHERE id = ?1", [id])?;
        if affected == 0 {
            return Err(AppError::BibleTranslationNotFound(id.to_string()));
        }
        self.conn.execute(
            "UPDATE translations SET is_default = 0 WHERE id != ?1",
            [id],
        )?;
        Ok(())
    }

    /// Inserts a single verse. Used by the JSON import path inside a
    /// transaction.
    pub fn insert_verse(&self, verse: &Verse) -> Result<(), AppError> {
        self.conn.execute(
            "INSERT INTO verses (translation_id, book_id, chapter, verse, text)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(translation_id, book_id, chapter, verse)
             DO UPDATE SET text = excluded.text",
            params![
                verse.translation_id,
                verse.book_id,
                verse.chapter,
                verse.verse,
                verse.text
            ],
        )?;
        Ok(())
    }

    /// Fetches a single verse.
    pub fn get_verse(
        &self,
        translation_id: &str,
        reference: &ScriptureReference,
        verse: u16,
    ) -> Result<Option<Verse>, AppError> {
        self.conn
            .query_row(
                "SELECT translation_id, book_id, chapter, verse, text FROM verses
                 WHERE translation_id = ?1 AND book_id = ?2 AND chapter = ?3 AND verse = ?4",
                params![
                    translation_id,
                    reference.book_id,
                    reference.chapter,
                    verse as i64
                ],
                map_verse,
            )
            .optional()
            .map_err(Into::into)
    }

    /// Fetches a whole chapter or a verse range.
    pub fn get_passage(
        &self,
        translation_id: &str,
        reference: &ScriptureReference,
    ) -> Result<Passage, AppError> {
        let (start_verse, end_verse) = match (reference.start_verse, reference.end_verse) {
            (Some(start), Some(end)) => (start, end),
            (Some(start), None) => (start, start),
            (None, _) => (1, u16::MAX),
        };
        if end_verse < start_verse {
            return Err(AppError::InvalidScriptureReference(format!(
                "end verse {end_verse} before start verse {start_verse}"
            )));
        }

        let mut stmt = self.conn.prepare(
            "SELECT translation_id, book_id, chapter, verse, text FROM verses
             WHERE translation_id = ?1 AND book_id = ?2 AND chapter = ?3
               AND verse BETWEEN ?4 AND ?5
             ORDER BY verse",
        )?;
        let rows = stmt.query_map(
            params![
                translation_id,
                reference.book_id,
                reference.chapter,
                start_verse as i64,
                end_verse as i64
            ],
            map_verse,
        )?;
        let verses = rows.collect::<Result<Vec<_>, _>>()?;

        if verses.is_empty() {
            return Err(AppError::ScriptureNotFound {
                translation: translation_id.to_string(),
                reference: reference.display(),
            });
        }

        let text = verses
            .iter()
            .map(|v| v.text.trim())
            .collect::<Vec<_>>()
            .join(" ");

        Ok(Passage {
            translation_id: translation_id.to_string(),
            reference: reference.display(),
            verses,
            text,
        })
    }

    /// Full-text search against the automatically-synced FTS5 index.
    pub fn search(
        &self,
        translation_id: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<Verse>, AppError> {
        let limit = limit.clamp(1, 200) as i64;
        let mut stmt = self.conn.prepare(
            "SELECT verses.translation_id, verses.book_id, verses.chapter, verses.verse, verses.text
             FROM verses_fts
             JOIN verses
               ON verses.rowid = verses_fts.rowid
              AND verses.translation_id = verses_fts.translation_id
             WHERE verses_fts.translation_id = ?1 AND verses_fts MATCH ?2
             LIMIT ?3",
        )?;
        let rows = stmt.query_map(params![translation_id, query, limit], map_verse)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Number of verses installed for a translation.
    pub fn verse_count(&self, translation_id: &str) -> Result<i64, AppError> {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM verses WHERE translation_id = ?1",
                [translation_id],
                |row| row.get(0),
            )
            .map_err(Into::into)
    }
}

/// Persistent key/value storage (user settings, setup status, ...).
pub struct SettingsRepository<'a> {
    conn: &'a Connection,
}

impl<'a> SettingsRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn get(&self, key: &str) -> Result<Option<String>, AppError> {
        self.conn
            .query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
                row.get(0)
            })
            .optional()
            .map_err(Into::into)
    }

    pub fn set(&self, key: &str, value: &str) -> Result<(), AppError> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_json<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, AppError> {
        match self.get(key)? {
            Some(raw) => serde_json::from_str(&raw)
                .map(Some)
                .map_err(|e| AppError::Internal(format!("corrupt settings '{key}': {e}"))),
            None => Ok(None),
        }
    }

    pub fn set_json<T: Serialize>(&self, key: &str, value: &T) -> Result<(), AppError> {
        let raw = serde_json::to_string(value)
            .map_err(|e| AppError::Internal(format!("failed to serialize settings: {e}")))?;
        self.set(key, &raw)
    }
}

/// Media library persistence. Files stay on disk — SQLite stores metadata.
pub struct MediaRepository<'a> {
    conn: &'a Connection,
}

impl<'a> MediaRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn list(&self) -> Result<Vec<MediaItem>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, type, name, path, metadata, created_at FROM media ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            let metadata: Option<String> = row.get(4)?;
            Ok(MediaItem {
                id: row.get(0)?,
                kind: row.get(1)?,
                name: row.get(2)?,
                path: row.get(3)?,
                metadata: metadata
                    .map(|m| serde_json::from_str(&m).unwrap_or_else(|_| serde_json::json!({}))),
                created_at: row.get(5)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn insert(&self, item: &MediaItem) -> Result<(), AppError> {
        let metadata = item
            .metadata
            .as_ref()
            .map(|m| m.to_string())
            .unwrap_or_else(|| "{}".to_string());
        self.conn.execute(
            "INSERT INTO media (id, type, name, path, metadata, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                item.id,
                item.kind,
                item.name,
                item.path,
                metadata,
                item.created_at
            ],
        )?;
        Ok(())
    }

    pub fn remove(&self, id: &str) -> Result<(), AppError> {
        self.conn.execute("DELETE FROM media WHERE id = ?1", [id])?;
        Ok(())
    }
}
/// Saved presentations persistence.
pub struct PresentationRepository<'a> {
    conn: &'a Connection,
}

impl<'a> PresentationRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    fn now() -> String {
        Utc::now().to_rfc3339()
    }

    pub fn create(&self, name: &str) -> Result<Presentation, AppError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Self::now();
        self.conn.execute(
            "INSERT INTO presentations (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
            params![id, name, now],
        )?;
        Ok(Presentation {
            id,
            name: name.to_string(),
            created_at: now.clone(),
            updated_at: now,
            items: Vec::new(),
        })
    }

    pub fn list(&self) -> Result<Vec<Presentation>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, created_at, updated_at FROM presentations ORDER BY updated_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Presentation {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
                items: Vec::new(),
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn get(&self, id: &str) -> Result<Option<Presentation>, AppError> {
        let meta = self
            .conn
            .query_row(
                "SELECT id, name, created_at, updated_at FROM presentations WHERE id = ?1",
                [id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                },
            )
            .optional()?;

        let Some((id, name, created_at, updated_at)) = meta else {
            return Ok(None);
        };

        let mut stmt = self.conn.prepare(
            "SELECT id, presentation_id, type, position, payload FROM presentation_items
             WHERE presentation_id = ?1 ORDER BY position",
        )?;
        let rows = stmt.query_map([&id], |row| {
            Ok(PresentationItemRecord {
                id: row.get(0)?,
                presentation_id: row.get(1)?,
                type_name: row.get(2)?,
                position: row.get(3)?,
                payload: row.get(4)?,
            })
        })?;
        let items = rows.collect::<Result<Vec<_>, _>>()?;

        Ok(Some(Presentation {
            id,
            name,
            created_at,
            updated_at,
            items,
        }))
    }

    /// Fetches a single stored item by id, across all presentations.
    pub fn get_item(&self, item_id: &str) -> Result<Option<PresentationItemRecord>, AppError> {
        self.conn
            .query_row(
                "SELECT id, presentation_id, type, position, payload
                 FROM presentation_items WHERE id = ?1",
                [item_id],
                |row| {
                    Ok(PresentationItemRecord {
                        id: row.get(0)?,
                        presentation_id: row.get(1)?,
                        type_name: row.get(2)?,
                        position: row.get(3)?,
                        payload: row.get(4)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn add_item(
        &self,
        presentation_id: &str,
        item: &PresentationItemRecord,
    ) -> Result<(), AppError> {
        let tx = self.conn.unchecked_transaction()?;
        let max: i64 = tx.query_row(
            "SELECT COALESCE(MAX(position), 0) FROM presentation_items WHERE presentation_id = ?1",
            [presentation_id],
            |row| row.get(0),
        )?;
        tx.execute(
            "INSERT INTO presentation_items (id, presentation_id, type, position, payload)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                item.id,
                presentation_id,
                item.type_name,
                max + 1,
                item.payload
            ],
        )?;
        tx.execute(
            "UPDATE presentations SET updated_at = ?1 WHERE id = ?2",
            params![Self::now(), presentation_id],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn remove_item(&self, presentation_id: &str, item_id: &str) -> Result<(), AppError> {
        let tx = self.conn.unchecked_transaction()?;
        let position: i64 = tx
            .query_row(
                "SELECT position FROM presentation_items WHERE id = ?1 AND presentation_id = ?2",
                params![item_id, presentation_id],
                |row| row.get(0),
            )
            .optional()?
            .ok_or_else(|| AppError::Presentation(format!("item {item_id} not found")))?;

        tx.execute(
            "DELETE FROM presentation_items WHERE id = ?1 AND presentation_id = ?2",
            params![item_id, presentation_id],
        )?;
        tx.execute(
            "UPDATE presentation_items SET position = position - 1
             WHERE presentation_id = ?1 AND position > ?2",
            params![presentation_id, position],
        )?;
        tx.execute(
            "UPDATE presentations SET updated_at = ?1 WHERE id = ?2",
            params![Self::now(), presentation_id],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<(), AppError> {
        self.conn
            .execute("DELETE FROM presentations WHERE id = ?1", [id])?;
        Ok(())
    }
}

fn map_verse(row: &rusqlite::Row<'_>) -> rusqlite::Result<Verse> {
    Ok(Verse {
        translation_id: row.get(0)?,
        book_id: row.get(1)?,
        chapter: row.get(2)?,
        verse: row.get(3)?,
        text: row.get(4)?,
    })
}
