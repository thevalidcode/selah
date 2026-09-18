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
    state: State<'_, AppState>,
) -> CommandResult<AppSettings> {
    // Apply audio device choice eagerly so capture can restart cleanly.
    {
        let mut settings = state
            .settings
            .lock()
            .map_err(|_| crate::errors::AppError::Internal("settings lock poisoned".to_string()))?;
        *settings = request.settings.clone();
    }
    state.save_settings()?;

    // Apply speech changes to the live manager (recognizer swap / model load).
    let speech_settings = request.settings.speech.clone();
    state.speech.reconfigure(&speech_settings)?;

    if let Ok(mut settings) = state.settings.lock() {
        if let Some(display) = request.settings.presentation.display_index {
            let _ = state.display.set_target(display);
            settings.presentation.display_index = Some(display);
        }
    }
    state.save_settings()?;

    Ok(request.settings)
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
