//! Audio commands: device discovery and capture lifecycle.

use tauri::{AppHandle, Emitter, State};

use crate::audio::AudioDeviceInfo;
use crate::errors::CommandResult;
use crate::events;
use crate::state::AppState;

#[tauri::command]
pub fn list_audio_devices(
    _app: AppHandle,
    _state: State<'_, AppState>,
) -> CommandResult<Vec<AudioDeviceInfo>> {
    crate::audio::device::list_input_devices()
}

#[tauri::command]
pub fn get_default_audio_device(
    _app: AppHandle,
    _state: State<'_, AppState>,
) -> CommandResult<Option<AudioDeviceInfo>> {
    crate::audio::device::default_input_device()
}

#[tauri::command]
pub fn start_audio_capture(
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<crate::audio::AudioCaptureState> {
    let device_id = state
        .settings
        .lock()
        .map(|s| s.audio.input_device_id.clone())
        .unwrap_or(None);
    let capture = state.audio.start(device_id.as_deref())?;
    let _ = app.emit(events::AUDIO_STARTED, &capture);
    Ok(capture)
}

#[tauri::command]
pub fn stop_audio_capture(app: AppHandle, state: State<'_, AppState>) -> CommandResult<()> {
    state.audio.stop();
    let _ = app.emit(events::AUDIO_STOPPED, ());
    Ok(())
}

#[tauri::command]
pub fn get_audio_capture_state(
    _app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<crate::audio::AudioCaptureState> {
    Ok(state.audio.state())
}
