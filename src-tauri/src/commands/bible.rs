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
