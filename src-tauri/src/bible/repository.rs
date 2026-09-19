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
            "SELECT id, name, language, abbreviation, is_default, created_at, builtin, origin
             FROM translations ORDER BY builtin DESC, is_default DESC, name",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Translation {
                id: row.get(0)?,
                name: row.get(1)?,
                language: row.get(2)?,
                abbreviation: row.get(3)?,
                is_default: row.get::<_, i64>(4)? != 0,
                created_at: row.get(5)?,
                builtin: row.get::<_, i64>(6)? != 0,
                origin: row.get(7)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn get_translation(&self, id: &str) -> Result<Option<Translation>, AppError> {
        self.conn
            .query_row(
                "SELECT id, name, language, abbreviation, is_default, created_at, builtin, origin
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
                        builtin: row.get::<_, i64>(6)? != 0,
                        origin: row.get(7)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    /// Registers translation metadata. Does not require verse data yet.
    ///
    /// New translations always land as editable (`builtin = 0`): only
    /// [`Self::mark_builtin`] may promote one to read-only, and only the
    /// seeding path calls it.
    pub fn upsert_translation(&self, info: &TranslationInfo) -> Result<(), AppError> {
        self.insert_translation(info, "operator")
    }

    /// Registers a translation that came from the published catalogue.
    pub fn add_catalogue_translation(&self, info: &TranslationInfo) -> Result<(), AppError> {
        self.insert_translation(info, "catalogue")
    }

    /// The shared insert used by every path that creates a translation.
    fn insert_translation(&self, info: &TranslationInfo, origin: &str) -> Result<(), AppError> {
        self.conn.execute(
            "INSERT INTO translations (id, name, language, abbreviation, is_default, created_at, builtin, origin)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7)
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
                Utc::now().to_rfc3339(),
                origin
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

    /// Marks a translation as one of Selah's own, which makes it read-only.
    ///
    /// Called for every bundled translation on start-up, so a database written
    /// before the flag existed repairs itself.
    pub fn mark_builtin(&self, id: &str) -> Result<(), AppError> {
        self.conn.execute(
            "UPDATE translations SET builtin = 1, origin = 'bundled' WHERE id = ?1",
            [id],
        )?;
        Ok(())
    }

    /// Renames a translation, or changes the details shown in the picker.
    ///
    /// The three translations Selah ships may not be renamed: they are the ones
    /// the application can always rely on being present and correctly named.
    pub fn rename_translation(
        &self,
        id: &str,
        name: &str,
        abbreviation: Option<&str>,
        language: &str,
    ) -> Result<(), AppError> {
        let existing = self
            .get_translation(id)?
            .ok_or_else(|| AppError::BibleTranslationNotFound(id.to_string()))?;

        if existing.builtin {
            return Err(AppError::InvalidConfiguration(format!(
                "{} is one of the Bibles that come with Selah, so it cannot be renamed",
                existing.name
            )));
        }

        self.conn.execute(
            "UPDATE translations SET name = ?1, abbreviation = ?2, language = ?3 WHERE id = ?4",
            params![name, abbreviation, language, id],
        )?;
        Ok(())
    }

    /// Removes a translation and every verse stored under it.
    ///
    /// Verses go with it through the schema's cascade. The bundled translations
    /// cannot be removed, and removing the default hands the default flag to
    /// another installed translation so the Bible screen always has one to
    /// open on.
    pub fn delete_translation(&self, id: &str) -> Result<(), AppError> {
        let existing = self
            .get_translation(id)?
            .ok_or_else(|| AppError::BibleTranslationNotFound(id.to_string()))?;

        if existing.builtin {
            return Err(AppError::InvalidConfiguration(format!(
                "{} is one of the Bibles that come with Selah, so it cannot be removed",
                existing.name
            )));
        }

        let tx = self.conn.unchecked_transaction()?;
        tx.execute("DELETE FROM translations WHERE id = ?1", [id])?;

        if existing.is_default {
            // Hand the default to something that is still installed, rather
            // than leaving the interface with nothing selected.
            tx.execute(
                "UPDATE translations SET is_default = 1
                 WHERE id = (SELECT id FROM translations ORDER BY builtin DESC, name LIMIT 1)",
                [],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    /// The translation currently flagged as the default, if any.
    pub fn default_translation(&self) -> Result<Option<Translation>, AppError> {
        self.conn
            .query_row(
                "SELECT id, name, language, abbreviation, is_default, created_at, builtin, origin
                 FROM translations WHERE is_default = 1 LIMIT 1",
                [],
                |row| {
                    Ok(Translation {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        language: row.get(2)?,
                        abbreviation: row.get(3)?,
                        is_default: row.get::<_, i64>(4)? != 0,
                        created_at: row.get(5)?,
                        builtin: row.get::<_, i64>(6)? != 0,
                        origin: row.get(7)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
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
    ///
    /// When `translation_id` is `None` the search runs across **every**
    /// translation, so an operator who does not remember which version a phrase
    /// came from still finds it. Results keep their `translation_id`, so the
    /// interface can say which version each match came from.
    pub fn search(
        &self,
        translation_id: Option<&str>,
        query: &str,
        limit: usize,
    ) -> Result<Vec<Verse>, AppError> {
        let limit = limit.clamp(1, 200) as i64;
        let statement = match translation_id {
            Some(_) => {
                "SELECT verses.translation_id, verses.book_id, verses.chapter, verses.verse, verses.text
                 FROM verses_fts
                 JOIN verses
                   ON verses.rowid = verses_fts.rowid
                  AND verses.translation_id = verses_fts.translation_id
                 WHERE verses_fts.translation_id = ?1 AND verses_fts MATCH ?2
                 LIMIT ?3"
            }
            None => {
                "SELECT verses.translation_id, verses.book_id, verses.chapter, verses.verse, verses.text
                 FROM verses_fts
                 JOIN verses
                   ON verses.rowid = verses_fts.rowid
                  AND verses.translation_id = verses_fts.translation_id
                 WHERE verses_fts MATCH ?1
                 LIMIT ?2"
            }
        };
        let mut stmt = self.conn.prepare(statement)?;

        let rows = match translation_id {
            Some(id) => stmt
                .query_map(params![id, query, limit], map_verse)?
                .collect::<Result<Vec<_>, _>>()?,
            None => stmt
                .query_map(params![query, limit], map_verse)?
                .collect::<Result<Vec<_>, _>>()?,
        };
        Ok(rows)
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

    /// Stores verses typed or pasted by the operator, in one transaction.
    ///
    /// This is the manual-entry path: the operator is responsible for the words,
    /// Selah only checks that each verse names a real book, chapter and verse.
    /// Existing rows for the same reference are replaced, so re-pasting a
    /// corrected chapter updates it rather than doubling it.
    pub fn save_verses(&self, info: &TranslationInfo, verses: &[Verse]) -> Result<usize, AppError> {
        let tx = self.conn.unchecked_transaction()?;
        {
            let repo = BibleRepository::new(&tx);
            // The translation is created only when it is not there yet: adding
            // verses to a Bible that is already installed must never rename it
            // or change which one is the default.
            if repo.get_translation(&info.id)?.is_none() {
                repo.upsert_translation(info)?;
            }
            for verse in verses {
                repo.insert_verse(verse)?;
            }
        }
        tx.commit()?;
        Ok(verses.len())
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

    /// Looks up a registered file by its path.
    ///
    /// Media files are read from a folder rather than added one at a time, so
    /// this is what keeps a repeated folder scan from creating duplicates.
    pub fn find_by_path(&self, path: &str) -> Result<Option<MediaItem>, AppError> {
        self.conn
            .query_row(
                "SELECT id, type, name, path, metadata, created_at FROM media WHERE path = ?1",
                [path],
                |row| {
                    let metadata: Option<String> = row.get(4)?;
                    Ok(MediaItem {
                        id: row.get(0)?,
                        kind: row.get(1)?,
                        name: row.get(2)?,
                        path: row.get(3)?,
                        metadata: metadata.map(|m| {
                            serde_json::from_str(&m).unwrap_or_else(|_| serde_json::json!({}))
                        }),
                        created_at: row.get(5)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
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

    /// Replaces the payload of a stored item, keeping its type and position.
    ///
    /// Editing a saved presentation is the same idea as the original save: the
    /// row's `payload` is the whole content, so replacing it is all that is
    /// needed. The presentation's `updated_at` is bumped so the list shows the
    /// edit.
    pub fn update_item(
        &self,
        presentation_id: &str,
        item_id: &str,
        type_name: &str,
        payload: &str,
    ) -> Result<(), AppError> {
        let tx = self.conn.unchecked_transaction()?;
        let changed = tx.execute(
            "UPDATE presentation_items
             SET payload = ?1, type = ?2
             WHERE id = ?3 AND presentation_id = ?4",
            params![payload, type_name, item_id, presentation_id],
        )?;

        if changed == 0 {
            return Err(AppError::Presentation(format!(
                "item {item_id} is not part of presentation {presentation_id}"
            )));
        }

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
