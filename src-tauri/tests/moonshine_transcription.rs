//! End-to-end Moonshine transcription.
//!
//! Ignored by default: it needs a model folder and a 16 kHz mono WAV clip,
//! neither of which is committed. Run it with:
//!
//! ```bash
//! SELAH_MOONSHINE_MODEL="$HOME/Library/Application Support/app.selah.desktop/models/moonshine" \
//! SELAH_MOONSHINE_WAV=/tmp/speech.wav \
//! cargo test --features moonshine --test moonshine_transcription -- --ignored --nocapture
//! ```

use std::path::Path;

use selah_lib::errors::AppError;
use selah_lib::speech::moonshine::MoonshineRecognizer;
use selah_lib::speech::{SpeechRecognizer, TranscriptSource};

/// A folder that exists but holds no graphs must fail with a message naming
/// the files it wanted, so the operator knows what to copy in.
#[test]
fn empty_model_folder_names_the_missing_files() {
    let dir = std::env::temp_dir().join(format!("selah-empty-model-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();

    let mut recognizer = MoonshineRecognizer::default();
    let err = recognizer
        .load_model(dir.to_str().unwrap())
        .expect_err("an empty folder is not a model");

    let message = err.to_string();
    assert!(message.contains("encoder_model"), "got: {message}");

    std::fs::remove_dir_all(&dir).ok();
}

/// A path that is a file, not a folder, must be rejected clearly.
#[test]
fn file_path_is_rejected() {
    let path = std::env::temp_dir().join(format!("selah-not-a-dir-{}", std::process::id()));
    std::fs::write(&path, b"x").unwrap();

    let mut recognizer = MoonshineRecognizer::default();
    let err = recognizer
        .load_model(path.to_str().unwrap())
        .expect_err("a file is not a model folder");
    assert!(matches!(err, AppError::SpeechModelNotFound(_)));

    std::fs::remove_file(&path).ok();
}

/// Loads a real model and transcribes real speech.
#[test]
#[ignore = "needs SELAH_MOONSHINE_MODEL and SELAH_MOONSHINE_WAV"]
fn transcribes_known_speech() {
    let model_dir = required_env("SELAH_MOONSHINE_MODEL");
    let wav = required_env("SELAH_MOONSHINE_WAV");

    let samples = read_wav_16k_mono(Path::new(&wav));
    println!(
        "loaded {} samples ({:.2}s)",
        samples.len(),
        samples.len() as f32 / 16_000.0
    );

    let mut recognizer = MoonshineRecognizer::default();
    recognizer
        .load_model(&model_dir)
        .expect("model should load");
    assert!(recognizer.is_loaded());

    let transcript = recognizer
        .transcribe(&samples, 16_000)
        .expect("transcription should succeed");

    println!("transcript: {:?}", transcript.text);
    println!("segments: {}", transcript.segments.len());

    assert_eq!(transcript.source, TranscriptSource::Moonshine);
    assert!(!transcript.text.trim().is_empty(), "no text was produced");

    let lowered = transcript.text.to_lowercase();
    assert!(
        lowered.contains("shepherd") || lowered.contains("shepard"),
        "expected the distinctive word in the transcript, got: {:?}",
        transcript.text
    );
}

/// Reads an environment variable, failing the test when it is unset.
fn required_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{name} must be set for this test"))
}

/// Decodes a 16-bit mono WAV file into normalised `f32` samples.
///
/// Deliberately dependency-free so the test doubles as a check that nothing
/// about transcription depends on a particular audio library.
fn read_wav_16k_mono(path: &Path) -> Vec<f32> {
    let bytes = std::fs::read(path).expect("audio file should be readable");
    assert!(bytes.len() > 44, "audio file is too small to be a WAV");

    let find = |tag: &[u8; 4]| {
        bytes
            .windows(4)
            .position(|window| window == tag)
            .unwrap_or_else(|| panic!("WAV is missing its {:?} chunk", tag))
    };

    let fmt = find(b"fmt ");
    let channels = u16::from_le_bytes([bytes[fmt + 10], bytes[fmt + 11]]);
    let sample_rate = u32::from_le_bytes([
        bytes[fmt + 12],
        bytes[fmt + 13],
        bytes[fmt + 14],
        bytes[fmt + 15],
    ]);
    let bits = u16::from_le_bytes([bytes[fmt + 22], bytes[fmt + 23]]);
    assert_eq!(channels, 1, "clip must be mono");
    assert_eq!(sample_rate, 16_000, "clip must be 16 kHz");
    assert_eq!(bits, 16, "clip must be 16-bit PCM");

    let data = find(b"data");
    let pcm = &bytes[data + 8..];
    pcm.chunks_exact(2)
        .map(|pair| i16::from_le_bytes([pair[0], pair[1]]) as f32 / 32_768.0)
        .collect()
}
