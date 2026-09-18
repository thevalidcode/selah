import { command } from "./client";
import type { DetectionResult } from "../../types";

/**
 * Deterministic content detection over a transcript (or any text).
 *
 * The parse itself happens in Rust — nothing about Scripture parsing is
 * reimplemented in the frontend.
 */
export function detectScripture(transcript: string): Promise<DetectionResult[]> {
  return command<DetectionResult[]>("detect_scripture", { transcript });
}
