//! Microphone device enumeration.
//!
//! Device ids use cpal's stable `DeviceId` (Display form). This is isolated
//! here so identity handling can evolve without touching callers.

use crate::errors::AppError;
use cpal::traits::{DeviceTrait, HostTrait};
use serde::Serialize;

/// A discoverable audio input device.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioDeviceInfo {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub default_sample_rate: u32,
    pub channels: u16,
}

/// All available input devices.
pub fn list_input_devices() -> Result<Vec<AudioDeviceInfo>, AppError> {
    let host = cpal::default_host();
    let default_id = host.default_input_device().and_then(|d| d.id().ok());

    let devices = host
        .input_devices()
        .map_err(|e| AppError::AudioQueryFailed(e.to_string()))?;

    let mut result = Vec::new();
    for device in devices {
        let Ok(description) = device.description() else {
            continue;
        };
        let Ok(id) = device.id() else {
            continue;
        };
        let id = id.to_string();
        let name = description.name().to_string();
        let (rate, channels) = device
            .default_input_config()
            .ok()
            .map(|c| (c.sample_rate(), c.channels()))
            .unwrap_or((0, 0));
        result.push(AudioDeviceInfo {
            is_default: default_id.as_ref().is_some_and(|d| d.to_string() == id),
            id,
            name,
            default_sample_rate: rate,
            channels,
        });
    }

    Ok(result)
}

/// The default input device, if any.
pub fn default_input_device() -> Result<Option<AudioDeviceInfo>, AppError> {
    let host = cpal::default_host();
    let Some(device) = host.default_input_device() else {
        return Ok(None);
    };
    let id = device
        .id()
        .map_err(|e| AppError::AudioQueryFailed(e.to_string()))?
        .to_string();
    let name = device
        .description()
        .map_err(|e| AppError::AudioQueryFailed(e.to_string()))?
        .name()
        .to_string();
    let (rate, channels) = device
        .default_input_config()
        .ok()
        .map(|c| (c.sample_rate(), c.channels()))
        .unwrap_or((0, 0));
    Ok(Some(AudioDeviceInfo {
        id,
        name,
        is_default: true,
        default_sample_rate: rate,
        channels,
    }))
}

/// Looks up a `cpal::Device` by its configured id.
pub fn find_device(id: &str) -> Result<Option<cpal::Device>, AppError> {
    let host = cpal::default_host();
    let devices = host
        .input_devices()
        .map_err(|e| AppError::AudioQueryFailed(e.to_string()))?;
    for device in devices {
        if let Ok(device_id) = device.id() {
            if device_id.to_string() == id {
                return Ok(Some(device));
            }
        }
    }
    Ok(None)
}
