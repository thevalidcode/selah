//! Moonshine speech recognition.
//!
//! Layout:
//!   * [`engine`]    — ONNX Runtime sessions + greedy decoding
//!   * [`tokenizer`] — token ids back to text
//!   * [`runtime`]   — locates the ONNX Runtime shared library
//!
//! A Moonshine "model" is a directory, not a single file:
//!
//! ```text
//! models/moonshine/
//!   encoder_model_int8.onnx        (or encoder_model.onnx)
//!   decoder_model_int8.onnx        (or decoder_model.onnx)
//!   decoder_with_past_model_int8.onnx   (optional: cached decoding only)
//!   tokenizer.json
//! ```
//!
//! Nothing is downloaded automatically — the operator supplies the folder.

pub mod engine;
pub mod runtime;
pub mod tokenizer;

use std::path::PathBuf;

use crate::errors::AppError;

use super::recognizer::SpeechRecognizer;
use super::transcript::{Transcript, TranscriptSegment, TranscriptSource};

use engine::MoonshineEngine;

/// Moonshine is trained on 16 kHz mono audio.
const EXPECTED_SAMPLE_RATE: u32 = 16_000;

/// Real speech-to-text using a local Moonshine model.
#[derive(Default)]
pub struct MoonshineRecognizer {
    engine: Option<MoonshineEngine>,
    /// Folder the model was loaded from, kept for diagnostics.
    model_dir: Option<PathBuf>,
}

impl MoonshineRecognizer {
    /// Folder currently in use, if a model is loaded.
    pub fn model_dir(&self) -> Option<&std::path::Path> {
        self.model_dir.as_deref()
    }
}

impl SpeechRecognizer for MoonshineRecognizer {
    fn load_model(&mut self, model_path: &str) -> Result<(), AppError> {
        let dir = PathBuf::from(model_path);
        if !dir.is_dir() {
            return Err(AppError::SpeechModelNotFound(format!(
                "{} is not a folder. Moonshine expects a folder holding \
                 encoder_model.onnx, decoder_model.onnx and tokenizer.json.",
                dir.display()
            )));
        }

        // Locate ONNX Runtime before any session is created, so a missing
        // runtime produces one clear message instead of an opaque link error.
        runtime::ensure_available(None)?;

        let engine = MoonshineEngine::load(&dir)?;
        tracing::info!(
            dir = %dir.display(),
            vocab = engine.vocab_len(),
            "moonshine recognizer ready"
        );

        self.model_dir = Some(dir);
        self.engine = Some(engine);
        Ok(())
    }

    fn transcribe(&mut self, audio: &[f32], sample_rate: u32) -> Result<Transcript, AppError> {
        if sample_rate != EXPECTED_SAMPLE_RATE {
            return Err(AppError::InvalidConfiguration(format!(
                "Moonshine expects {EXPECTED_SAMPLE_RATE} Hz audio, got {sample_rate} Hz"
            )));
        }

        let Some(engine) = self.engine.as_mut() else {
            return Err(AppError::SpeechModelNotFound(
                "no Moonshine model loaded".to_string(),
            ));
        };

        let text = engine.transcribe(audio)?.trim().to_string();
        let duration_ms = audio.len() as u64 * 1000 / u64::from(sample_rate);

        // Moonshine returns a single utterance with no word timings, so the
        // whole segment is reported as one span.
        let segments = if text.is_empty() {
            Vec::new()
        } else {
            vec![TranscriptSegment {
                text: text.clone(),
                start_ms: 0,
                end_ms: duration_ms,
                confidence: None,
            }]
        };

        tracing::debug!(
            samples = audio.len(),
            chars = text.len(),
            "moonshine transcription complete"
        );

        Ok(Transcript {
            text,
            segments,
            source: TranscriptSource::Moonshine,
            sample_count: audio.len(),
        })
    }

    fn is_loaded(&self) -> bool {
        self.engine.is_some()
    }

    fn id(&self) -> &'static str {
        "moonshine"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn without_a_model_it_reports_not_loaded() {
        let recognizer = MoonshineRecognizer::default();
        assert!(!recognizer.is_loaded());
        assert_eq!(recognizer.id(), "moonshine");
    }

    #[test]
    fn transcribing_without_a_model_is_an_error_not_a_panic() {
        let mut recognizer = MoonshineRecognizer::default();
        let err = recognizer
            .transcribe(&vec![0.0; 16_000], EXPECTED_SAMPLE_RATE)
            .unwrap_err();
        assert!(matches!(err, AppError::SpeechModelNotFound(_)));
    }

    #[test]
    fn rejecting_a_file_path_explains_the_folder_requirement() {
        let mut recognizer = MoonshineRecognizer::default();
        let err = recognizer.load_model("/tmp/not-a-folder.bin").unwrap_err();
        let message = err.to_string();
        assert!(message.contains("folder"), "got: {message}");
    }
}
