import { command } from "./client";
import type {
  BibleBook,
  Passage,
  TranslationStatus,
  Verse,
} from "../../types";

export function listBibleTranslations(): Promise<TranslationStatus[]> {
  return command<TranslationStatus[]>("list_bible_translations");
}

export function listBooks(): Promise<BibleBook[]> {
  return command<BibleBook[]>("list_books");
}

export interface PassageRequest {
  translationId: string;
  bookId: number;
  chapter: number;
  startVerse?: number;
  endVerse?: number;
}

export function getPassage(request: PassageRequest): Promise<Passage> {
  return command<Passage>("get_passage", {
    request: {
      translationId: request.translationId,
      bookId: request.bookId,
      chapter: request.chapter,
      startVerse: request.startVerse ?? null,
      endVerse: request.endVerse ?? null,
    },
  });
}

export function searchBible(
  translationId: string,
  query: string,
  limit = 25,
): Promise<Verse[]> {
  return command<Verse[]>("search_bible", { translationId, query, limit });
}

export function setDefaultTranslation(translationId: string): Promise<void> {
  return command<void>("set_default_translation", { translationId });
}

/** Result of importing a translation document. */
export interface TranslationImportResult {
  translationId: string;
  versesImported: number;
}

/**
 * Imports a translation from a local JSON document the operator is licensed to
 * use. Selah never bundles Bible text.
 */
export function importBibleTranslation(
  path: string,
): Promise<TranslationImportResult> {
  return command<TranslationImportResult>("import_bible_translation", { path });
}

/** Details needed to import one of the common SQLite Bible downloads. */
export interface SqliteImportRequest {
  /** Absolute path to the `.sqlite` file. */
  path: string;
  /** Short id used afterwards, e.g. `kjv`. */
  translationId: string;
  /** Full name shown in the interface, e.g. `King James Version`. */
  name: string;
  abbreviation?: string;
  /** Make this the translation Selah starts with. */
  makeDefault?: boolean;
}

/**
 * Imports a translation from a SQLite file using the widely published
 * `verses(book_id, chapter, number, text)` layout.
 */
export function importSqliteBibleTranslation(
  request: SqliteImportRequest,
): Promise<TranslationImportResult> {
  return command<TranslationImportResult>("import_sqlite_bible_translation", {
    request: {
      path: request.path,
      translationId: request.translationId,
      name: request.name,
      abbreviation: request.abbreviation ?? null,
      makeDefault: request.makeDefault ?? false,
    },
  });
}