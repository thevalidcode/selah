//! Transcript types shared by all recognizers.
//!
//! The application never depends on Whisper-specific types outside the
//! `speech::whisper` module.

use serde::Serialize;

/// Which implementation produced a transcript.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TranscriptSource {
    /// Development recognizer. It never fabricates text.
    Mock,
    /// whisper.cpp via whisper-rs.
    Whisper,
}

/// The result of transcribing one speech segment.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Transcript {
    pub text: String,
    pub segments: Vec<TranscriptSegment>,
    pub source: TranscriptSource,
    /// Number of input audio samples the recognizer consumed.
    pub sample_count: usize,
}

/// A piece of a transcript within the full recognized audio.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptSegment {
    pub text: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub confidence: Option<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transcript_serializes_camel_case() {
        let t = Transcript {
            text: "hello".into(),
            segments: vec![TranscriptSegment {
                text: "hello".into(),
                start_ms: 0,
                end_ms: 500,
                confidence: Some(0.9),
            }],
            source: TranscriptSource::Mock,
            sample_count: 8000,
        };
        let v = serde_json::to_value(&t).unwrap();
        assert_eq!(v["source"], "mock");
        assert_eq!(v["segments"][0]["startMs"], 0);
        assert!(v["sampleCount"].is_number());
    }
}
