//! Media commands: folder browsing, library listing and projection.
//!
//! Media is loaded from a folder chosen at runtime — no path is baked into the
//! application — so the commands here are "look at this folder", "load this
//! folder" and "put this file on the screen".

use serde::Deserialize;
use std::path::PathBuf;
use tauri::{AppHandle, State};

use crate::errors::{AppError, CommandResult};
use crate::media::{DirectoryListing, MediaItem, MediaLibrary, MediaScan};
use crate::models::presentation::{PresentationItem, PresentationState};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportMediaRequest {
    pub path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadMediaDirectoryRequest {
    pub path: String,
    /// Whether sub-folders are read as well.
    #[serde(default)]
    pub recursive: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMediaRequest {
    pub path: String,
    /// Heading shown with the file; falls back to the file name.
    #[serde(default)]
    pub title: Option<String>,
}

#[tauri::command]
pub fn list_media(state: State<'_, AppState>) -> CommandResult<Vec<MediaItem>> {
    state.with_conn(crate::media::MediaLibrary::list)
}

/// Lists a folder so the operator can walk to the folder they want.
///
/// `path` omitted starts at the user's home folder.
#[tauri::command]
pub fn browse_directory(path: Option<String>) -> CommandResult<DirectoryListing> {
    let path = path.map(PathBuf::from);
    MediaLibrary::browse(path.as_deref())
}

/// Reads a folder and registers the media inside it.
///
/// The folder is remembered in settings (and granted to the projector window)
/// so the same library is available on the next launch without picking it again.
#[tauri::command]
pub fn load_media_directory(
    request: LoadMediaDirectoryRequest,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<MediaScan> {
    let requested = PathBuf::from(&request.path);
    let directory = requested
        .canonicalize()
        .map_err(|e| AppError::Media(format!("cannot open {}: {e}", requested.display())))?;

    // Open the folder to the projector window before anything is registered,
    // otherwise the pictures would list but never draw.
    crate::media::grant_directory(&app, &directory)?;

    let scan = state.with_conn(|conn| MediaLibrary::scan(conn, &directory, request.recursive))?;

    {
        let mut settings = state
            .settings
            .lock()
            .map_err(|_| AppError::Internal("settings lock poisoned".to_string()))?;
        settings.media.directory = Some(directory.to_string_lossy().to_string());
        settings.media.recursive = request.recursive;
    }
    state.save_settings()?;

    Ok(scan)
}

/// Registers a single file (used when a file is added by hand).
#[tauri::command]
pub fn import_media(
    request: ImportMediaRequest,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<MediaItem> {
    let path = PathBuf::from(&request.path);
    let item = state.with_conn(|conn| crate::media::MediaLibrary::import(conn, path.clone()))?;
    // Make sure the projector window may read it.
    crate::media::grant_file(&app, &path)?;
    Ok(item)
}

#[tauri::command]
pub fn remove_media(id: String, state: State<'_, AppState>) -> CommandResult<()> {
    state.with_conn(|conn| crate::media::MediaLibrary::remove(conn, &id))
}

/// Puts a media file on the congregation's screen.
///
/// The projector window is a separate webview, so the file is handed over as an
/// `asset:` URL by the frontend; this command checks the file is real and that
/// Selah can present it, then projects it.
#[tauri::command]
pub fn project_media(
    request: ProjectMediaRequest,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<PresentationState> {
    let path = PathBuf::from(&request.path);
    if !path.is_file() {
        return Err(AppError::Media(format!(
            "file does not exist: {}",
            path.display()
        )));
    }
    let kind = crate::media::kind_for_path(&path).ok_or_else(|| {
        AppError::Media(format!(
            "Selah cannot present {}",
            path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.display().to_string())
        ))
    })?;

    // A file added by hand may live outside the current media folder, so open
    // it explicitly before projecting.
    crate::media::grant_file(&app, &path)?;

    let title = request
        .title
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(str::to_string)
        .or_else(|| path.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "Media".to_string());

    // Only print a caption when one was given: a picture or video should fill
    // the screen without its file name written over it.
    let caption = request
        .title
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty());

    let item = PresentationItem::media(title, request.path, Some(kind.to_string()), caption);
    state.project(item, &app)?;
    Ok(state.presentation.state())
}
