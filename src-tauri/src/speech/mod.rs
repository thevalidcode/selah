//! Speech subsystem.
//!
//! Layout:
//!   traits (recognizer, VAD) → implementations (whisper behind a feature,
//!   mock always available) → live pipeline manager.

pub mod manager;
pub mod mock;
pub mod recognizer;
pub mod transcript;
pub mod vad;

#[cfg(feature = "whisper")]
pub mod whisper;

pub use manager::{SpeechManager, SpeechManagerState};
pub use recognizer::SpeechRecognizer;
pub use transcript::{Transcript, TranscriptSegment, TranscriptSource};
pub use vad::{EnergyVad, SpeechSegment, VoiceActivityDetector};
