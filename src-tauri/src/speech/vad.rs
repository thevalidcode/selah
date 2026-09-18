//! Voice activity detection abstraction.
//!
//! The pipeline never streams silence into the recognizer: only clipped
//! speech segments reach transcription. The default implementation is a
//! simple energy-based detector; a learned (Silero) implementation can be
//! swapped in through the same trait. The streaming helpers
//! (`frame_is_speech`, `frame_size_ms`, ...) let the live worker make
//! frame-by-frame decisions while `detect` supports batch analysis.

use crate::errors::AppError;

/// A contiguous speech region inside a stream of audio.
#[derive(Debug, Clone, PartialEq)]
pub struct SpeechSegment {
    pub start_ms: u64,
    pub end_ms: u64,
    /// The samples belonging to this segment (mono, at `sample_rate`).
    pub samples: Vec<f32>,
}

impl SpeechSegment {
    pub fn duration_ms(&self) -> u64 {
        self.end_ms.saturating_sub(self.start_ms)
    }
}

/// Detect speech regions in mono audio.
pub trait VoiceActivityDetector: Send {
    /// Finds speech segments in the provided buffer (batch use, tests).
    fn detect(&mut self, audio: &[f32], sample_rate: u32) -> Result<Vec<SpeechSegment>, AppError>;

    /// Streaming use: is this one analysis frame speech?
    fn frame_is_speech(&self, frame: &[f32]) -> bool;

    /// Streaming use: analysis frame length in milliseconds.
    fn frame_size_ms(&self) -> u32;

    /// Streaming use: minimum silence before a segment ends, in ms.
    fn min_silence_ms(&self) -> u64;

    /// Streaming use: minimum duration for a segment to be kept, in ms.
    fn min_speech_ms(&self) -> u64;

    /// Object-safe clone for moving a copy into worker threads.
    fn clone_box(&self) -> Box<dyn VoiceActivityDetector>;
}

impl Clone for Box<dyn VoiceActivityDetector> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Simple frame-energy VAD. Tuned conservative: low false-positive rate at
/// the cost of slightly later detection than a learned model.
#[derive(Debug, Clone)]
pub struct EnergyVad {
    pub threshold: f32,
    /// Analysis frame length in milliseconds.
    pub frame_ms: u32,
    /// A segment shorter than this is discarded as noise.
    pub min_speech_ms: u64,
    /// Silences longer than this split segments.
    pub min_silence_ms: u64,
}

impl Default for EnergyVad {
    fn default() -> Self {
        Self {
            threshold: 0.02,
            frame_ms: 20,
            min_speech_ms: 250,
            min_silence_ms: 400,
        }
    }
}

impl EnergyVad {
    /// RMS power of a mono frame.
    pub fn frame_energy(&self, frame: &[f32]) -> f32 {
        if frame.is_empty() {
            return 0.0;
        }
        let sum: f32 = frame.iter().map(|s| s * s).sum();
        (sum / frame.len() as f32).sqrt()
    }

    /// Whether a mono frame is loud enough to count as speech.
    pub fn is_speech_frame(&self, frame: &[f32]) -> bool {
        self.frame_energy(frame) >= self.threshold
    }

    /// Samples per analysis frame at the given rate.
    pub fn frame_size(&self, sample_rate: u32) -> usize {
        (sample_rate as u64 * u64::from(self.frame_ms) / 1000) as usize
    }
}

impl VoiceActivityDetector for EnergyVad {
    fn detect(&mut self, audio: &[f32], sample_rate: u32) -> Result<Vec<SpeechSegment>, AppError> {
        if sample_rate == 0 {
            return Err(AppError::VadFailed("sample rate must be non-zero".into()));
        }

        let frame_size = self.frame_size(sample_rate);
        let mut segments = Vec::new();
        let mut seg_start: Option<usize> = None; // sample index of first speech frame
        let mut last_speech_end: usize = 0; // sample index just past the last speech frame
        let mut silence_frames = 0usize;

        for (frame_idx, frame) in audio.chunks(frame_size).enumerate() {
            let frame_end = ((frame_idx + 1) * frame_size).min(audio.len());
            let is_speech = self.is_speech_frame(frame);
            if is_speech {
                if seg_start.is_none() {
                    seg_start = Some(frame_idx * frame_size);
                }
                last_speech_end = frame_end;
                silence_frames = 0;
            } else if let Some(start) = seg_start {
                silence_frames += 1;
                let silence_ms = silence_frames as u64 * u64::from(self.frame_ms);
                if silence_ms >= self.min_silence_ms {
                    let start_ms = start as u64 * 1000 / u64::from(sample_rate);
                    let end_ms = last_speech_end as u64 * 1000 / u64::from(sample_rate);
                    if end_ms.saturating_sub(start_ms) >= self.min_speech_ms {
                        segments.push(SpeechSegment {
                            start_ms,
                            end_ms,
                            samples: audio[start..last_speech_end].to_vec(),
                        });
                    }
                    seg_start = None;
                    silence_frames = 0;
                }
            }
        }

        // Trailing speech at end of buffer (trimmed to the last speech frame).
        if let Some(start) = seg_start {
            let start_ms = start as u64 * 1000 / u64::from(sample_rate);
            let end_ms = last_speech_end as u64 * 1000 / u64::from(sample_rate);
            if end_ms.saturating_sub(start_ms) >= self.min_speech_ms {
                segments.push(SpeechSegment {
                    start_ms,
                    end_ms,
                    samples: audio[start..last_speech_end].to_vec(),
                });
            }
        }

        Ok(segments)
    }

    fn frame_is_speech(&self, frame: &[f32]) -> bool {
        self.is_speech_frame(frame)
    }

    fn frame_size_ms(&self) -> u32 {
        self.frame_ms
    }

    fn min_silence_ms(&self) -> u64 {
        self.min_silence_ms
    }

    fn min_speech_ms(&self) -> u64 {
        self.min_speech_ms
    }

    fn clone_box(&self) -> Box<dyn VoiceActivityDetector> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn silence(rate: u32, ms: u64) -> Vec<f32> {
        vec![0.0; (rate as u64 * ms / 1000) as usize]
    }

    fn tone(rate: u32, ms: u64) -> Vec<f32> {
        vec![0.5; (rate as u64 * ms / 1000) as usize]
    }

    #[test]
    fn finds_isolated_speech() {
        let mut vad = EnergyVad::default();
        let mut audio = silence(16_000, 500);
        audio.extend(tone(16_000, 800));
        audio.extend(silence(16_000, 800));
        let segs = vad.detect(&audio, 16_000).unwrap();
        assert_eq!(segs.len(), 1);
        assert!(segs[0].duration_ms() >= 700);
    }

    #[test]
    fn ignores_brief_noise() {
        let mut vad = EnergyVad::default();
        let mut audio = silence(16_000, 200);
        audio.extend(tone(16_000, 50)); // shorter than min_speech_ms
        audio.extend(silence(16_000, 500));
        let segs = vad.detect(&audio, 16_000).unwrap();
        assert!(segs.is_empty());
    }

    #[test]
    fn splits_on_long_silence() {
        let mut vad = EnergyVad::default();
        let mut audio = tone(16_000, 500);
        audio.extend(silence(16_000, 600));
        audio.extend(tone(16_000, 500));
        let segs = vad.detect(&audio, 16_000).unwrap();
        assert_eq!(segs.len(), 2);
    }

    #[test]
    fn frame_classifier_matches_detect() {
        let vad = EnergyVad::default();
        assert!(vad.frame_is_speech(&[0.5; 320]));
        assert!(!vad.frame_is_speech(&[0.0; 320]));
    }
}
