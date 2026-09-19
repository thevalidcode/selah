import { command } from "./client";
import type {
  BibleBook,
  BibleSearchResult,
  CatalogueEntry,
  CustomVerseImportResult,
  Passage,
  SaveCustomVersesRequest,
  TopicSuggestion,
  Translation,
  TranslationStatus,
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
  translationId: string | null | undefined,
  query: string,
  limit = 25,
): Promise<BibleSearchResult> {
  return command<BibleSearchResult>("search_bible", {
    // `null` searches every installed translation, so an operator who cannot
    // remember which version a phrase came from still finds it.
    translationId: translationId ?? null,
    query,
    limit,
  });
}

/** Themes to offer when a search finds nothing (references only). */
export function suggestBibleTopics(
  query: string,
  limit = 6,
): Promise<TopicSuggestion[]> {
  return command<TopicSuggestion[]>("suggest_bible_topics", { query, limit });
}

/**
 * Saves verses the operator typed or pasted.
 *
 * The backend checks every reference against the canonical book registry and
 * rejects mixed books/chapters, so a bad paste is reported rather than stored.
 */
export function saveCustomVerses(
  request: SaveCustomVersesRequest,
): Promise<CustomVerseImportResult> {
  return command<CustomVerseImportResult>("save_custom_verses", {
    request: {
      translationId: request.translationId ?? null,
      name: request.name ?? null,
      abbreviation: request.abbreviation ?? null,
      verses: request.verses,
    },
  });
}

export function setDefaultTranslation(translationId: string): Promise<void> {
  return command<void>("set_default_translation", { translationId });
}

// ------------------------------------------------- managing translations

/**
 * Every published translation Selah knows about, merged with what is installed.
 *
 * Metadata only: this is the "Add a Translation" list. Nothing is downloaded,
 * and adding one creates the translation so verses can be stored under it.
 */
export function listTranslationCatalogue(): Promise<CatalogueEntry[]> {
  return command<CatalogueEntry[]>("list_translation_catalogue");
}

/**
 * Adds a translation so verses can be kept under it.
 *
 * A published id (`msg`) brings its own name, abbreviation and language; a
 * translation Selah does not know needs a name of its own.
 */
export function addTranslation(request: {
  id: string;
  name?: string;
  abbreviation?: string;
  language?: string;
}): Promise<TranslationStatus> {
  return command<TranslationStatus>("add_translation", {
    request: {
      id: request.id,
      name: request.name ?? null,
      abbreviation: request.abbreviation ?? null,
      language: request.language ?? null,
    },
  });
}

/** Renames a translation the operator added. Selah's own three are refused. */
export function updateTranslation(request: {
  id: string;
  name: string;
  abbreviation?: string;
  language?: string;
}): Promise<Translation> {
  return command<Translation>("update_translation", {
    request: {
      id: request.id,
      name: request.name,
      abbreviation: request.abbreviation ?? null,
      language: request.language ?? null,
    },
  });
}

/** Removes a translation and every verse stored under it. */
export function deleteTranslation(id: string): Promise<void> {
  return command<void>("delete_translation", { id });
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