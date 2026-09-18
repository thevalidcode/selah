import { command } from "./client";
import type {
  AudioCaptureState,
  AudioDeviceInfo,
} from "../../types";

export function listAudioDevices(): Promise<AudioDeviceInfo[]> {
  return command<AudioDeviceInfo[]>("list_audio_devices");
}

export function getDefaultAudioDevice(): Promise<AudioDeviceInfo | null> {
  return command<AudioDeviceInfo | null>("get_default_audio_device");
}

export function startAudioCapture(): Promise<AudioCaptureState> {
  return command<AudioCaptureState>("start_audio_capture");
}

export function stopAudioCapture(): Promise<void> {
  return command<void>("stop_audio_capture");
}

export function getAudioCaptureState(): Promise<AudioCaptureState> {
  return command<AudioCaptureState>("get_audio_capture_state");
}