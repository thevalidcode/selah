//! Scripture resolver: reference → resolved passage text.
//!
//! The resolver is the bridge between the deterministic parser and the SQLite
//! Bible repository. Detection is decoupled from resolution: a reference can
//! be detected before the translation data has been imported.

use crate::bible::models::Passage;
use crate::bible::repository::BibleRepository;
use crate::errors::AppError;

use super::reference::ScriptureReference;

/// Resolves a parsed reference against an installed translation.
pub fn resolve_passage(
    repo: &BibleRepository<'_>,
    translation_id: &str,
    reference: &ScriptureReference,
) -> Result<Passage, AppError> {
    if !reference.is_valid() {
        return Err(AppError::InvalidScriptureReference(reference.display()));
    }
    let translation = repo.get_translation(translation_id)?;
    match translation {
        None => Err(AppError::BibleTranslationNotFound(
            translation_id.to_string(),
        )),
        Some(_) => repo.get_passage(translation_id, reference),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bible::repository::BibleRepository;
    use crate::db;
    use rusqlite::Connection;

    fn memory_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        db::migrations::run_migrations(&conn).unwrap();
        conn
    }

    #[test]
    fn resolution_roundtrip() {
        let conn = memory_db();
        let repo = BibleRepository::new(&conn);
        repo.upsert_translation(&crate::bible::models::TranslationInfo {
            id: "web".into(),
            name: "World English Bible".into(),
            language: "en".into(),
            abbreviation: Some("WEB".into()),
            is_default: true,
        })
        .unwrap();
        repo.insert_verse(&crate::bible::models::Verse {
            translation_id: "web".into(),
            book_id: 43,
            chapter: 3,
            verse: 16,
            text: "For God so loved the world...".into(),
        })
        .unwrap();

        let reference = ScriptureReference::new(43, 3).with_verse(16);
        let passage = resolve_passage(&repo, "web", &reference).unwrap();
        assert_eq!(passage.reference, "John 3:16");
        assert_eq!(passage.verses.len(), 1);
        assert!(passage.text.contains("God"));
    }

    #[test]
    fn unknown_translation_errors() {
        let conn = memory_db();
        let repo = BibleRepository::new(&conn);
        let reference = ScriptureReference::new(43, 3);
        let err = resolve_passage(&repo, "kjv", &reference).unwrap_err();
        assert!(matches!(err, AppError::BibleTranslationNotFound(_)));
    }
}
