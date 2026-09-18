//! Generic content detection model.
//!
//! Detected content is a typed, serializable result. In this bootstrap phase
//! only Scripture detection is wired; the variant set is designed to grow
//! (Lyrics, Media, Announcement, ...) without API rewrites.

use crate::scripture::reference::ScriptureReference;
use serde::{Deserialize, Serialize};

/// A piece of content detected from a transcript or text input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum DetectedContent {
    /// A structurally valid Scripture reference.
    Scripture { reference: ScriptureReference },
    /// Free text that is not (yet) a known content type.
    Text { text: String },
    /// Text we could not interpret at all.
    Unknown { text: String },
}

/// A detection with an optional confidence score (0.0 - 1.0).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectionResult {
    pub content: DetectedContent,
    pub confidence: Option<f32>,
}

/// Deterministic content detector.
///
/// Runs the Scripture parser over the input. Rule-based detection is fully
/// reproducible; the confidence value is a conservative constant because
/// there is no model involved.
pub struct ContentDetector;

impl Default for ContentDetector {
    fn default() -> Self {
        Self
    }
}

/// Confidence assigned to deterministic rule-based matches.
const RULE_BASED_CONFIDENCE: f32 = 0.95;

impl ContentDetector {
    /// Detects content in a transcript (or free text).
    pub fn detect(&self, transcript: &str) -> Vec<DetectionResult> {
        let transcript = transcript.trim();
        if transcript.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();
        for reference in crate::scripture::find_references(transcript) {
            results.push(DetectionResult {
                content: DetectedContent::Scripture { reference },
                confidence: Some(RULE_BASED_CONFIDENCE),
            });
        }

        // Only fall back to Text/Unknown when nothing more specific matched.
        if results.is_empty() {
            results.push(DetectionResult {
                content: DetectedContent::Text {
                    text: transcript.to_string(),
                },
                confidence: None,
            });
        }
        results
    }
}

impl DetectedContent {
    /// Short human label used by the UI.
    pub fn label(&self) -> String {
        match self {
            DetectedContent::Scripture { reference } => reference.display(),
            DetectedContent::Text { .. } => "Text".to_string(),
            DetectedContent::Unknown { .. } => "Unknown".to_string(),
        }
    }

    /// True when the detection is a Scripture reference.
    pub fn is_scripture(&self) -> bool {
        matches!(self, DetectedContent::Scripture { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_scripture_in_speech() {
        let detector = ContentDetector;
        let results =
            detector.detect("and today we are going to read John chapter three verse sixteen");
        assert_eq!(results.len(), 1);
        let DetectionResult {
            content,
            confidence,
        } = &results[0];
        assert!(content.is_scripture());
        assert!(confidence.unwrap() > 0.9);
    }

    #[test]
    fn empty_input_has_no_results() {
        let detector = ContentDetector;
        assert!(detector.detect("   ").is_empty());
    }

    #[test]
    fn plain_text_returns_text_result() {
        let detector = ContentDetector;
        let results = detector.detect("remember to bring the offering plates");
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0].content, DetectedContent::Text { .. }));
    }

    #[test]
    fn serde_shape() {
        let content = DetectedContent::Scripture {
            reference: ScriptureReference::new(43, 3).with_verse(16),
        };
        let json = serde_json::to_value(&content).unwrap();
        assert_eq!(json["type"], "scripture");
        assert_eq!(json["reference"]["bookId"], 43);
    }
}
