//! Typed application errors.
//!
//! Domain logic should return [`AppError`]. Tauri commands convert it to a
//! safe, serialized payload for the frontend — technical detail is logged
//! internally and never leaked to the UI.

use serde::Serialize;

/// Application-level error type shared across every module.
///
/// `thiserror` derives the `Display` impl used for internal logging; the
/// manual `Serialize` impl controls exactly what the frontend sees.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("audio device not found: {0}")]
    AudioDeviceNotFound(String),

    #[error("audio device query failed: {0}")]
    AudioQueryFailed(String),

    #[error("audio capture failed: {0}")]
    AudioCaptureFailed(String),

    #[error("speech model not found: {0}")]
    SpeechModelNotFound(String),

    #[error("speech recognition failed: {0}")]
    SpeechRecognitionFailed(String),

    #[error("voice activity detection failed: {0}")]
    VadFailed(String),

    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("bible translation not found: {0}")]
    BibleTranslationNotFound(String),

    #[error("scripture not found in translation '{translation}': {reference}")]
    ScriptureNotFound {
        translation: String,
        reference: String,
    },

    #[error("invalid scripture reference: {0}")]
    InvalidScriptureReference(String),

    #[error("display not found: {0}")]
    DisplayNotFound(String),

    #[error("presentation error: {0}")]
    Presentation(String),

    #[error("invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("media error: {0}")]
    Media(String),

    #[error("the requested operation is unavailable in this build: {0}")]
    Unavailable(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl AppError {
    /// Serialized payload shown to the frontend.
    ///
    /// Only the human-readable message and a stable machine-readable kind are
    /// included. No internal stack traces, paths, or source locations.
    pub fn to_frontend(&self) -> ErrorPayload {
        ErrorPayload {
            kind: self.kind().to_string(),
            message: self.to_string(),
        }
    }

    /// Stable machine-readable error kind (camelCase).
    fn kind(&self) -> &'static str {
        match self {
            AppError::AudioDeviceNotFound(_) => "audioDeviceNotFound",
            AppError::AudioQueryFailed(_) => "audioQueryFailed",
            AppError::AudioCaptureFailed(_) => "audioCaptureFailed",
            AppError::SpeechModelNotFound(_) => "speechModelNotFound",
            AppError::SpeechRecognitionFailed(_) => "speechRecognitionFailed",
            AppError::VadFailed(_) => "vadFailed",
            AppError::Database(_) => "databaseError",
            AppError::BibleTranslationNotFound(_) => "bibleTranslationNotFound",
            AppError::ScriptureNotFound { .. } => "scriptureNotFound",
            AppError::InvalidScriptureReference(_) => "invalidScriptureReference",
            AppError::DisplayNotFound(_) => "displayNotFound",
            AppError::Presentation(_) => "presentationError",
            AppError::InvalidConfiguration(_) => "invalidConfiguration",
            AppError::Media(_) => "mediaError",
            AppError::Unavailable(_) => "unavailable",
            AppError::Internal(_) => "internalError",
        }
    }
}

/// The serializable shape of an error handed to React.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorPayload {
    pub kind: String,
    pub message: String,
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_frontend().serialize(serializer)
    }
}

/// Convenience alias used by commands.
pub type CommandResult<T> = Result<T, AppError>;
