//! Presentation commands: projection, display window, saved presentations.

use serde::Deserialize;
use tauri::{AppHandle, State};

use crate::errors::{AppError, CommandResult};
use crate::models::presentation::PresentationItem;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectTextRequest {
    pub title: String,
    pub text: String,
    pub display: Option<usize>,
    pub fullscreen: Option<bool>,
}

#[tauri::command]
pub fn get_presentation_state(
    state: State<'_, AppState>,
) -> CommandResult<crate::models::presentation::PresentationState> {
    Ok(state.presentation.state())
}

/// Projects plain text (announcements, custom text).
#[tauri::command]
pub async fn project_text(
    app: AppHandle,
    state: State<'_, AppState>,
    request: ProjectTextRequest,
) -> CommandResult<crate::models::presentation::PresentationState> {
    if let Some(display) = request.display {
        state.display.set_target(display)?;
    }
    // An explicit fullscreen choice wins; otherwise the saved setting does.
    if let Some(fullscreen) = request.fullscreen {
        if state.display.is_open() {
            state.display.set_fullscreen(fullscreen)?;
        }
    }
    let item = PresentationItem::plain_text(&request.title, &request.text);
    state.project(item, &app)?;
    Ok(state.presentation.state())
}

/// Projects a resolved passage.
#[tauri::command]
pub async fn project_passage(
    app: AppHandle,
    state: State<'_, AppState>,
    passage: crate::bible::models::Passage,
) -> CommandResult<crate::models::presentation::PresentationState> {
    let item = PresentationItem::scripture(
        &passage.reference,
        &passage.translation_id,
        passage.text.clone(),
    );
    state.project(item, &app)?;
    Ok(state.presentation.state())
}

#[tauri::command]
pub fn clear_presentation(
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<crate::models::presentation::PresentationState> {
    state.presentation.clear(&state.display, &app)?;
    Ok(state.presentation.state())
}

#[tauri::command]
pub async fn open_presentation_window(
    app: AppHandle,
    state: State<'_, AppState>,
    display: Option<usize>,
    fullscreen: Option<bool>,
) -> CommandResult<crate::presentation::DisplayInfo> {
    // Saved settings decide the screen and the fullscreen state; an explicit
    // argument from the caller (Settings' "Show the screen" button) wins.
    let saved = state
        .settings
        .lock()
        .map(|s| s.presentation.clone())
        .unwrap_or_default();
    let target = display.or(saved.display_index);
    if let Some(index) = target {
        state.display.set_target(index)?;
    }
    let fullscreen = fullscreen.unwrap_or(saved.fullscreen);

    let opened = state.display.open(&app, fullscreen)?;

    // Tell the freshly opened window how projected content should look, so it
    // does not paint one frame of black-on-white defaults first.
    state.apply_presentation_settings(&app)?;
    Ok(opened)
}

#[tauri::command]
pub fn close_presentation_window(state: State<'_, AppState>) -> CommandResult<()> {
    state.display.close()
}

#[tauri::command]
pub fn set_presentation_display(
    state: State<'_, AppState>,
    display: usize,
) -> CommandResult<crate::presentation::DisplayInfo> {
    state.display.set_target(display)?;
    let displays = state.display.list_displays()?;
    displays
        .into_iter()
        .find(|d| d.index == display)
        .ok_or_else(|| AppError::DisplayNotFound(format!("no display with index {display}")))
}

#[tauri::command]
pub fn set_fullscreen(state: State<'_, AppState>, fullscreen: bool) -> CommandResult<()> {
    state.display.set_fullscreen(fullscreen)
}

#[tauri::command]
pub fn list_displays(
    state: State<'_, AppState>,
) -> CommandResult<Vec<crate::presentation::DisplayInfo>> {
    state.display.list_displays()
}

// ---------------------------------------------------------------
// Saved presentations
// ---------------------------------------------------------------

#[tauri::command]
pub fn list_presentations(
    state: State<'_, AppState>,
) -> CommandResult<Vec<crate::models::presentation::Presentation>> {
    state.with_conn(|conn| crate::bible::repository::PresentationRepository::new(conn).list())
}

#[tauri::command]
pub fn get_presentation(
    id: String,
    state: State<'_, AppState>,
) -> CommandResult<Option<crate::models::presentation::Presentation>> {
    state.with_conn(|conn| crate::bible::repository::PresentationRepository::new(conn).get(&id))
}

#[tauri::command]
pub fn create_presentation(
    name: String,
    state: State<'_, AppState>,
) -> CommandResult<crate::models::presentation::Presentation> {
    state
        .with_conn(|conn| crate::bible::repository::PresentationRepository::new(conn).create(&name))
}

#[tauri::command]
pub fn delete_presentation(id: String, state: State<'_, AppState>) -> CommandResult<()> {
    state.with_conn(|conn| crate::bible::repository::PresentationRepository::new(conn).delete(&id))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddItemRequest {
    pub presentation_id: String,
    pub r#type: String,
    pub payload: String,
}

#[tauri::command]
pub fn add_presentation_item(
    request: AddItemRequest,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    state.with_conn(|conn| {
        crate::bible::repository::PresentationRepository::new(conn).add_item(
            &request.presentation_id,
            &crate::models::presentation::PresentationItemRecord {
                id: uuid::Uuid::new_v4().to_string(),
                presentation_id: request.presentation_id.clone(),
                type_name: request.r#type.clone(),
                position: 0, // ignored; repository appends at the end
                payload: request.payload.clone(),
            },
        )
    })
}

#[tauri::command]
pub fn remove_presentation_item(
    presentation_id: String,
    item_id: String,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    state.with_conn(|conn| {
        crate::bible::repository::PresentationRepository::new(conn)
            .remove_item(&presentation_id, &item_id)
    })
}

// ---------------------------------------------------------
// Showing saved content
// ---------------------------------------------------------

/// Shows one saved item on the projector.
///
/// Saved items are stored as JSON so new content kinds never need a schema
/// change; this rebuilds the projectable item from that row and sends it to
/// the presentation window.
#[tauri::command]
pub fn project_saved_item(
    item_id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<crate::models::presentation::PresentationState> {
    let record = state
        .with_conn(|conn| {
            crate::bible::repository::PresentationRepository::new(conn).get_item(&item_id)
        })?
        .ok_or_else(|| AppError::Presentation(format!("item {item_id} no longer exists")))?;

    let item = record.to_item()?;
    state.project(item, &app)?;
    Ok(state.presentation.state())
}

/// Shows a whole saved presentation, starting at its first item.
///
/// Remaining items are queued, so the operator can walk through the whole
/// list with the Next button without touching the editor.
#[tauri::command]
pub fn project_saved_presentation(
    id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<crate::models::presentation::PresentationState> {
    let presentation = state
        .with_conn(|conn| crate::bible::repository::PresentationRepository::new(conn).get(&id))?
        .ok_or_else(|| AppError::Presentation(format!("presentation {id} no longer exists")))?;

    if presentation.items.is_empty() {
        return Err(AppError::Presentation(
            "this presentation is empty — add something to it first".to_string(),
        ));
    }

    let mut items = Vec::with_capacity(presentation.items.len());
    for record in &presentation.items {
        items.push(record.to_item()?);
    }

    // First item goes on screen, the rest wait in the queue.
    let mut remaining = items.into_iter();
    let first = remaining.next().expect("checked for an empty list above");

    state.presentation.clear_queue()?;
    for item in remaining {
        state.presentation.queue(item)?;
    }
    state.project(first, &app)?;

    Ok(state.presentation.state())
}

/// Moves to the next queued item.
#[tauri::command]
pub fn show_next_item(
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<crate::models::presentation::PresentationState> {
    state.presentation.show_next(&app)?;
    Ok(state.presentation.state())
}

/// Returns to the previous item.
#[tauri::command]
pub fn show_previous_item(
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<crate::models::presentation::PresentationState> {
    state.presentation.show_previous(&app)?;
    Ok(state.presentation.state())
}
