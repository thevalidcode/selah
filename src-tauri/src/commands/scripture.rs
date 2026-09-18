//! Scripture commands: deterministic detection over text.

use tauri::State;

use crate::errors::CommandResult;
use crate::models::content::{ContentDetector, DetectionResult};
use crate::state::AppState;

#[tauri::command]
pub fn detect_scripture(
    transcript: String,
    _state: State<'_, AppState>,
) -> CommandResult<Vec<DetectionResult>> {
    Ok(ContentDetector.detect(&transcript))
}
