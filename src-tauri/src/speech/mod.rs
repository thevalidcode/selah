//! Speech subsystem.
//!
//! Layout:
//!   traits (recognizer, VAD) → implementations (moonshine behind a feature,
//!   mock always available) → live pipeline manager.

pub mod manager;
pub mod mock;
pub mod recognizer;
pub mod transcript;
pub mod vad;

#[cfg(feature = "moonshine")]
pub mod moonshine;

pub use manager::{SpeechManager, SpeechManagerState};
pub use recognizer::SpeechRecognizer;
pub use transcript::{Transcript, TranscriptSegment, TranscriptSource};
pub use vad::{EnergyVad, SpeechSegment, VoiceActivityDetector};
