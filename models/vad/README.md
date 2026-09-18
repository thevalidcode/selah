# VAD models

Selah's default voice-activity detector is a dependency-free energy VAD
(`src-tauri/src/speech/vad.rs`): it needs no model file and runs on any CPU.

This directory is reserved for a Silero VAD ONNX model when the detector is
upgraded behind the same `VoiceActivityDetector` trait. Nothing is required
here for the current build, and no model is ever downloaded automatically.
