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

/// Payload emitted when the presentation display opens/closes.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresentationDisplayEvent {
    pub open: bool,
    pub display: Option<String>,
}
