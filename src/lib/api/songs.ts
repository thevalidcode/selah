import { command } from "./client";
import type { PresentationState, Song, SongInput } from "../../types";

/**
 * Songs in the library.
 *
 * A song is stored as ordered sections, so it can be presented one verse or
 * chorus at a time — the same way Bible verses are presented.
 */

export function listSongs(): Promise<Song[]> {
  return command<Song[]>("list_songs");
}

export function getSong(id: string): Promise<Song | null> {
  return command<Song | null>("get_song", { id });
}

/** Creates a song, or replaces one when `id` is supplied. */
export function saveSong(song: SongInput, id?: string): Promise<Song> {
  return command<Song>("save_song", { request: { id: id ?? null, song } });
}

export function deleteSong(id: string): Promise<void> {
  return command<void>("delete_song", { id });
}

/**
 * Shows a song on the projector, starting at `section` (1-based).
 *
 * The remaining sections are queued, so `showNextItem` / `showPreviousItem`
 * step through the song section by section.
 */
export function projectSong(
  songId: string,
  section?: number,
): Promise<PresentationState> {
  return command<PresentationState>("project_song", {
    request: { songId, section: section ?? null },
  });
}
