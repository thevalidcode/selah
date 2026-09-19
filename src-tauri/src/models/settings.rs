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
    /// Moonshine (ONNX) through transcribe-rs. Requires the `moonshine`
    /// cargo feature (on by default) and a model directory at runtime.
    ///
    /// `alias` keeps settings written by an earlier build (which named the
    /// whisper.cpp engine) loading cleanly instead of failing to deserialize
    /// and blocking application start-up.
    #[serde(alias = "whisper")]
    Moonshine,
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
    /// The expected/speech sample rate after resampling (Moonshine wants 16k).
    pub speech_sample_rate: u32,
}

impl Default for SpeechSettings {
    fn default() -> Self {
        Self {
            // The mock recognizer needs no model, so it is the safe default.
            recognizer: RecognizerKind::Mock,
            model_path: None,
            language: None,
            threads: std::thread::available_parallelism()
                .map(|n| n.get() as u32)
                .unwrap_or(4),
            vad_enabled: true,
            speech_sample_rate: 16_000,
        }
    }
}

/// Font family the projector draws with when a setting does not name one.
pub fn default_font_family() -> String {
    "Creato Display".to_string()
}

/// Background colour the projector falls back to when none is set.
pub fn default_background() -> String {
    "#000000".to_string()
}

/// Text size (in CSS pixels) the projector falls back to when none is set.
pub fn default_font_size() -> u32 {
    64
}

/// Which edge of the screen the branding overlay is drawn against.
///
/// Stored in settings, so the set of values is deliberately small and stable:
/// an unknown value from a hand-edited document falls back to the default
/// rather than failing to load.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BrandingPosition {
    Top,
    #[default]
    Bottom,
    Left,
    Right,
}

impl BrandingPosition {
    /// The CSS class-safe name used by the presentation window.
    pub fn as_str(self) -> &'static str {
        match self {
            BrandingPosition::Top => "top",
            BrandingPosition::Bottom => "bottom",
            BrandingPosition::Left => "left",
            BrandingPosition::Right => "right",
        }
    }
}

/// Longest branding line Selah will draw. Anything longer is trimmed in
/// [`BrandingSettings::sanitized`] so a pasted paragraph cannot be mistaken for
/// projected content.
pub const MAX_BRANDING_TEXT: usize = 120;

/// The operator's own branding: a line of text and/or a logo image, drawn on
/// every projected item.
///
/// Both are optional: an empty `text` and no `logo` simply means "no branding",
/// so there is no separate on/off flag to fall out of sync with the content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BrandingSettings {
    /// Which edge the overlay hugs.
    pub position: BrandingPosition,
    /// Church name, service title, slogan…
    pub text: Option<String>,
    /// Absolute path to a logo image the operator chose. The projector window
    /// receives it as an `asset:` URL, exactly like media files.
    pub logo: Option<String>,
    /// How much smaller than the projected text the overlay is, as a
    /// percentage. Kept small so branding never competes with Scripture.
    #[serde(default = "default_branding_size_percent")]
    pub size_percent: u32,
}

impl BrandingSettings {
    /// Largest overlay size allowed, as a percentage of the projected text.
    pub const MAX_SIZE_PERCENT: u32 = 60;
    /// Smallest overlay size allowed.
    pub const MIN_SIZE_PERCENT: u32 = 10;

    /// Trims, de-duplicates whitespace and repairs values that would render
    /// badly.
    pub fn sanitized(&self) -> (Self, bool) {
        let mut fixed = self.clone();
        let mut changed = false;

        let text = self
            .text
            .as_deref()
            .map(|t| t.split_whitespace().collect::<Vec<_>>().join(" "))
            .filter(|t| !t.is_empty())
            .map(|t| {
                if t.chars().count() > MAX_BRANDING_TEXT {
                    t.chars().take(MAX_BRANDING_TEXT).collect::<String>()
                } else {
                    t
                }
            });
        if text != fixed.text {
            fixed.text = text;
            changed = true;
        }

        // Only an absolute path can be opened by the projector window; a
        // relative one is a typo rather than something to guess at.
        let logo = self
            .logo
            .as_deref()
            .map(str::trim)
            .filter(|l| std::path::Path::new(l).is_absolute())
            .map(str::to_string);
        if logo != fixed.logo {
            fixed.logo = logo;
            changed = true;
        }

        if !(Self::MIN_SIZE_PERCENT..=Self::MAX_SIZE_PERCENT).contains(&self.size_percent) {
            fixed.size_percent = Self::default().size_percent;
            changed = true;
        }

        (fixed, changed)
    }

    /// Whether there is anything to draw at all.
    pub fn is_empty(&self) -> bool {
        self.text.is_none() && self.logo.is_none()
    }
}

/// Default overlay size, as a percentage of the projected text.
pub fn default_branding_size_percent() -> u32 {
    30
}

impl Default for BrandingSettings {
    fn default() -> Self {
        Self {
            position: BrandingPosition::default(),
            text: None,
            logo: None,
            size_percent: default_branding_size_percent(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresentationSettings {
    pub display_index: Option<usize>,
    pub fullscreen: bool,
    #[serde(default = "default_background")]
    pub background: String,
    #[serde(default = "default_font_size")]
    pub font_size: u32,
    /// Family used for projected text. Matched against the frontend font
    /// catalog, so a stale value simply falls back to the interface default.
    #[serde(default = "default_font_family")]
    pub font_family: String,
    pub follow_live: bool,
    /// The operator's own logo and line of text, drawn on every projected item.
    #[serde(default)]
    pub branding: BrandingSettings,
}

impl Default for PresentationSettings {
    fn default() -> Self {
        Self {
            display_index: None,
            fullscreen: true,
            background: default_background(),
            font_size: default_font_size(),
            font_family: default_font_family(),
            follow_live: true,
            branding: BrandingSettings::default(),
        }
    }
}

/// Where the media library reads files from.
///
/// Nothing is hardcoded: the operator picks a folder at runtime and Selah
/// remembers it, so the same installation works on any machine.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaSettings {
    /// Folder chosen by the operator. `None` until they pick one.
    pub directory: Option<String>,
    /// Whether sub-folders are read as well.
    pub recursive: bool,
}

impl PresentationSettings {
    /// Repairs values that would break the projector window.
    ///
    /// Settings are a free-form JSON document, so a hand-edited or partially
    /// written file can contain a malformed background colour (for example
    /// `#06666`) or a font size small enough to be unreadable from the back of
    /// a room. Rather than silently render a black-on-black or hair-thin
    /// service, clamp those two fields back into a safe range and report
    /// whether anything was changed.
    pub fn sanitized(&self) -> (Self, bool) {
        let mut fixed = self.clone();
        let mut changed = false;

        if !is_valid_hex_color(&self.background) {
            fixed.background = Self::default().background;
            changed = true;
        }
        if !(MIN_FONT_SIZE..=MAX_FONT_SIZE).contains(&self.font_size) {
            fixed.font_size = Self::default().font_size;
            changed = true;
        }
        if self.font_family.trim().is_empty() {
            fixed.font_family = Self::default().font_family;
            changed = true;
        }

        let (branding, branding_changed) = self.branding.sanitized();
        if branding_changed {
            fixed.branding = branding;
            changed = true;
        }

        (fixed, changed)
    }
}

/// Smallest projector font size considered readable (theatre/back-of-room).
const MIN_FONT_SIZE: u32 = 16;
/// Largest projector font size; above this a single word fills the screen.
const MAX_FONT_SIZE: u32 = 200;

/// Whether `value` is a `#RRGGBB` colour literal.
fn is_valid_hex_color(value: &str) -> bool {
    let Some(digits) = value.strip_prefix('#') else {
        return false;
    };
    digits.len() == 6 && digits.chars().all(|c| c.is_ascii_hexdigit())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppSettings {
    pub general: GeneralSettings,
    pub audio: AudioSettings,
    pub speech: SpeechSettings,
    pub presentation: PresentationSettings,
    /// Where the media library reads files from (chosen at runtime).
    #[serde(default)]
    pub media: MediaSettings,
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
            presentation: PresentationSettings::default(),
            media: MediaSettings::default(),
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

    #[test]
    fn legacy_whisper_recognizer_loads_as_moonshine() {
        // Settings written by the previous build named the whisper.cpp engine.
        // They must load rather than abort start-up.
        let json = r#"{
            "speech": { "recognizer": "whisper", "modelPath": null, "language": null,
                        "threads": 4, "vadEnabled": true, "speechSampleRate": 16000 }
        }"#;
        let s: AppSettings = serde_json::from_str(json).unwrap();
        assert_eq!(s.speech.recognizer, RecognizerKind::Moonshine);
    }

    #[test]
    fn sanitize_repairs_malformed_projector_settings() {
        // Reproduces the truncated values seen in a real settings document:
        // an invalid 5-digit colour and a font size far too small to read.
        let broken = PresentationSettings {
            display_index: None,
            fullscreen: false,
            background: "#06666".to_string(),
            font_size: 10,
            font_family: "   ".to_string(),
            follow_live: true,
            branding: BrandingSettings::default(),
        };
        let (fixed, changed) = broken.sanitized();
        assert!(changed);
        assert_eq!(fixed.background, "#000000");
        assert_eq!(fixed.font_size, 64);
        assert_eq!(fixed.font_family, "Creato Display");
        // Fields that were already valid are left alone.
        assert!(!fixed.fullscreen);
        assert!(fixed.follow_live);
    }

    #[test]
    fn settings_written_before_fonts_and_media_existed_still_load() {
        // A settings document from an earlier build names neither a font nor a
        // media folder; both must fall back to defaults rather than fail.
        let json = r##"{
            "general": { "appName": "Selah", "defaultTranslationId": null, "theme": "dark" },
            "audio": { "inputDeviceId": null, "sampleRate": 0 },
            "speech": { "recognizer": "mock", "modelPath": null, "language": null,
                        "threads": 4, "vadEnabled": true, "speechSampleRate": 16000 },
            "presentation": { "displayIndex": null, "fullscreen": true,
                              "background": "#101010", "fontSize": 72, "followLive": true }
        }"##;
        let s: AppSettings = serde_json::from_str(json).unwrap();
        assert_eq!(s.presentation.font_family, "Creato Display");
        assert_eq!(s.presentation.font_size, 72);
        assert_eq!(s.media.directory, None);
        assert!(!s.media.recursive);
    }

    #[test]
    fn sanitize_leaves_valid_settings_untouched() {
        let settings = PresentationSettings::default();
        let (fixed, changed) = settings.sanitized();
        assert!(!changed);
        assert_eq!(fixed, settings);
    }

    #[test]
    fn branding_without_text_or_logo_draws_nothing() {
        let branding = BrandingSettings::default();
        assert!(branding.is_empty());
        assert_eq!(branding.position, BrandingPosition::Bottom);
        assert_eq!(branding.size_percent, default_branding_size_percent());
    }

    #[test]
    fn branding_text_is_trimmed_and_capped() {
        // A pasted paragraph must not turn into projected content: the overlay
        // is one line, so it is collapsed and cut to length.
        let crowded = BrandingSettings {
            text: Some(format!("  Grace   Chapel\n{}", "x".repeat(400))),
            ..BrandingSettings::default()
        };
        let (fixed, changed) = crowded.sanitized();
        assert!(changed);
        let text = fixed.text.expect("text kept");
        assert!(text.starts_with("Grace Chapel x"));
        assert_eq!(text.chars().count(), MAX_BRANDING_TEXT);
    }

    #[test]
    fn a_relative_logo_path_is_dropped_but_a_real_one_is_kept() {
        let wrong = BrandingSettings {
            logo: Some("logo.png".to_string()),
            ..BrandingSettings::default()
        };
        let (fixed, changed) = wrong.sanitized();
        assert!(changed);
        assert_eq!(fixed.logo, None);

        let right = BrandingSettings {
            logo: Some("/Users/someone/logo.png".to_string()),
            size_percent: 40,
            ..BrandingSettings::default()
        };
        let (fixed, changed) = right.sanitized();
        assert!(!changed);
        assert_eq!(fixed.logo.as_deref(), Some("/Users/someone/logo.png"));
    }

    #[test]
    fn an_out_of_range_branding_size_falls_back() {
        let silly = BrandingSettings {
            size_percent: 5000,
            ..BrandingSettings::default()
        };
        let (fixed, changed) = silly.sanitized();
        assert!(changed);
        assert_eq!(fixed.size_percent, default_branding_size_percent());
    }

    #[test]
    fn settings_without_branding_still_load() {
        // A document saved before branding existed must load with no branding
        // rather than failing and blocking start-up.
        let json = r##"{
            "general": { "appName": "Selah", "defaultTranslationId": null, "theme": "dark" },
            "audio": { "inputDeviceId": null, "sampleRate": 0 },
            "speech": { "recognizer": "mock", "modelPath": null, "language": null,
                        "threads": 4, "vadEnabled": true, "speechSampleRate": 16000 },
            "presentation": { "displayIndex": null, "fullscreen": true,
                              "background": "#101010", "fontSize": 72,
                              "fontFamily": "Inter", "followLive": true }
        }"##;
        let s: AppSettings = serde_json::from_str(json).unwrap();
        assert!(s.presentation.branding.is_empty());
        assert_eq!(s.presentation.branding.position, BrandingPosition::Bottom);
    }
}
