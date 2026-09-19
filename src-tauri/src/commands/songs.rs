//! Song commands: library management and section-by-section projection.

use serde::Deserialize;
use tauri::{AppHandle, State};

use crate::errors::{AppError, CommandResult};
use crate::models::presentation::{PresentationItem, PresentationState};
use crate::songs::{Song, SongInput, SongRepository};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSongRequest {
    /// Absent when creating, present when replacing an existing song.
    #[serde(default)]
    pub id: Option<String>,
    pub song: SongInput,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSongRequest {
    pub song_id: String,
    /// 1-based section to start from. The remaining sections wait in the
    /// queue, so the operator can walk the song with Next/Back.
    #[serde(default)]
    pub section: Option<usize>,
}

#[tauri::command]
pub fn list_songs(state: State<'_, AppState>) -> CommandResult<Vec<Song>> {
    state.with_conn(|conn| SongRepository::new(conn).list())
}

#[tauri::command]
pub fn get_song(id: String, state: State<'_, AppState>) -> CommandResult<Option<Song>> {
    state.with_conn(|conn| SongRepository::new(conn).get(&id))
}

/// Creates a song, or replaces one when `id` is supplied.
#[tauri::command]
pub fn save_song(request: SaveSongRequest, state: State<'_, AppState>) -> CommandResult<Song> {
    let input = request.song.normalized()?;
    state.with_conn(|conn| {
        let repo = SongRepository::new(conn);
        match request.id.as_deref() {
            Some(id) => repo.update(id, &input),
            None => repo.create(&input),
        }
    })
}

#[tauri::command]
pub fn delete_song(id: String, state: State<'_, AppState>) -> CommandResult<()> {
    state.with_conn(|conn| SongRepository::new(conn).delete(&id))
}

/// Shows a song on the projector, starting at `section`.
///
/// Each section becomes its own projectable item, which is what lets a song be
/// presented verse by verse the same way Scripture is.
#[tauri::command]
pub fn project_song(
    request: ProjectSongRequest,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<PresentationState> {
    let song = state
        .with_conn(|conn| SongRepository::new(conn).get(&request.song_id))?
        .ok_or_else(|| {
            AppError::Presentation(format!("song {} no longer exists", request.song_id))
        })?;

    if song.sections.is_empty() {
        return Err(AppError::Presentation(
            "this song has no sections yet — add some words first".to_string(),
        ));
    }

    let total = song.sections.len() as u32;
    let start = request.section.unwrap_or(1).clamp(1, total as usize);

    let section_item = |index: usize| {
        let section = &song.sections[index];
        PresentationItem::song_section(
            &song.title,
            section.label.clone(),
            section.text.clone(),
            index as u32 + 1,
            total,
        )
    };

    state.presentation.clear_queue()?;
    for index in start..song.sections.len() {
        state.presentation.queue(section_item(index))?;
    }
    state.project(section_item(start - 1), &app)?;

    Ok(state.presentation.state())
}
