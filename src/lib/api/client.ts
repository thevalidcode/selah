/**
 * Centralized Tauri command wrappers.
 *
 * React components never call `invoke` directly — they use this API layer so
 * command names and payload shapes live in exactly one place.
 */

import { invoke } from "@tauri-apps/api/core";

/**
 * Unwraps a Selah error payload into a readable Error.
 * Rust commands serialize failures as { kind, message }.
 */
export function toError(payload: unknown): Error {
  if (
    payload &&
    typeof payload === "object" &&
    "message" in payload &&
    typeof (payload as { message: unknown }).message === "string"
  ) {
    const { kind, message } = payload as { kind?: string; message: string };
    return new Error(kind ? `${message} (${kind})` : message);
  }
  if (payload instanceof Error) {
    return payload;
  }
  return new Error(String(payload));
}

/** Runs a command and normalizes failures to JS errors. */
export async function command<T>(name: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(name, args);
  } catch (e) {
    throw toError(e);
  }
}