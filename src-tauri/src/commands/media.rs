//! Media commands: library listing and import.

use serde::Deserialize;
use tauri::State;

use crate::errors::CommandResult;
use crate::media::MediaItem;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportMediaRequest {
    pub path: String,
}

#[tauri::command]
pub fn list_media(state: State<'_, AppState>) -> CommandResult<Vec<MediaItem>> {
    state.with_conn(crate::media::MediaLibrary::list)
}

#[tauri::command]
pub fn import_media(
    request: ImportMediaRequest,
    state: State<'_, AppState>,
) -> CommandResult<MediaItem> {
    let path = std::path::PathBuf::from(&request.path);
    state.with_conn(|conn| crate::media::MediaLibrary::import(conn, path))
}

#[tauri::command]
pub fn remove_media(id: String, state: State<'_, AppState>) -> CommandResult<()> {
    state.with_conn(|conn| crate::media::MediaLibrary::remove(conn, &id))
}
