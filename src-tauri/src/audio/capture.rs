//! Microphone capture via cpal.
//!
//! The cpal data callback only pushes floats into a shared [`AudioBuffer`]; it
//! never does parsing, VAD, or transcription. Sample format conversion to f32
//! is requested from cpal; channel downmixing (stereo → mono) happens here.

use std::sync::Arc;

use cpal::traits::{DeviceTrait, StreamTrait};

use crate::errors::AppError;

use super::AudioBuffer;

/// An open input stream together with the configuration it was opened with.
pub struct CaptureHandle {
    #[allow(dead_code)] // keeping the stream alive is the point
    stream: cpal::Stream,
    pub device_id: String,
    pub sample_rate: u32,
    pub channels: u16,
}

/// Opens a microphone stream and starts pushing samples into `buffer`.
pub fn start_capture(device_id: &str, buffer: Arc<AudioBuffer>) -> Result<CaptureHandle, AppError> {
    let device = super::device::find_device(device_id)?
        .ok_or_else(|| AppError::AudioDeviceNotFound(device_id.to_string()))?;

    let config: cpal::StreamConfig = device
        .default_input_config()
        .map_err(|e| AppError::AudioQueryFailed(e.to_string()))?
        .into();

    let channels = config.channels.max(1) as usize;
    let sample_rate = config.sample_rate; // cpal 0.18: SampleRate is u32
    let buffer_for_cb = buffer.clone();

    let stream = device
        .build_input_stream(
            config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if channels == 1 {
                    buffer_for_cb.push(data);
                } else {
                    // Downmix interleaved frames to mono.
                    let mut mono = Vec::with_capacity(data.len() / channels);
                    for frame in data.chunks(channels) {
                        let sum: f32 = frame.iter().sum();
                        mono.push(sum / channels as f32);
                    }
                    buffer_for_cb.push(&mono);
                }
            },
            move |err| {
                tracing::warn!(error = %err, "audio input stream error");
            },
            None,
        )
        .map_err(|e| AppError::AudioCaptureFailed(e.to_string()))?;

    stream
        .play()
        .map_err(|e| AppError::AudioCaptureFailed(e.to_string()))?;

    tracing::info!(
        device = %device_id,
        sample_rate,
        channels = %channels,
        "audio device initialized"
    );

    Ok(CaptureHandle {
        stream,
        device_id: device_id.to_string(),
        sample_rate,
        channels: channels as u16,
    })
}
