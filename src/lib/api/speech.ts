import { command } from "./client";
import type { SpeechManagerState } from "../../types";

export function getSpeechState(): Promise<SpeechManagerState> {
  return command<SpeechManagerState>("get_speech_state");
}

/** Starts microphone capture + the recognition worker. */
export function startListening(): Promise<SpeechManagerState> {
  return command<SpeechManagerState>("start_listening");
}

export function stopListening(): Promise<SpeechManagerState> {
  return command<SpeechManagerState>("stop_listening");
}