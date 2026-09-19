//! Settings commands: read/write persisted settings and setup state.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::errors::CommandResult;
use crate::models::settings::{AppSettings, SETUP_COMPLETE_KEY};
use crate::state::AppState;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupState {
    pub completed: bool,
    pub has_translations: bool,
    pub has_model: bool,
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> CommandResult<AppSettings> {
    state
        .settings
        .lock()
        .map(|s| s.clone())
        .map_err(|_| crate::errors::AppError::Internal("settings lock poisoned".to_string()))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSettingsRequest {
    pub settings: AppSettings,
}

#[tauri::command]
pub fn update_settings(
    request: UpdateSettingsRequest,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<AppSettings> {
    // Repair anything that would present badly (a truncated colour, an
    // unreadable text size) before it is stored, so the projector can never be
    // put into a state the operator cannot see their way out of.
    let (presentation, repaired) = request.settings.presentation.sanitized();
    if repaired {
        tracing::warn!("presentation settings were out of range; safe values used");
    }

    let next = AppSettings {
        presentation,
        ..request.settings.clone()
    };

    // Apply speech changes to the live manager (recognizer swap / model load).
    state.speech.reconfigure(&next.speech)?;

    {
        let mut settings = state
            .settings
            .lock()
            .map_err(|_| crate::errors::AppError::Internal("settings lock poisoned".to_string()))?;
        *settings = next.clone();
    }
    state.save_settings()?;

    // Make the save visible: select the screen, follow the fullscreen setting
    // and tell the projector window about the new background, text size and
    // typeface.
    state.apply_presentation_settings(&app)?;

    Ok(next)
}

#[tauri::command]
pub fn get_setup_state(state: State<'_, AppState>) -> CommandResult<SetupState> {
    let completed = state.with_conn(|conn| {
        Ok(crate::bible::repository::SettingsRepository::new(conn)
            .get(SETUP_COMPLETE_KEY)?
            .as_deref()
            == Some("1"))
    })?;

    let has_translations = state.with_conn(|conn| {
        Ok(!crate::bible::repository::BibleRepository::new(conn)
            .list_translations()?
            .is_empty())
    })?;

    let has_model = state
        .settings
        .lock()
        .map(|s| !s.speech.model_path.as_deref().unwrap_or("").is_empty())
        .unwrap_or(false);

    Ok(SetupState {
        completed,
        has_translations,
        has_model,
    })
}

#[tauri::command]
pub fn complete_setup(state: State<'_, AppState>) -> CommandResult<()> {
    state.with_conn(|conn| {
        crate::bible::repository::SettingsRepository::new(conn).set(SETUP_COMPLETE_KEY, "1")
    })
}
