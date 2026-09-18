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
