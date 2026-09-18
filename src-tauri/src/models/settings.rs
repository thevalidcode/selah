//! Persisted application settings.
//!
//! Stored as a single JSON document in the `settings` table so new fields do
//! not require schema migrations. Defaults are conservative.

use serde::{Deserialize, Serialize};

/// Which speech recognition implementation is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecognizerKind {
    /// Development recognizer. Returns empty transcripts — never pretends to
    /// transcribe audio.
    Mock,
    /// whisper.cpp via whisper-rs (requires the `whisper` cargo feature).
    Whisper,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneralSettings {
    pub app_name: String,
    pub default_translation_id: Option<String>,
    pub theme: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioSettings {
    pub input_device_id: Option<String>,
    /// 0 = device default sample rate.
    pub sample_rate: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeechSettings {
    pub recognizer: RecognizerKind,
    pub model_path: Option<String>,
    pub language: Option<String>,
    pub threads: u32,
    pub vad_enabled: bool,
    /// The expected/speech sample rate after resampling (whisper wants 16k).
    pub speech_sample_rate: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresentationSettings {
    pub display_index: Option<usize>,
    pub fullscreen: bool,
    pub background: String,
    pub font_size: u32,
    pub follow_live: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppSettings {
    pub general: GeneralSettings,
    pub audio: AudioSettings,
    pub speech: SpeechSettings,
    pub presentation: PresentationSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            general: GeneralSettings {
                app_name: "Selah".to_string(),
                default_translation_id: None,
                theme: "dark".to_string(),
            },
            audio: AudioSettings {
                input_device_id: None,
                sample_rate: 0,
            },
            speech: SpeechSettings {
                recognizer: RecognizerKind::Mock,
                model_path: None,
                language: None,
                threads: std::thread::available_parallelism()
                    .map(|n| n.get() as u32)
                    .unwrap_or(4),
                vad_enabled: true,
                speech_sample_rate: 16_000,
            },
            presentation: PresentationSettings {
                display_index: None,
                fullscreen: true,
                background: "#000000".to_string(),
                font_size: 64,
                follow_live: true,
            },
        }
    }
}

/// Storage key for the settings JSON document.
pub const SETTINGS_KEY: &str = "app_settings";
/// Storage key marking that the first-run setup has been completed.
pub const SETUP_COMPLETE_KEY: &str = "setup_complete";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_are_sane() {
        let s = AppSettings::default();
        assert_eq!(s.speech.speech_sample_rate, 16_000);
        assert_eq!(s.speech.recognizer, RecognizerKind::Mock);
        assert!(s.speech.vad_enabled);
        assert!(s.speech.threads >= 1);
    }

    #[test]
    fn serde_roundtrip() {
        let s = AppSettings::default();
        let json = serde_json::to_string(&s).unwrap();
        let back: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }
}
