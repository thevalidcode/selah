import { command } from "./client";
import type {
  DirectoryListing,
  MediaItem,
  MediaPlaybackAction,
  MediaPlaybackState,
  MediaScan,
  PresentationState,
} from "../../types";

/** Every media record registered in the local library. */
export function listMedia(): Promise<MediaItem[]> {
  return command<MediaItem[]>("list_media");
}

/**
 * Lists a folder so it can be picked.
 *
 * Omit `path` to start at the user's home folder.
 */
export function browseDirectory(path?: string): Promise<DirectoryListing> {
  return command<DirectoryListing>("browse_directory", {
    path: path ?? null,
  });
}

/**
 * Loads a folder into the library.
 *
 * Selah reads the media inside it — no path is hardcoded anywhere — remembers
 * the folder for next time, and reports what it found.
 */
export function loadMediaDirectory(
  path: string,
  recursive = false,
): Promise<MediaScan> {
  return command<MediaScan>("load_media_directory", {
    request: { path, recursive },
  });
}

/**
 * Registers a single local file. The file itself stays on disk — SQLite stores
 * only metadata, so large videos are never copied into the database.
 */
export function importMedia(path: string): Promise<MediaItem> {
  return command<MediaItem>("import_media", { request: { path } });
}

export function removeMedia(id: string): Promise<void> {
  return command<void>("remove_media", { id });
}

/** Puts a media file on the congregation's screen. */
export function projectMedia(
  path: string,
  title?: string,
): Promise<PresentationState> {
  return command<PresentationState>("project_media", {
    request: { path, title: title ?? null },
  });
}

// ------------------------------------------------------------ playback

/** What the projector says it is playing right now. */
export function getMediaPlaybackState(): Promise<MediaPlaybackState> {
  return command<MediaPlaybackState>("get_media_playback_state");
}

/**
 * Asks the projector window to play, pause, restart or stop what is on it.
 *
 * The command goes through Rust because the projector is a separate webview:
 * only it can touch the `<video>` element, so the operator screen sends an
 * instruction rather than reaching across.
 */
export function controlMediaPlayback(
  action: MediaPlaybackAction,
): Promise<MediaPlaybackState> {
  return command<MediaPlaybackState>("control_media_playback", {
    request: { action },
  });
}

/**
 * Reports what the projector's player is doing (called by the projector
 * window itself, never by the operator screen).
 */
export function reportMediaPlayback(state: {
  playing: boolean;
  positionMs?: number;
  durationMs?: number;
  ended?: boolean;
}): Promise<MediaPlaybackState> {
  return command<MediaPlaybackState>("report_media_playback", {
    report: {
      playing: state.playing,
      positionMs: Math.max(0, Math.round(state.positionMs ?? 0)),
      durationMs: Math.max(0, Math.round(state.durationMs ?? 0)),
      ended: state.ended ?? false,
    },
  });
}
