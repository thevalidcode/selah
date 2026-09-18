//! Live speech pipeline manager.
//!
//! Owns the recognizer, the VAD and a worker thread that runs:
//!
//!   shared AudioBuffer -> resample to 16 kHz -> energy VAD -> speech segment
//!   -> recognizer.transcribe -> events (`speech://*`, `content://detected`)
//!
//! The worker only wakes on incoming samples; it never touches the UI thread.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use tauri::{AppHandle, Emitter};
use tracing::{info, warn};

use crate::audio::{AudioBuffer, LinearResampler};
use crate::errors::AppError;
use crate::events::{self, DetectionEvent, VadSegmentEvent};
use crate::models::content::ContentDetector;
use crate::models::settings::SpeechSettings;

use super::recognizer::SpeechRecognizer;
use super::vad::VoiceActivityDetector;

/// Hard cap for a single speech segment (30 s).
const MAX_SEGMENT_MS: u64 = 30_000;

/// In-memory state the UI can poll.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeechManagerState {
    pub listening: bool,
    pub recognizer_id: String,
    pub model_loaded: bool,
    pub vad_enabled: bool,
    pub segments_seen: u64,
    pub transcripts_generated: u64,
    pub last_error: Option<String>,
}

impl Default for SpeechManagerState {
    fn default() -> Self {
        Self {
            listening: false,
            recognizer_id: "mock".to_string(),
            model_loaded: false,
            vad_enabled: true,
            segments_seen: 0,
            transcripts_generated: 0,
            last_error: None,
        }
    }
}

#[derive(Debug, Clone)]
struct WorkerConfig {
    input_sample_rate: u32,
    target_sample_rate: u32,
    vad_enabled: bool,
}

/// Mutable state shared with the worker (audio, recognizer, VAD).
struct SpeechInner {
    buffer: Arc<AudioBuffer>,
    recognizer: Mutex<Box<dyn SpeechRecognizer>>,
    vad: Mutex<Box<dyn VoiceActivityDetector>>,
}

/// Manages the recognition worker thread.
pub struct SpeechManager {
    inner: Arc<SpeechInner>,
    running: Arc<AtomicBool>,
    worker: Mutex<Option<JoinHandle<()>>>,
    config: Arc<Mutex<WorkerConfig>>,
    state: Arc<Mutex<SpeechManagerState>>,
    app: AppHandle,
}

/// A speech segment being collected by the worker.
struct LiveSegment {
    start_ms: u64,
    samples: Vec<f32>,
    silence_ms: u64,
}

impl LiveSegment {
    fn duration_ms(&self, sample_rate: u32) -> u64 {
        (self.samples.len() as u64 * 1000) / sample_rate as u64
    }
}

impl SpeechManager {
    pub fn new(
        app: AppHandle,
        buffer: Arc<AudioBuffer>,
        recognizer: Box<dyn SpeechRecognizer>,
        vad: Box<dyn VoiceActivityDetector>,
        settings: &SpeechSettings,
    ) -> Self {
        let state = Arc::new(Mutex::new(SpeechManagerState {
            listening: false,
            recognizer_id: recognizer.id().to_string(),
            model_loaded: recognizer.is_loaded(),
            vad_enabled: settings.vad_enabled,
            ..SpeechManagerState::default()
        }));

        Self {
            inner: Arc::new(SpeechInner {
                buffer,
                recognizer: Mutex::new(recognizer),
                vad: Mutex::new(vad),
            }),
            running: Arc::new(AtomicBool::new(false)),
            worker: Mutex::new(None),
            config: Arc::new(Mutex::new(WorkerConfig {
                // Updated on every `start` from the capture state.
                input_sample_rate: settings.speech_sample_rate,
                target_sample_rate: settings.speech_sample_rate,
                vad_enabled: settings.vad_enabled,
            })),
            state,
            app,
        }
    }

    /// Rebuilds the recognizer from (possibly changed) settings.
    ///
    /// If the pipeline is currently listening it is restarted with the new
    /// configuration.
    pub fn reconfigure(&self, settings: &SpeechSettings) -> Result<(), AppError> {
        let was_listening = self.is_listening();
        if was_listening {
            self.stop();
        }

        let mut recognizer = super::recognizer::recognizer_for(settings.recognizer);
        if !settings.model_path.as_deref().unwrap_or("").is_empty() {
            recognizer.load_model(
                settings
                    .model_path
                    .as_deref()
                    .ok_or_else(|| AppError::InvalidConfiguration("missing model path".into()))?,
            )?;
        }

        *self
            .inner
            .recognizer
            .lock()
            .map_err(|_| AppError::Internal("recognizer lock poisoned".to_string()))? = recognizer;

        {
            let mut cfg = self
                .config
                .lock()
                .map_err(|_| AppError::Internal("config lock poisoned".to_string()))?;
            cfg.target_sample_rate = settings.speech_sample_rate;
            cfg.vad_enabled = settings.vad_enabled;
        }

        {
            let mut state = self
                .state
                .lock()
                .map_err(|_| AppError::Internal("state lock poisoned".to_string()))?;
            state.vad_enabled = settings.vad_enabled;
            state.recognizer_id = self
                .inner
                .recognizer
                .lock()
                .map(|r| r.id().to_string())
                .unwrap_or_else(|_| "unknown".to_string());
            state.model_loaded = self
                .inner
                .recognizer
                .lock()
                .map(|r| r.is_loaded())
                .unwrap_or(false);
        }

        if was_listening {
            self.start(None)?;
        }
        Ok(())
    }

    /// Starts the worker loop. `input_sample_rate` is the device rate feeding
    /// the shared buffer.
    pub fn start(&self, input_sample_rate: Option<u32>) -> Result<(), AppError> {
        if self.is_listening() {
            return Ok(());
        }
        self.inner.buffer.clear();

        {
            let mut cfg = self
                .config
                .lock()
                .map_err(|_| AppError::Internal("config lock poisoned".to_string()))?;
            // Unknown devices default to "already at target rate": the worker
            // is only ever fed mono capture resampled toward target.
            if let Some(rate) = input_sample_rate.filter(|r| *r > 0) {
                cfg.input_sample_rate = rate;
            }
        }

        self.running.store(true, Ordering::SeqCst);

        let running = self.running.clone();
        let inner = self.inner.clone();
        let config = self.config.clone();
        let state = self.state.clone();
        let app = self.app.clone();

        let handle = thread::Builder::new()
            .name("selah-speech-worker".to_string())
            .spawn(move || run_worker(running, inner, config, state, app))
            .map_err(|e| AppError::Internal(format!("failed to spawn speech worker: {e}")))?;

        *self
            .worker
            .lock()
            .map_err(|_| AppError::Internal("worker lock poisoned".to_string()))? = Some(handle);

        {
            let mut state = self
                .state
                .lock()
                .map_err(|_| AppError::Internal("state lock poisoned".to_string()))?;
            state.listening = true;
            state.last_error = None;
        }

        info!("speech recognition started");
        let _ = self.app.emit(events::SPEECH_STARTED, ());
        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        let handle = self.worker.lock().map(|mut g| g.take()).unwrap_or(None);
        if let Some(handle) = handle {
            let _ = handle.join();
        }
        if let Ok(mut state) = self.state.lock() {
            state.listening = false;
        }
        info!("speech recognition stopped");
        let _ = self.app.emit(events::SPEECH_STOPPED, ());
    }

    pub fn is_listening(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn state(&self) -> SpeechManagerState {
        self.state.lock().map(|g| g.clone()).unwrap_or_default()
    }
}

impl Drop for SpeechManager {
    fn drop(&mut self) {
        if self.is_listening() {
            self.stop();
        }
    }
}
/// Worker loop: drain buffer → resample → VAD-gate → transcribe → events.
fn run_worker(
    running: Arc<AtomicBool>,
    inner: Arc<SpeechInner>,
    config: Arc<Mutex<WorkerConfig>>,
    state: Arc<Mutex<SpeechManagerState>>,
    app: AppHandle,
) {
    let cfg = config.lock().map(|g| g.clone()).unwrap_or(WorkerConfig {
        input_sample_rate: 16_000,
        target_sample_rate: 16_000,
        vad_enabled: true,
    });

    let mut resampler = match LinearResampler::new(cfg.input_sample_rate, cfg.target_sample_rate) {
        Ok(r) => r,
        Err(e) => {
            if let Ok(mut s) = state.lock() {
                s.last_error = Some(e.to_string());
            }
            warn!(error = %e, "speech worker failed to configure resampler");
            return;
        }
    };

    let (frame_ms, min_silence_ms, min_speech_ms, frame_size) = {
        let vad = inner.vad.lock().unwrap();
        let frame_ms = vad.frame_size_ms();
        let frame_size = (cfg.target_sample_rate as usize * frame_ms as usize) / 1000;
        (
            frame_ms,
            vad.min_silence_ms(),
            vad.min_speech_ms(),
            frame_size.max(1),
        )
    };

    let mut segment: Option<LiveSegment> = None;
    let mut elapsed_ms: u64 = 0;
    let detector = ContentDetector;

    while running.load(Ordering::SeqCst) {
        let chunk = inner.buffer.drain();
        if chunk.is_empty() {
            thread::sleep(Duration::from_millis(30));
            continue;
        }

        let samples = resampler.process(&chunk);
        let vad = inner.vad.lock().unwrap();

        for frame in samples.chunks(frame_size) {
            let is_speech = vad.frame_is_speech(frame);
            match (&mut segment, is_speech) {
                (None, true) => {
                    segment = Some(LiveSegment {
                        start_ms: elapsed_ms,
                        samples: frame.to_vec(),
                        silence_ms: 0,
                    });
                }
                (Some(seg), true) => {
                    seg.samples.extend_from_slice(frame);
                    seg.silence_ms = 0;
                }
                (Some(seg), false) => {
                    seg.silence_ms += u64::from(frame_ms);
                    let duration = seg.duration_ms(cfg.target_sample_rate);
                    if seg.silence_ms >= min_silence_ms || duration >= MAX_SEGMENT_MS {
                        finalize_segment(
                            segment.take().unwrap(),
                            &inner,
                            &state,
                            &app,
                            cfg.target_sample_rate,
                            min_speech_ms,
                            &detector,
                        );
                    }
                }
                (None, false) => {}
            }
            elapsed_ms = elapsed_ms.saturating_add(u64::from(frame_ms));
        }
        drop(vad);
    }

    // Flush any in-progress segment when the worker is asked to stop.
    if let Some(seg) = segment {
        finalize_segment(
            seg,
            &inner,
            &state,
            &app,
            cfg.target_sample_rate,
            min_speech_ms,
            &detector,
        );
    }

    info!("speech worker finished");
}

/// Transcribes a finished segment and emits the resulting events.
#[allow(clippy::too_many_arguments)]
fn finalize_segment(
    segment: LiveSegment,
    inner: &SpeechInner,
    state: &Arc<Mutex<SpeechManagerState>>,
    app: &AppHandle,
    sample_rate: u32,
    min_speech_ms: u64,
    detector: &ContentDetector,
) {
    let duration_ms = segment.duration_ms(sample_rate);
    if duration_ms < min_speech_ms {
        return;
    }

    let end_ms = segment.start_ms.saturating_add(duration_ms);
    let _ = app.emit(
        events::SPEECH_VAD_SEGMENT,
        VadSegmentEvent {
            start_ms: segment.start_ms,
            end_ms,
            duration_ms,
        },
    );

    {
        if let Ok(mut s) = state.lock() {
            s.segments_seen += 1;
        }
    }

    let transcript = {
        let mut recognizer = match inner.recognizer.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        match recognizer.transcribe(&segment.samples, sample_rate) {
            Ok(t) => t,
            Err(e) => {
                if let Ok(mut s) = state.lock() {
                    s.last_error = Some(e.to_string());
                }
                warn!(error = %e, "transcription failed");
                return;
            }
        }
    };

    {
        if let Ok(mut s) = state.lock() {
            s.transcripts_generated += 1;
            s.last_error = None;
        }
        let _ = app.emit(events::SPEECH_TRANSCRIPT, &transcript);
    }

    // Rule-based content detection on the transcript.
    if !transcript.text.trim().is_empty() {
        let results = detector.detect(&transcript.text);
        if !results.is_empty() {
            let _ = app.emit(
                events::CONTENT_DETECTED,
                DetectionEvent {
                    source: format!("{:?}", transcript.source).to_lowercase(),
                    results,
                },
            );
        }
    }
}
