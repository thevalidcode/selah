//! Bible commands: translations, books, passages, search, import.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::bible::models::{Passage, Translation, Verse};
use crate::errors::{AppError, CommandResult};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PassageRequest {
    pub translation_id: String,
    pub book_id: i32,
    pub chapter: u16,
    pub start_verse: Option<u16>,
    pub end_verse: Option<u16>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationStatus {
    pub translation: Translation,
    pub verse_count: i64,
}

fn list_books_impl(
    conn: &rusqlite::Connection,
) -> Result<Vec<crate::bible::models::BibleBook>, crate::errors::AppError> {
    crate::bible::repository::BibleRepository::new(conn).list_books()
}

#[tauri::command]
pub fn list_bible_translations(
    state: State<'_, AppState>,
) -> CommandResult<Vec<TranslationStatus>> {
    state.with_conn(|conn| {
        let repo = crate::bible::repository::BibleRepository::new(conn);
        let mut out = Vec::new();
        for translation in repo.list_translations()? {
            let verse_count = repo.verse_count(&translation.id)?;
            out.push(TranslationStatus {
                translation,
                verse_count,
            });
        }
        Ok(out)
    })
}

#[tauri::command]
pub fn list_books(
    state: State<'_, AppState>,
) -> CommandResult<Vec<crate::bible::models::BibleBook>> {
    state.with_conn(list_books_impl)
}

#[tauri::command]
pub fn get_passage(request: PassageRequest, state: State<'_, AppState>) -> CommandResult<Passage> {
    let reference = crate::scripture::ScriptureReference {
        book_id: request.book_id,
        chapter: request.chapter,
        start_verse: request.start_verse,
        end_verse: request.end_verse,
    };
    state.with_conn(|conn| {
        let repo = crate::bible::repository::BibleRepository::new(conn);
        crate::scripture::resolver::resolve_passage(&repo, &request.translation_id, &reference)
    })
}

#[tauri::command]
pub fn search_bible(
    translation_id: String,
    query: String,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> CommandResult<Vec<Verse>> {
    state.with_conn(|conn| {
        crate::bible::repository::BibleRepository::new(conn).search(
            &translation_id,
            &query,
            limit.unwrap_or(25),
        )
    })
}

#[tauri::command]
pub fn set_default_translation(
    translation_id: String,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    state.with_conn(|conn| {
        crate::bible::repository::BibleRepository::new(conn)
            .set_default_translation(&translation_id)
    })?;
    // Keep settings in sync so the UI has a single source of truth.
    if let Ok(mut settings) = state.settings.lock() {
        settings.general.default_translation_id = Some(translation_id);
    }
    state.save_settings()
}

/// Result of importing a translation document.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub translation_id: String,
    pub verses_imported: usize,
}

/// Request to import one of the common SQLite Bible downloads.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SqliteImportRequest {
    /// Absolute path to the `.sqlite` file the operator chose.
    pub path: String,
    /// Short id used everywhere afterwards, e.g. `kjv`.
    pub translation_id: String,
    /// Full name shown in the interface, e.g. `King James Version`.
    pub name: String,
    pub abbreviation: Option<String>,
    /// Whether this should become the translation Selah starts with.
    #[serde(default)]
    pub make_default: bool,
}

/// Imports a translation from a SQLite file that uses the widely published
/// `verses(book_id, chapter, number, text)` layout.
///
/// Selah ships no Bible text; the operator points at a file they are entitled
/// to use. The file is attached read-only and copied in one transaction.
#[tauri::command]
pub fn import_sqlite_bible_translation(
    request: SqliteImportRequest,
    state: State<'_, AppState>,
) -> CommandResult<ImportResult> {
    let path = std::path::PathBuf::from(request.path.trim());
    if !path.is_absolute() {
        return Err(AppError::InvalidConfiguration(
            "the Bible file path must be absolute".to_string(),
        ));
    }
    if !path.is_file() {
        return Err(AppError::Media(format!(
            "Bible file does not exist: {}",
            path.display()
        )));
    }

    let translation_id = request.translation_id.trim().to_lowercase();
    if translation_id.is_empty() || request.name.trim().is_empty() {
        return Err(AppError::InvalidConfiguration(
            "a short id and a name are both required".to_string(),
        ));
    }

    let info = crate::bible::models::TranslationInfo {
        id: translation_id,
        name: request.name.trim().to_string(),
        language: "en".to_string(),
        abbreviation: request.abbreviation.filter(|a| !a.trim().is_empty()),
        is_default: request.make_default,
    };

    let outcome = state
        .with_conn(|conn| crate::bible::import::import_sqlite_translation(conn, &path, &info))?;

    if request.make_default {
        if let Ok(mut settings) = state.settings.lock() {
            settings.general.default_translation_id = Some(outcome.translation_id.clone());
        }
        state.save_settings()?;
    }

    Ok(ImportResult {
        translation_id: outcome.translation_id,
        verses_imported: outcome.verses_imported,
    })
}

/// Imports a translation from a local JSON document.
///
/// Selah ships no Bible text: the operator points at a file they are licensed
/// to use. The path is read directly (no shell, no globbing) and the whole
/// import runs inside a single transaction.
#[tauri::command]
pub fn import_bible_translation(
    path: String,
    state: State<'_, AppState>,
) -> CommandResult<ImportResult> {
    let path = std::path::PathBuf::from(path.trim());
    if !path.is_absolute() {
        return Err(AppError::InvalidConfiguration(
            "the import path must be absolute".to_string(),
        ));
    }
    if !path.is_file() {
        return Err(AppError::Media(format!(
            "import file does not exist: {}",
            path.display()
        )));
    }

    let document = crate::bible::import::parse_import(&path)?;
    let (verses_imported, translation_id) =
        state.with_conn(|conn| crate::bible::import::apply_import(conn, &document))?;

    tracing::info!(
        translation = %translation_id,
        verses = verses_imported,
        "bible translation imported"
    );

    Ok(ImportResult {
        translation_id,
        verses_imported,
    })
}
