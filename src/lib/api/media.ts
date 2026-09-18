import { command } from "./client";
import type { MediaItem } from "../../types";

/** Every media record registered in the local library. */
export function listMedia(): Promise<MediaItem[]> {
  return command<MediaItem[]>("list_media");
}

/**
 * Registers a local file. The file itself stays on disk — SQLite stores only
 * metadata, so large videos are never copied into the database.
 */
export function importMedia(path: string): Promise<MediaItem> {
  return command<MediaItem>("import_media", { request: { path } });
}

export function removeMedia(id: string): Promise<void> {
  return command<void>("remove_media", { id });
}
