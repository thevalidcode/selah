//! Speech commands: pipeline status and listening lifecycle.

use tauri::State;

use crate::errors::CommandResult;
use crate::speech::SpeechManagerState;
use crate::state::AppState;

#[tauri::command]
pub fn get_speech_state(state: State<'_, AppState>) -> CommandResult<SpeechManagerState> {
    Ok(state.speech.state())
}

/// Starts microphone capture AND the recognition worker.
#[tauri::command]
pub fn start_listening(state: State<'_, AppState>) -> CommandResult<SpeechManagerState> {
    let device_id = state
        .settings
        .lock()
        .map(|s| s.audio.input_device_id.clone())
        .unwrap_or(None);
    let capture = state.audio.start(device_id.as_deref())?;
    state.speech.start(capture.sample_rate)?;
    Ok(state.speech.state())
}

/// Stops the recognition worker and microphone capture.
#[tauri::command]
pub fn stop_listening(state: State<'_, AppState>) -> CommandResult<SpeechManagerState> {
    state.speech.stop();
    state.audio.stop();
    Ok(state.speech.state())
}

/// Starts microphone capture only (no transcription). Useful for VAD or
/// level monitoring in the UI.
#[tauri::command]
pub fn start_audio_pipeline(
    state: State<'_, AppState>,
) -> CommandResult<crate::audio::AudioCaptureState> {
    let device_id = state
        .settings
        .lock()
        .map(|s| s.audio.input_device_id.clone())
        .unwrap_or(None);
    state.audio.start(device_id.as_deref())
}

/// Stops microphone capture.
#[tauri::command]
pub fn stop_audio_pipeline(state: State<'_, AppState>) -> CommandResult<()> {
    state.audio.stop();
    Ok(())
}
