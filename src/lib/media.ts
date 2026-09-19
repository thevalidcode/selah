/**
 * Media URLs.
 *
 * The projector window is a separate webview, so it cannot read a `file://`
 * path. Files are served through Tauri's `asset:` protocol instead; which
 * folders may be read is granted at runtime from the media folder the operator
 * chose (see `src-tauri/src/media/access.rs`). Nothing here hardcodes a path.
 */

import { convertFileSrc } from "@tauri-apps/api/core";

import type { MediaKind } from "@/types";

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
