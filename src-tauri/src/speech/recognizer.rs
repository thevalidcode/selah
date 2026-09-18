//! Speech recognition boundary.
//!
//! Everything outside this module depends only on [`SpeechRecognizer`] and
//! [`crate::speech::Transcript`]. Whisper specifics stay in `whisper.rs`.

use crate::errors::AppError;

use super::transcript::Transcript;

/// Abstraction for local speech recognition.
///
/// Implementations may be real (whisper.cpp) or development mocks. Nothing in
/// the rest of the application may depend on an implementation type.
pub trait SpeechRecognizer: Send {
    /// Loads a speech model from disk. Fails cleanly if the file is missing
    /// or unreadable.
    fn load_model(&mut self, model_path: &str) -> Result<(), AppError>;

    /// Transcribes raw PCM mono audio at the given sample rate.
    fn transcribe(&mut self, audio: &[f32], sample_rate: u32) -> Result<Transcript, AppError>;

    /// Whether a model is currently loaded.
    fn is_loaded(&self) -> bool;

    /// Stable identifier of this implementation ("whisper", "mock").
    fn id(&self) -> &'static str;
}

/// Creates the recognizer chosen in settings.
pub fn recognizer_for(kind: crate::models::settings::RecognizerKind) -> Box<dyn SpeechRecognizer> {
    match kind {
        crate::models::settings::RecognizerKind::Mock => {
            Box::new(super::mock::MockSpeechRecognizer)
        }
        #[cfg(feature = "whisper")]
        crate::models::settings::RecognizerKind::Whisper => {
            Box::new(super::whisper::WhisperRecognizer::default())
        }
        #[cfg(not(feature = "whisper"))]
        crate::models::settings::RecognizerKind::Whisper => {
            tracing::warn!("whisper recognizer requested but the 'whisper' cargo feature is disabled; falling back to mock");
            Box::new(super::mock::MockSpeechRecognizer)
        }
    }
}
