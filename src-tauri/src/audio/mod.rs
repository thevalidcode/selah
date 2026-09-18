//! Audio subsystem.
//!
//! Layout:
//!   device enum -> capture (cpal) -> shared buffer -> speech worker (VAD/STT)
//!
//! No audio bytes ever cross to the frontend.

pub mod buffer;
pub mod capture;
pub mod device;
pub mod resampler;

use serde::Serialize;
use std::sync::{Arc, Mutex};

use crate::errors::AppError;

pub use buffer::AudioBuffer;
pub use capture::CaptureHandle;
pub use device::AudioDeviceInfo;
pub use resampler::LinearResampler;

/// How much raw audio the shared buffer will hold (≈16 seconds at 16 kHz).
pub const DEFAULT_BUFFER_CAPACITY: usize = 16_000 * 16;

/// Snapshot of the capture state for the UI.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioCaptureState {
    pub capturing: bool,
    pub device_id: Option<String>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
    pub buffered_samples: usize,
}

/// Owns microphone capture for the whole application. The shared buffer is
/// also consumed by the speech worker.
pub struct AudioManager {
    buffer: Arc<AudioBuffer>,
    capture: Mutex<Option<CaptureHandle>>,
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new(DEFAULT_BUFFER_CAPACITY)
    }
}

impl AudioManager {
    pub fn new(buffer_capacity: usize) -> Self {
        Self {
            buffer: Arc::new(AudioBuffer::new(buffer_capacity)),
            capture: Mutex::new(None),
        }
    }

    pub fn buffer(&self) -> Arc<AudioBuffer> {
        self.buffer.clone()
    }

    /// Starts capture on the given device id (falls back to default when
    /// `None`).
    pub fn start(&self, device_id: Option<&str>) -> Result<AudioCaptureState, AppError> {
        if self.is_capturing() {
            return Ok(self.state());
        }

        let device_id = match device_id {
            Some(id) if !id.is_empty() => id.to_string(),
            _ => device::default_input_device()?
                .map(|d| d.id)
                .ok_or_else(|| {
                    AppError::AudioDeviceNotFound("no default input device available".to_string())
                })?,
        };

        self.buffer.clear();
        let handle = capture::start_capture(&device_id, self.buffer.clone())?;
        *self
            .capture
            .lock()
            .map_err(|_| AppError::Internal("audio capture lock poisoned".to_string()))? =
            Some(handle);

        Ok(self.state())
    }

    pub fn stop(&self) {
        if let Ok(mut guard) = self.capture.lock() {
            if guard.take().is_some() {
                tracing::info!("audio capture stopped");
            }
        }
        self.buffer.clear();
    }

    pub fn is_capturing(&self) -> bool {
        self.capture.lock().map(|g| g.is_some()).unwrap_or(false)
    }

    pub fn state(&self) -> AudioCaptureState {
        let guard = self.capture.lock();
        match guard {
            Ok(g) => AudioCaptureState {
                capturing: g.is_some(),
                device_id: g.as_ref().map(|c| c.device_id.clone()),
                sample_rate: g.as_ref().map(|c| c.sample_rate),
                channels: g.as_ref().map(|c| c.channels),
                buffered_samples: self.buffer.sample_count(),
            },
            Err(_) => AudioCaptureState {
                capturing: false,
                device_id: None,
                sample_rate: None,
                channels: None,
                buffered_samples: 0,
            },
        }
    }
}

impl Drop for AudioManager {
    fn drop(&mut self) {
        self.stop();
    }
}
