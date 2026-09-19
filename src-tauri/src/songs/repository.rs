//! Song library persistence.
//!
//! All song SQL lives here (see the module docs in `bible::repository` for why
//! repositories — not services — own the SQL). Sections are replaced as a whole
//! on update: the editor always sends the complete, ordered song.

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};

use crate::errors::AppError;

use super::models::{Song, SongInput, SongSection};

pub struct SongRepository<'a> {
    conn: &'a Connection,
}

impl<'a> SongRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    fn now() -> String {
        Utc::now().to_rfc3339()
    }

    /// Every song, in alphabetical order, without sections (library list).
    pub fn list(&self) -> Result<Vec<Song>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, author, created_at, updated_at FROM songs
             ORDER BY lower(title)",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Song {
                id: row.get(0)?,
                title: row.get(1)?,
                author: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
                sections: Vec::new(),
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// A single song with its sections in order.
    pub fn get(&self, id: &str) -> Result<Option<Song>, AppError> {
        let meta = self
            .conn
            .query_row(
                "SELECT id, title, author, created_at, updated_at FROM songs WHERE id = ?1",
                [id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                    ))
                },
            )
            .optional()?;

        let Some((id, title, author, created_at, updated_at)) = meta else {
            return Ok(None);
        };

        let mut stmt = self.conn.prepare(
            "SELECT id, label, text, position FROM song_sections
             WHERE song_id = ?1 ORDER BY position",
        )?;
        let rows = stmt.query_map([&id], |row| {
            Ok(SongSection {
                id: row.get(0)?,
                label: row.get(1)?,
                text: row.get(2)?,
                position: row.get(3)?,
            })
        })?;
        let sections = rows.collect::<Result<Vec<_>, _>>()?;

        Ok(Some(Song {
            id,
            title,
            author,
            created_at,
            updated_at,
            sections,
        }))
    }

    /// Creates a song and its sections in one transaction.
    pub fn create(&self, input: &SongInput) -> Result<Song, AppError> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Self::now();

        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "INSERT INTO songs (id, title, author, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?4)",
            params![id, input.title, input.author, now],
        )?;
        insert_sections(&tx, &id, input)?;
        tx.commit()?;

        self.get(&id)?
            .ok_or_else(|| AppError::Internal(format!("song {id} vanished after insert")))
    }

    /// Replaces a song's title, author and sections as a whole.
    pub fn update(&self, id: &str, input: &SongInput) -> Result<Song, AppError> {
        let tx = self.conn.unchecked_transaction()?;
        let updated = tx.execute(
            "UPDATE songs SET title = ?1, author = ?2, updated_at = ?3 WHERE id = ?4",
            params![input.title, input.author, Self::now(), id],
        )?;
        if updated == 0 {
            return Err(AppError::InvalidConfiguration(format!(
                "song {id} does not exist"
            )));
        }
        tx.execute("DELETE FROM song_sections WHERE song_id = ?1", [id])?;
        insert_sections(&tx, id, input)?;
        tx.commit()?;

        self.get(id)?
            .ok_or_else(|| AppError::Internal(format!("song {id} vanished after update")))
    }

    pub fn delete(&self, id: &str) -> Result<(), AppError> {
        self.conn.execute("DELETE FROM songs WHERE id = ?1", [id])?;
        Ok(())
    }
}

/// Writes the ordered section rows for a song.
fn insert_sections(conn: &Connection, song_id: &str, input: &SongInput) -> Result<(), AppError> {
    let mut stmt = conn.prepare(
        "INSERT INTO song_sections (id, song_id, position, label, text)
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )?;
    for (index, section) in input.sections.iter().enumerate() {
        stmt.execute(params![
            uuid::Uuid::new_v4().to_string(),
            song_id,
            index as i64 + 1,
            section.label,
            section.text
        ])?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::songs::models::SongSectionInput;

    fn memory_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        // Foreign keys are enforced in the real database; mirror that here so
        // the ON DELETE CASCADE below is actually exercised.
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        db::migrations::run_migrations(&conn).unwrap();
        conn
    }

    fn input(title: &str, sections: &[(&str, &str)]) -> SongInput {
        SongInput {
            title: title.to_string(),
            author: Some("Someone".to_string()),
            sections: sections
                .iter()
                .map(|(label, text)| SongSectionInput {
                    label: Some(label.to_string()),
                    text: text.to_string(),
                })
                .collect(),
        }
    }

    #[test]
    fn songs_round_trip_with_ordered_sections() {
        let conn = memory_db();
        let repo = SongRepository::new(&conn);

        let created = repo
            .create(&input(
                "Amazing Grace",
                &[
                    ("Verse 1", "Amazing grace"),
                    ("Chorus", "My chains are gone"),
                ],
            ))
            .expect("create");

        assert_eq!(created.sections.len(), 2);
        assert_eq!(created.sections[0].position, 1);
        assert_eq!(created.sections[1].position, 2);
        assert_eq!(created.sections[1].label.as_deref(), Some("Chorus"));

        let listed = repo.list().unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].title, "Amazing Grace");

        let fetched = repo.get(&created.id).unwrap().expect("song exists");
        assert_eq!(fetched, created);
    }

    #[test]
    fn updating_replaces_sections_instead_of_appending() {
        let conn = memory_db();
        let repo = SongRepository::new(&conn);
        let created = repo
            .create(&input("Song", &[("Verse 1", "one"), ("Verse 2", "two")]))
            .unwrap();

        let updated = repo
            .update(&created.id, &input("Song", &[("Verse 1", "one, edited")]))
            .unwrap();

        assert_eq!(updated.sections.len(), 1);
        assert_eq!(updated.sections[0].text, "one, edited");
    }

    #[test]
    fn deleting_a_song_removes_its_sections() {
        let conn = memory_db();
        let repo = SongRepository::new(&conn);
        let created = repo.create(&input("Song", &[("Verse 1", "one")])).unwrap();

        repo.delete(&created.id).unwrap();

        assert!(repo.get(&created.id).unwrap().is_none());
        let remaining: i64 = conn
            .query_row("SELECT COUNT(*) FROM song_sections", [], |row| row.get(0))
            .unwrap();
        assert_eq!(remaining, 0, "sections must cascade with their song");
    }

    #[test]
    fn updating_a_missing_song_reports_a_clear_error() {
        let conn = memory_db();
        let repo = SongRepository::new(&conn);
        let err = repo
            .update("nope", &input("Song", &[("Verse 1", "one")]))
            .unwrap_err();
        assert!(err.to_string().contains("does not exist"));
    }
}
