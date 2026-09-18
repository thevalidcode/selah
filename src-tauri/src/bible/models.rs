//! Bible domain models.

use serde::{Deserialize, Serialize};

/// A Bible translation installed in the local database.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Translation {
    pub id: String,
    pub name: String,
    pub language: String,
    pub abbreviation: Option<String>,
    pub is_default: bool,
    pub created_at: String,
}

/// A Bible book row mirrored from the canonical registry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BibleBook {
    pub id: i32,
    pub name: String,
    pub testament: String,
    pub abbreviation: Option<String>,
    /// Canonical chapter count, resolved from the deterministic Scripture
    /// registry. The `books` table intentionally does not duplicate it.
    pub chapters: u16,
}

impl BibleBook {
    /// Builds a book row, filling the chapter count from the canonical
    /// registry so callers never duplicate that data.
    pub fn new(id: i32, name: String, testament: String, abbreviation: Option<String>) -> Self {
        Self {
            id,
            name,
            testament,
            abbreviation,
            chapters: crate::scripture::books::book_by_id(id)
                .map(|book| book.chapters)
                .unwrap_or(0),
        }
    }
}

/// A single verse row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Verse {
    pub translation_id: String,
    pub book_id: i32,
    pub chapter: i32,
    pub verse: i32,
    pub text: String,
}

/// A resolved passage: one verse or a contiguous range of verses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Passage {
    pub translation_id: String,
    /// Human readable reference label, e.g. "John 3:16" or "Psalm 23".
    pub reference: String,
    pub verses: Vec<Verse>,
    /// Verse text separated by spaces, ready for display.
    pub text: String,
}

/// Metadata describing a translation before any verses are imported.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationInfo {
    pub id: String,
    pub name: String,
    pub language: String,
    pub abbreviation: Option<String>,
    pub is_default: bool,
}
