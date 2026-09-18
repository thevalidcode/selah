//! whisper.cpp recognition through whisper-rs.
//!
//! This module is compiled only with the `whisper` cargo feature because
//! whisper-rs compiles the whisper.cpp C++ codebase (requires `cmake` and a
//! C++ toolchain). The default bootstrap build uses `MockSpeechRecognizer`;
//! THIS file is the place where real recognition becomes available.
//!
//! No model is ever downloaded automatically — the operator provides a
//! `ggml-*.bin` file in the models directory.

#![cfg(feature = "whisper")]

use std::path::Path;

use crate::errors::AppError;
use crate::models::settings::SpeechSettings;
use whisper_rs::{SamplingStrategy, WhisperContext, WhisperContextParameters};

use super::recognizer::SpeechRecognizer;
use super::transcript::{Transcript, TranscriptSegment, TranscriptSource};

/// Whisper recognizer built around whisper.cpp.
pub struct WhisperRecognizer {
    context: Option<WhisperContext>,
    settings: SpeechSettings,
}

impl Default for WhisperRecognizer {
    fn default() -> Self {
        Self {
            context: None,
            settings: SpeechSettings {
                model_path: None,
                language: None,
                threads: 4,
                ..SpeechSettings::default()
            },
        }
    }
}

impl WhisperRecognizer {
    pub fn with_settings(settings: SpeechSettings) -> Self {
        Self {
            context: None,
            settings,
        }
    }
}

impl SpeechRecognizer for WhisperRecognizer {
    fn load_model(&mut self, model_path: &str) -> Result<(), AppError> {
        if !Path::new(model_path).is_file() {
            return Err(AppError::SpeechModelNotFound(model_path.to_string()));
        }

        let mut params = WhisperContextParameters::default();
        params.use_gpu(false); // CPU inference: keeps Intel macs happy
        params.flash_attn(false);

        let context = WhisperContext::new_with_params(model_path, params)
            .map_err(|e| AppError::SpeechRecognitionFailed(e.to_string()))?;

        tracing::info!(model = %model_path, "whisper model loaded");
        self.context = Some(context);
        Ok(())
    }

    fn transcribe(&mut self, audio: &[f32], sample_rate: u32) -> Result<Transcript, AppError> {
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| AppError::SpeechModelNotFound("no model loaded".into()))?;

        if sample_rate != 16_000 {
            return Err(AppError::InvalidConfiguration(format!(
                "whisper expects 16 kHz audio, got {sample_rate} Hz"
            )));
        }

        let mut state = context
            .create_state()
            .map_err(|e| AppError::SpeechRecognitionFailed(e.to_string()))?;

        let mut params = whisper_rs::FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_n_threads(self.settings.threads as i32);
        params.set_language(self.settings.language.as_deref());
        params.set_single_segment(false);
        params.set_no_speech_thold(0.6);

        state
            .full(params, audio)
            .map_err(|e| AppError::SpeechRecognitionFailed(e.to_string()))?;

        let n = state.full_n_segments();
        let mut segments = Vec::with_capacity(n.max(0) as usize);
        let mut parts = Vec::new();
        for i in 0..n {
            let Some(segment) = state.get_segment(i) else {
                continue;
            };
            let text = segment.to_str_lossy().unwrap_or_default().to_string();
            // whisper.cpp reports timestamps in centiseconds.
            let start_ms = (segment.start_timestamp().max(0) as u64) * 10;
            let end_ms = (segment.end_timestamp().max(0) as u64) * 10;
            let no_speech = segment.no_speech_probability();
            let confidence = if no_speech > 0.9 {
                None
            } else {
                Some(1.0 - no_speech)
            };
            parts.push(text.clone());
            segments.push(TranscriptSegment {
                text,
                start_ms,
                end_ms,
                confidence,
            });
        }

        tracing::info!(segments = segments.len(), "transcript generated");
        Ok(Transcript {
            text: parts.join(" ").trim().to_string(),
            segments,
            source: TranscriptSource::Whisper,
            sample_count: audio.len(),
        })
    }

    fn is_loaded(&self) -> bool {
        self.context.is_some()
    }

    fn id(&self) -> &'static str {
        "whisper"
    }
}
