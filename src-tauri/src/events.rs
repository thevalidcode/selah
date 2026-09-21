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

/// Operator → projector: start, pause, restart or stop what is playing.
pub const MEDIA_PLAYBACK: &str = "media://playback";
/// Projector → everyone: what is actually playing right now.
pub const MEDIA_PLAYBACK_STATE: &str = "media://playback-state";

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

/// Payload of `presentation://settings` — how projected content should look.
///
/// The projector window is a separate webview, so it cannot read the operator's
/// form state. This carries the values that change how projected content looks
/// (background, text size, typeface, branding) so a Save in Settings applies
/// immediately.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresentationSettingsEvent {
    pub background: String,
    pub font_size: u32,
    pub font_family: String,
    pub fullscreen: bool,
    pub follow_live: bool,
    /// The operator's logo and line of text, drawn on every projected item.
    pub branding: crate::models::settings::BrandingSettings,
    /// Whether a video starts again when it reaches the end. Carried to the
    /// projector so a Save changes a video that is on the screen right now,
    /// instead of only the next one.
    pub repeat_videos: bool,
}

impl From<&crate::models::settings::PresentationSettings> for PresentationSettingsEvent {
    fn from(settings: &crate::models::settings::PresentationSettings) -> Self {
        Self {
            background: settings.background.clone(),
            font_size: settings.font_size,
            font_family: settings.font_family.clone(),
            fullscreen: settings.fullscreen,
            follow_live: settings.follow_live,
            branding: settings.branding.clone(),
            repeat_videos: settings.repeat_videos,
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

/// What the operator asked the projector's player to do.
///
/// A video (or a piece of music) is the one kind of content that keeps moving
/// after it is on the screen, so the operator needs a way to start it again,
/// hold it, or send it back to the beginning without leaving their seat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PlaybackAction {
    Play,
    Pause,
    /// Back to the start and playing again.
    Restart,
    /// Back to the start and held there.
    Stop,
}

impl PlaybackAction {
    /// Parses the action sent from the interface.
    pub fn parse(value: &str) -> Option<Self> {
        Some(match value.trim().to_lowercase().as_str() {
            "play" => PlaybackAction::Play,
            "pause" => PlaybackAction::Pause,
            "restart" => PlaybackAction::Restart,
            "stop" => PlaybackAction::Stop,
            _ => return None,
        })
    }
}

/// Payload of `media://playback`, sent from the operator screen to the
/// projector window.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaPlaybackCommand {
    pub action: PlaybackAction,
    /// The item the action is about, so a stale command cannot affect a
    /// different picture or video that is already on the screen.
    pub item_id: Option<String>,
}

/// Payload of `media://playback-state`: what is actually playing.
///
/// The projector window is the only place that knows whether a video really
/// started — the operator screen cannot see the congregation's screen — so it
/// reports back and this state is shown in the interface.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaPlaybackState {
    /// The media item on screen, when it is media.
    pub item_id: Option<String>,
    pub playing: bool,
    pub position_ms: u64,
    pub duration_ms: u64,
    /// A video that has run to its end (unless it is looping).
    pub ended: bool,
}

impl MediaPlaybackState {
    /// The state for a new item that has not reported anything yet.
    pub fn for_item(item_id: Option<String>) -> Self {
        Self {
            item_id,
            ..Self::default()
        }
    }

    /// Whether the state describes `item_id`.
    pub fn describes(&self, item_id: &str) -> bool {
        self.item_id.as_deref() == Some(item_id)
    }

    /// Seconds of the total, for a progress readout.
    pub fn position_seconds(&self) -> f64 {
        self.position_ms as f64 / 1000.0
    }
}
