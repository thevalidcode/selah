//! Tauri event names and payloads pushed from Rust to React.
//!
//! The frontend subscribes to these names through `@tauri-apps/api/event`.

use serde::Serialize;

use crate::models::content::DetectionResult;
use crate::models::presentation::PresentationItem;

pub const AUDIO_STARTED: &str = "audio://started";
pub const AUDIO_STOPPED: &str = "audio://stopped";

pub const SPEECH_STARTED: &str = "speech://started";
pub const SPEECH_STOPPED: &str = "speech://stopped";
pub const SPEECH_VAD_SEGMENT: &str = "speech://segment";
pub const SPEECH_TRANSCRIPT: &str = "speech://transcript";

pub const CONTENT_DETECTED: &str = "content://detected";

pub const PRESENTATION_CHANGED: &str = "presentation://changed";
pub const PRESENTATION_SETTINGS: &str = "presentation://settings";
pub const PRESENTATION_DISPLAY_OPENED: &str = "presentation://display-opened";
pub const PRESENTATION_DISPLAY_CLOSED: &str = "presentation://display-closed";

/// Payload emitted on `speech://segment` (VAD found speech).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VadSegmentEvent {
    pub start_ms: u64,
    pub end_ms: u64,
    pub duration_ms: u64,
}

/// Payload emitted on `content://detected`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectionEvent {
    pub source: String,
    pub results: Vec<DetectionResult>,
}

/// Payload emitted on `presentation://changed`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresentationChangedEvent {
    pub item: Option<PresentationItem>,
    pub projected: bool,
}

/// Payload emitted on `presentation://settings`.
///
/// The projector window is a separate webview, so it cannot read the operator's
/// form state. This carries the values that change how projected content looks
/// (background, text size, typeface) so a Save in Settings applies immediately.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresentationSettingsEvent {
    pub background: String,
    pub font_size: u32,
    pub font_family: String,
    pub fullscreen: bool,
    pub follow_live: bool,
}

impl From<&crate::models::settings::PresentationSettings> for PresentationSettingsEvent {
    fn from(settings: &crate::models::settings::PresentationSettings) -> Self {
        Self {
            background: settings.background.clone(),
            font_size: settings.font_size,
            font_family: settings.font_family.clone(),
            fullscreen: settings.fullscreen,
            follow_live: settings.follow_live,
        }
    }
}

/// Payload emitted when the presentation display opens/closes.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresentationDisplayEvent {
    pub open: bool,
    pub display: Option<String>,
}
