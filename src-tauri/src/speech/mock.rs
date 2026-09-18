//! Development speech recognizer.
//!
//! This recognizer intentionally returns an EMPTY transcript. It is a
//! placeholder that keeps the microphone → VAD → recognizer pipeline
//! runnable without a model or cmake; it never fabricates recognizable text,
//! so nothing on the projector can be produced from mock output.

use crate::errors::AppError;

use super::recognizer::SpeechRecognizer;
use super::transcript::{Transcript, TranscriptSource};

pub struct MockSpeechRecognizer;

impl SpeechRecognizer for MockSpeechRecognizer {
    fn load_model(&mut self, model_path: &str) -> Result<(), AppError> {
        tracing::debug!(model_path, "mock recognizer: load_model accepted (no-op)");
        Ok(())
    }

    fn transcribe(&mut self, audio: &[f32], sample_rate: u32) -> Result<Transcript, AppError> {
        tracing::debug!(
            samples = audio.len(),
            sample_rate,
            "mock recognizer: transcription is intentionally disabled in this build"
        );
        Ok(Transcript {
            text: String::new(),
            segments: Vec::new(),
            source: TranscriptSource::Mock,
            sample_count: audio.len(),
        })
    }

    fn is_loaded(&self) -> bool {
        true
    }

    fn id(&self) -> &'static str {
        "mock"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_never_invents_text() {
        let mut recognizer = MockSpeechRecognizer;
        let audio = vec![0.5; 16_000];
        let transcript = recognizer.transcribe(&audio, 16_000).unwrap();
        assert!(transcript.text.is_empty());
        assert!(transcript.segments.is_empty());
        assert_eq!(transcript.source, TranscriptSource::Mock);
    }
}
