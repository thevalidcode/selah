//! Moonshine transcription on top of ONNX Runtime.
//!
//! Moonshine is an encoder/decoder transformer: the encoder turns 16 kHz mono
//! audio into hidden states, then the decoder emits text tokens one at a time
//! until it produces an end-of-sequence token.
//!
//! ## Why this is hand-written
//!
//! The popular wrapper crates expect Moonshine's *merged* decoder graph, which
//! takes a `use_cache_branch` input. The models published for
//! Transformers.js / optimum — the ones Selah ships — are the *split* export
//! (`encoder_model`, `decoder_model`, `decoder_with_past_model`) and contain no
//! such input, so they cannot be renamed into the merged layout.
//!
//! ## Decoding strategy
//!
//! This implementation re-runs the plain decoder over the whole token sequence
//! at each step and takes the final position's logits. That is the simplest
//! correct formulation: it needs no key/value cache and therefore no
//! assumptions about cache tensor names or shapes, which vary between exports.
//! Speech segments handed to it by the voice-activity detector are short
//! (typically a few seconds), so the extra work is not noticeable.
//!
//! `decoder_with_past_model` is consequently optional: it exists only to speed
//! up cached decoding.

use std::borrow::Cow;
use std::path::{Path, PathBuf};

use ndarray::{Array2, ArrayD};
use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::{DynValue, Value};

use crate::errors::AppError;

use super::tokenizer::MoonshineTokenizer;

/// First token fed to the decoder (`<s>`).
const DECODER_START_TOKEN: i64 = 1;
/// Token that ends generation (`</s>`).
const EOS_TOKEN: i64 = 2;
/// Moonshine English models emit roughly six tokens per second of audio.
const TOKENS_PER_SECOND: f32 = 6.0;
/// Absolute ceiling on generated tokens, whatever the duration suggests.
const MAX_NEW_TOKENS: usize = 448;

/// A loaded Moonshine model, ready to transcribe.
pub struct MoonshineEngine {
    encoder: Session,
    decoder: Session,
    /// Input names each graph declares. Different exports differ (some
    /// encoders take only the waveform, some also an attention mask), so the
    /// engine feeds exactly what each one asks for.
    encoder_inputs: Vec<String>,
    decoder_inputs: Vec<String>,
    /// Name of the encoder's hidden-state output. Exports differ here too:
    /// Transformer.js writes `last_hidden_state`, the optimum CLI writes
    /// `encoder_hidden_states`.
    encoder_output: String,
    /// Name of the decoder's token-score output (normally `logits`).
    decoder_logits: String,
    tokenizer: MoonshineTokenizer,
    /// Folder the model was loaded from, for logging and diagnostics.
    pub loaded_from: PathBuf,
}

impl MoonshineEngine {
    /// Loads encoder, decoder and tokenizer from a model directory.
    pub fn load(model_dir: &Path) -> Result<Self, AppError> {
        let encoder_path = resolve_model_file(model_dir, "encoder_model")?;
        let decoder_path = resolve_model_file(model_dir, "decoder_model")?;

        let tokenizer = MoonshineTokenizer::load(model_dir)?;

        let encoder = Self::open_session(&encoder_path)?;
        let decoder = Self::open_session(&decoder_path)?;

        // A graph that demands past key/values would need the cached decoder
        // path, which this implementation deliberately does not use.
        if let Some(cached) = decoder
            .inputs()
            .iter()
            .map(|input| input.name())
            .find(|name| name.starts_with("past_") || *name == "use_cache_branch")
        {
            return Err(AppError::InvalidConfiguration(format!(
                "{} is a cached decoder (it expects `{cached}`). Point Selah at the \
                 split export whose decoder takes only input_ids and \
                 encoder_hidden_states.",
                decoder_path.display()
            )));
        }

        tracing::info!(
            encoder = %encoder_path.display(),
            decoder = %decoder_path.display(),
            vocab = tokenizer.len(),
            "moonshine model loaded"
        );

        let encoder_inputs = input_names(&encoder);
        let decoder_inputs = input_names(&decoder);
        let encoder_output = hidden_state_output(&encoder).ok_or_else(|| {
            AppError::InvalidConfiguration(format!(
                "{} declares no outputs",
                encoder_path.display()
            ))
        })?;
        let decoder_logits = logits_output(&decoder).ok_or_else(|| {
            AppError::InvalidConfiguration(format!(
                "{} declares no outputs",
                decoder_path.display()
            ))
        })?;

        tracing::debug!(
            encoder_inputs = ?encoder_inputs,
            decoder_inputs = ?decoder_inputs,
            encoder_output = %encoder_output,
            decoder_logits = %decoder_logits,
            "model graph requirements"
        );

        Ok(Self {
            encoder_inputs,
            decoder_inputs,
            encoder_output,
            decoder_logits,
            encoder,
            decoder,
            tokenizer,
            loaded_from: model_dir.to_path_buf(),
        })
    }

    /// Vocabulary size, exposed for sanity checks.
    pub fn vocab_len(&self) -> usize {
        self.tokenizer.len()
    }

    /// Builds an ONNX Runtime session with conservative CPU threading.
    fn open_session(path: &Path) -> Result<Session, AppError> {
        let threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);

        // Each builder step reports its own error type, so they are mapped
        // individually instead of chained with `and_then`.
        let mut builder = Session::builder()
            .map_err(|e| open_error(path, e))?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| open_error(path, e))?
            .with_intra_threads(threads)
            .map_err(|e| open_error(path, e))?;

        builder
            .commit_from_file(path)
            .map_err(|e| open_error(path, e))
    }

    /// Runs the encoder over 16 kHz mono audio.
    fn encode(&mut self, samples: &[f32]) -> Result<ArrayD<f32>, AppError> {
        let audio = Array2::<f32>::from_shape_vec((1, samples.len()), samples.to_vec())
            .map_err(|e| AppError::SpeechRecognitionFailed(format!("bad audio buffer: {e}")))?;
        let attention_mask = Array2::<i64>::ones((1, samples.len()));

        // Feed exactly the inputs this export declares: some encoders take
        // only the waveform, others also want an attention mask.
        let mut inputs: Vec<(Cow<'_, str>, DynValue)> = Vec::new();
        for name in self.encoder_inputs.clone() {
            let value = if name.contains("attention_mask") {
                Value::from_array(attention_mask.clone())
                    .map_err(ort_error)?
                    .into_dyn()
            } else {
                Value::from_array(audio.clone())
                    .map_err(ort_error)?
                    .into_dyn()
            };
            inputs.push((Cow::Owned(name), value));
        }

        let outputs = self.encoder.run(inputs).map_err(ort_error)?;
        let hidden = outputs
            .get(self.encoder_output.as_str())
            .ok_or_else(|| missing_output(&self.encoder_output))?;
        let (shape, data) = hidden.try_extract_tensor::<f32>().map_err(ort_error)?;

        ArrayD::from_shape_vec(shape.to_ixdyn(), data.to_vec()).map_err(|e| {
            AppError::SpeechRecognitionFailed(format!("unexpected encoder output shape: {e}"))
        })
    }

    /// Greedily generates text tokens for one encoded segment.
    fn generate(&mut self, hidden: &ArrayD<f32>, audio_len: usize) -> Result<Vec<i64>, AppError> {
        let attention_mask = Array2::<i64>::ones((1, audio_len));
        let budget = max_new_tokens(audio_len);

        // Generation always starts with the begin-of-sequence token.
        let mut ids: Vec<i64> = vec![DECODER_START_TOKEN];
        let mut produced: Vec<i64> = Vec::new();

        for _ in 0..budget {
            let input_ids = Array2::<i64>::from_shape_vec((1, ids.len()), ids.clone())
                .map_err(|e| AppError::SpeechRecognitionFailed(format!("bad token buffer: {e}")))?;

            let inputs: Vec<(Cow<'_, str>, DynValue)> = {
                let mut inputs: Vec<(Cow<'_, str>, DynValue)> = Vec::new();
                for name in self.decoder_inputs.clone() {
                    // Names vary by export: `input_ids` or `decoder_input_ids`;
                    // `encoder_attention_mask` or `attention_mask`.
                    let value = if name.contains("input_ids") {
                        Value::from_array(input_ids.clone())
                            .map_err(ort_error)?
                            .into_dyn()
                    } else if name.contains("attention_mask") {
                        Value::from_array(attention_mask.clone())
                            .map_err(ort_error)?
                            .into_dyn()
                    } else {
                        Value::from_array(hidden.clone())
                            .map_err(ort_error)?
                            .into_dyn()
                    };
                    inputs.push((Cow::Owned(name), value));
                }
                inputs
            };

            let outputs = self.decoder.run(inputs).map_err(ort_error)?;
            let logits = outputs
                .get(self.decoder_logits.as_str())
                .ok_or_else(|| missing_output(&self.decoder_logits))?;
            let (shape, data) = logits.try_extract_tensor::<f32>().map_err(ort_error)?;

            // logits are [batch, sequence, vocabulary]; only the final
            // position predicts the next token.
            let sequence = shape[1] as usize;
            let vocab = shape[2] as usize;
            if sequence == 0 || vocab == 0 {
                break;
            }
            let start = sequence.saturating_sub(1) * vocab;
            let Some(row) = data.get(start..start + vocab) else {
                break;
            };

            let next = argmax(row);
            if next == EOS_TOKEN {
                break;
            }
            produced.push(next);
            ids.push(next);
        }

        Ok(produced)
    }

    /// Transcribes one segment of 16 kHz mono audio into text.
    pub fn transcribe(&mut self, samples: &[f32]) -> Result<String, AppError> {
        if samples.is_empty() {
            return Ok(String::new());
        }
        let hidden = self.encode(samples)?;
        let tokens = self.generate(&hidden, samples.len())?;
        Ok(self.tokenizer.decode(&tokens))
    }
}

/// Maps `ort` failures onto the application's speech error.
fn ort_error(error: ort::Error) -> AppError {
    AppError::SpeechRecognitionFailed(error.to_string())
}

/// Names of the inputs a graph declares, in order.
fn input_names(session: &Session) -> Vec<String> {
    session
        .inputs()
        .iter()
        .map(|input| input.name().to_string())
        .collect()
}

/// Finds the encoder's hidden-state output.
///
/// Moonshine exports disagree on the name — Transformer.js writes
/// `last_hidden_state`, the optimum CLI writes `encoder_hidden_states` — so
/// both are accepted, falling back to whatever the graph declares first.
fn hidden_state_output(session: &Session) -> Option<String> {
    preferred_output(session, &["encoder_hidden_states", "last_hidden_state"])
}

/// Finds the decoder's token-score output.
fn logits_output(session: &Session) -> Option<String> {
    preferred_output(session, &["logits"])
}

/// Returns the first of `preferred` the graph actually declares, else its
/// first output.
fn preferred_output(session: &Session, preferred: &[&str]) -> Option<String> {
    let names: Vec<&str> = session
        .outputs()
        .iter()
        .map(|output| output.name())
        .collect();

    preferred
        .iter()
        .find(|wanted| names.contains(wanted))
        .map(|name| (*name).to_string())
        .or_else(|| names.first().map(|name| (*name).to_string()))
}

/// Error raised when a model graph cannot be opened.
fn open_error(path: &Path, error: impl std::fmt::Display) -> AppError {
    AppError::SpeechRecognitionFailed(format!("cannot open {}: {error}", path.display()))
}

/// Error raised when a graph lacks an output we rely on.
fn missing_output(name: &str) -> AppError {
    AppError::SpeechRecognitionFailed(format!("model produced no `{name}` output"))
}

/// How many tokens to allow for `audio_len` samples of 16 kHz audio.
fn max_new_tokens(audio_len: usize) -> usize {
    let seconds = audio_len as f32 / 16_000.0;
    let expected = (seconds * TOKENS_PER_SECOND).ceil() as usize;
    // Moonshine occasionally emits a few extra tokens; pad a little rather
    // than truncating a sentence, while still bounding runaway generation.
    expected.saturating_add(8).clamp(8, MAX_NEW_TOKENS)
}

/// Index of the largest value in `scores`.
fn argmax(scores: &[f32]) -> i64 {
    let mut best_index = 0usize;
    let mut best = f32::NEG_INFINITY;
    for (index, &score) in scores.iter().enumerate() {
        if score > best {
            best = score;
            best_index = index;
        }
    }
    best_index as i64
}

/// Finds the best available `{base}*.onnx` in `dir`.
///
/// `_quantized` is preferred over `_int8`: both are 8-bit, but a
/// dynamically-quantised export contains `ConvInteger` nodes that ONNX
/// Runtime's CPU provider cannot execute, whereas the QDQ (`_quantized`)
/// export runs everywhere.
fn resolve_model_file(dir: &Path, base: &str) -> Result<PathBuf, AppError> {
    let mut tried = Vec::new();
    for suffix in ["_quantized", ".quantized", "_int8", ".int8", ""] {
        let candidate = dir.join(format!("{base}{suffix}.onnx"));
        if candidate.is_file() {
            return Ok(candidate);
        }
        tried.push(candidate);
    }

    let list = tried
        .iter()
        .map(|path| format!("  - {}", path.display()))
        .collect::<Vec<_>>()
        .join("\n");

    Err(AppError::SpeechModelNotFound(format!(
        "no {base} model in {}. Looked for:\n{list}",
        dir.display()
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn argmax_returns_highest_scoring_index() {
        assert_eq!(argmax(&[0.1, 0.9, 0.3]), 1);
        assert_eq!(argmax(&[5.0, -1.0]), 0);
        assert_eq!(argmax(&[1.0, 2.0, 2.0]), 1);
    }

    #[test]
    fn token_budget_scales_with_audio_length() {
        // 1 second of audio -> ~6 tokens, plus headroom.
        assert_eq!(max_new_tokens(16_000), 14);
        // 0.1 seconds -> a single token, but still with the full headroom.
        assert_eq!(max_new_tokens(1_600), 9);
        // Long audio is capped rather than left unbounded.
        assert_eq!(max_new_tokens(16_000 * 600), MAX_NEW_TOKENS);
    }

    #[test]
    fn resolve_prefers_qdq_over_dynamic_int8() {
        // A dynamic-int8 encoder is unusable on a CPU, so the QDQ export wins
        // even though both are 8-bit.
        let dir = scratch_dir("prefers_qdq");
        std::fs::create_dir_all(&dir).unwrap();
        for name in [
            "encoder_model.onnx",
            "encoder_model_int8.onnx",
            "encoder_model_quantized.onnx",
        ] {
            std::fs::write(dir.join(name), b"x").unwrap();
        }

        let found = resolve_model_file(&dir, "encoder_model").unwrap();
        assert!(found.ends_with("encoder_model_quantized.onnx"));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn resolve_prefers_quantised_over_full_precision() {
        let dir = scratch_dir("prefers_int8");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("encoder_model.onnx"), b"x").unwrap();
        std::fs::write(dir.join("encoder_model_int8.onnx"), b"x").unwrap();

        let found = resolve_model_file(&dir, "encoder_model").unwrap();
        assert!(found.ends_with("encoder_model_int8.onnx"));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn resolve_falls_back_to_unquantised_files() {
        let dir = scratch_dir("falls_back");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("encoder_model.onnx"), b"x").unwrap();

        let found = resolve_model_file(&dir, "encoder_model").unwrap();
        assert!(found.ends_with("encoder_model.onnx"));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn resolve_lists_candidates_when_nothing_matches() {
        let dir = scratch_dir("missing");
        std::fs::create_dir_all(&dir).unwrap();

        let err = resolve_model_file(&dir, "encoder_model").unwrap_err();
        let message = err.to_string();
        assert!(message.contains("encoder_model_int8.onnx"));
        assert!(message.contains("encoder_model.onnx"));

        std::fs::remove_dir_all(&dir).ok();
    }

    /// Unique temporary directory for a test.
    fn scratch_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("selah-engine-{name}-{}", std::process::id()))
    }
}
