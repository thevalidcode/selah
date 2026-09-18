//! End-to-end check that the whisper.cpp recognizer actually turns audio into
//! text.
//!
//! This is the only test that proves the real speech path works end to end: it
//! loads a real `ggml-*.bin` model and transcribes a real 16 kHz mono clip.
//!
//! It is `#[ignore]`d because it needs two things that are not part of the
//! repository: a compiled `whisper` cargo feature and an operator-supplied
//! model file. Run it explicitly:
//!
//! ```bash
//! SELAH_WHISPER_MODEL=/path/to/ggml-base.en.bin \
//! SELAH_WHISPER_AUDIO=/path/to/clip.f32 \
//!   cargo test --features whisper --test whisper_transcription -- --ignored --nocapture
//! ```
//!
//! `SELAH_WHISPER_AUDIO` is raw little-endian `f32` mono PCM at 16 kHz — produce
//! it from any recording with:
//!
//! ```bash
//! ffmpeg -i clip.wav -ar 16000 -ac 1 -f f32le -c:a pcm_f32le clip.f32
//! ```

#![cfg(feature = "whisper")]

use selah_lib::speech::recognizer::SpeechRecognizer;
use selah_lib::speech::whisper::WhisperRecognizer;

/// Reads raw little-endian `f32` mono PCM.
fn read_pcm_f32(path: &str) -> Vec<f32> {
    let bytes = std::fs::read(path).expect("audio file should be readable");
    assert_eq!(
        bytes.len() % 4,
        0,
        "raw f32 audio must be a whole number of 4-byte samples"
    );
    bytes
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

#[test]
#[ignore = "needs the whisper feature, a ggml model and a 16 kHz f32 PCM clip"]
fn whisper_transcribes_speech_into_text() {
    let model = std::env::var("SELAH_WHISPER_MODEL")
        .expect("set SELAH_WHISPER_MODEL to a ggml-*.bin model file");
    let audio_path =
        std::env::var("SELAH_WHISPER_AUDIO").expect("set SELAH_WHISPER_AUDIO to a .f32 clip");

    let audio = read_pcm_f32(&audio_path);
    assert!(!audio.is_empty(), "audio clip must contain samples");

    let mut recognizer = WhisperRecognizer::default();
    assert!(!recognizer.is_loaded(), "starts with no model loaded");

    recognizer
        .load_model(&model)
        .expect("model should load from disk");
    assert!(
        recognizer.is_loaded(),
        "model should be loaded after load_model"
    );
    assert_eq!(recognizer.id(), "whisper");

    let transcript = recognizer
        .transcribe(&audio, 16_000)
        .expect("transcription should succeed");

    println!("transcript: {:?}", transcript.text);
    println!("segments: {}", transcript.segments.len());

    assert!(
        !transcript.text.trim().is_empty(),
        "whisper returned no text for audible speech"
    );

    // "the lord is my shepherd i shall not want" is the clip used in the manual
    // check; accept a couple of likely spellings of the distinctive word.
    let lowered = transcript.text.to_lowercase();
    assert!(
        lowered.contains("shepherd") || lowered.contains("shepard"),
        "expected the distinctive word in the transcript, got: {:?}",
        transcript.text
    );

    // Timestamps are converted from whisper's centiseconds to milliseconds.
    let first = &transcript.segments[0];
    assert!(
        first.end_ms >= first.start_ms,
        "segment end must not precede its start"
    );
}

#[test]
#[ignore = "needs the whisper feature and a ggml model"]
fn whisper_rejects_a_missing_model_file() {
    let mut recognizer = WhisperRecognizer::default();
    let err = recognizer
        .load_model("/definitely/not/a/real/model.bin")
        .expect_err("a missing model file must not load");
    // The operator gets a clear "model not found" rather than a crash.
    assert!(matches!(
        err,
        selah_lib::errors::AppError::SpeechModelNotFound(_)
    ));
}

#[test]
#[ignore = "needs the whisper feature and a ggml model"]
fn whisper_rejects_non_16khz_audio() {
    let model = std::env::var("SELAH_WHISPER_MODEL")
        .expect("set SELAH_WHISPER_MODEL to a ggml-*.bin model file");

    let mut recognizer = WhisperRecognizer::default();
    recognizer
        .load_model(&model)
        .expect("model should load from disk");

    let err = recognizer
        .transcribe(&[0.0; 16_000], 44_100)
        .expect_err("whisper only accepts 16 kHz audio");
    assert!(matches!(
        err,
        selah_lib::errors::AppError::InvalidConfiguration(_)
    ));
}
