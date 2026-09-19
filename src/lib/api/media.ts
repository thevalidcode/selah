import { command } from "./client";
import type {
  DirectoryListing,
  MediaItem,
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
