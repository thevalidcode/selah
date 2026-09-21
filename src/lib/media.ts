/**
 * Media URLs.
 *
 * The projector window is a separate webview, so it cannot read a `file://`
 * path. Files are served through Tauri's `asset:` protocol instead; which
 * folders may be read is granted at runtime from the media folder the operator
 * chose (see `src-tauri/src/media/access.rs`). Nothing here hardcodes a path.
 */

import { convertFileSrc } from "@tauri-apps/api/core";

import type { MediaClip, MediaKind } from "@/types";

/** A URL the webview can load for a local media file. */
export function mediaUrl(path: string): string {
  return convertFileSrc(path, "asset");
}

/**
 * Guesses the media kind from a file name.
 *
 * Only needed for files registered before the kind was stored; everything
 * loaded by the current build carries its kind already.
 */
export function mediaKindFromPath(path: string): MediaKind {
  if (/\.(mp4|mov|m4v|mkv|webm)$/i.test(path)) {
    return "video";
  }
  if (/\.(mp3|wav|flac|ogg|m4a)$/i.test(path)) {
    return "audio";
  }
  return "image";
}

// -------------------------------------------------------------- video ranges

/**
 * The part of a video the operator chose to show, read out of a media record's
 * metadata.
 *
 * Metadata is free-form JSON, so anything unexpected is treated as "no range
 * chosen" rather than breaking the screen that asked for it.
 */
export function clipFromMetadata(
  metadata: Record<string, unknown> | undefined,
): MediaClip | undefined {
  const raw = metadata?.clip;
  if (!raw || typeof raw !== "object") {
    return undefined;
  }
  const { startMs, endMs, repeat } = raw as Record<string, unknown>;
  if (typeof startMs !== "number" || !Number.isFinite(startMs)) {
    return undefined;
  }
  return {
    startMs: Math.max(0, Math.round(startMs)),
    endMs:
      typeof endMs === "number" && Number.isFinite(endMs)
        ? Math.round(endMs)
        : undefined,
    repeat: repeat === true,
  };
}

/** Milliseconds as `m:ss`, for anything said about a video's timeline. */
export function formatClipTime(ms: number): string {
  const totalSeconds = Math.max(0, Math.round(ms / 1000));
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes}:${String(seconds).padStart(2, "0")}`;
}

/**
 * A one-line description of a chosen range, or `null` when the whole file plays.
 *
 * The repeat note is only added when it matters — a range that stops at its end
 * says nothing, because that is what a range normally does.
 */
export function describeClip(clip: MediaClip | undefined): string | null {
  if (!clip || (clip.startMs <= 0 && clip.endMs === undefined)) {
    return null;
  }
  const from = formatClipTime(clip.startMs);
  const to = clip.endMs === undefined ? "the end" : formatClipTime(clip.endMs);
  return `${from} – ${to}${clip.repeat ? " · repeats" : ""}`;
}

/**
 * What a player should do when it is at `positionMs`.
 *
 * `"restart"` sends a repeating range back to its start, `"end"` stops a range
 * that should stop, and `"continue"` means there is still something to play. A
 * range with no end runs to the file's own end, which the player reports through
 * its `ended` event — so this never answers `"end"` for one.
 */
export function clipEndAction(
  positionMs: number,
  clip: MediaClip | undefined,
  durationMs = 0,
): "continue" | "restart" | "end" {
  const endMs = clip?.endMs ?? (durationMs > 0 ? durationMs : undefined);
  if (endMs === undefined || positionMs < endMs) {
    return "continue";
  }
  return clip?.repeat ? "restart" : "end";
}
