/**
 * Typing or pasting Bible verses by hand.
 *
 * Selah ships no Bible text, so the words an operator supplies are their own
 * responsibility — but the *shape* of what they paste is not. This module does
 * the checking that can be done without the database: a real JSON array, the
 * documented field names, sane numbers, one book and chapter, and words that
 * are words rather than markup. The reference itself is validated on the Rust
 * side against the canonical book registry.
 */

import type { CustomVerseInput } from "@/types";

/** Longest a single verse may be; the longest verse in the Bible is shorter. */
export const MAX_VERSE_CHARS = 2000;
/** Longest a single typed verse (the "one verse" switch) may be. */
export const MAX_SINGLE_VERSE_CHARS = 400;
/** Highest verse number accepted, used as a typo guard. */
const MAX_VERSE_NUMBER = 200;
/** Most verses accepted in one paste. */
const MAX_VERSE_COUNT = 400;

/** The exact shape an AI agent is asked to return. */
export const VERSE_ARRAY_EXAMPLE = `[
  { "book": "John", "chapter": 3, "verse": 16, "text": "For God so loved the world…" },
  { "book": "John", "chapter": 3, "verse": 17, "text": "For God sent not his Son…" }
]`;

/** The fields each object may contain — and nothing else. */
const ALLOWED_FIELDS = ["book", "chapter", "verse", "text"];

export interface ParsedVerses {
  book: string;
  chapter: number;
  verses: CustomVerseInput[];
  /** Verse numbers inside the range that were not supplied. */
  missing: number[];
}

/** Either a parsed paste or the reason it was rejected. */
export type ParseResult =
  | { ok: true; value: ParsedVerses }
  | { ok: false; error: string };

/**
 * The prompt to hand an AI agent.
 *
 * It names the translation as well as the chapter — the words differ between
 * versions, and a reply in the wrong one is worse than no reply — and asks for
 * the array and *only* the array: models like to add a friendly sentence and a
 * closing note, and both would break the paste.
 */
export function buildAiPrompt(
  book: string,
  chapter: number,
  translation?: string,
): string {
  const where = book.trim() ? `${book.trim()} ${chapter}` : "the chapter I name";
  const version = translation?.trim();
  return [
    `Return every verse of ${where} of the Bible${version ? ` in the ${version}` : ""} as a JSON array, and nothing else.`,
    "",
    "Rules:",
    "1. Reply with the array only — no introduction, no explanation, no code fences, no closing note.",
    `2. One object per verse, using exactly these keys: ${ALLOWED_FIELDS.join(", ")}.`,
    `3. "book" is the book name ("${book.trim() || "John"}"), "chapter" is the chapter number, "verse" is the verse number, "text" is the verse words only.`,
    "4. Do not put the book, chapter or verse number inside the verse text.",
    '5. Order the verses by number, and use the same wording for "book" and "chapter" in every object.',
    "",
    "Example of the exact format expected:",
    VERSE_ARRAY_EXAMPLE,
  ].join("\n");
}


/**
 * Parses a pasted array of verses.
 *
 * Everything is checked before anything is offered for saving, and the first
 * problem found is reported in plain language — an operator should not have to
 * work out which of ninety verses is wrong.
 */
export function parseVerseArray(input: string): ParseResult {
  const text = input.trim();
  if (text.length === 0) {
    return { ok: false, error: "Paste the verses first." };
  }

  // Models often add a code fence; stripping it is kinder than refusing.
  const unfenced = text
    .replace(/^```(?:json)?\s*/i, "")
    .replace(/\s*```$/, "")
    .trim();

  let parsed: unknown;
  try {
    parsed = JSON.parse(unfenced);
  } catch (e: unknown) {
    return {
      ok: false,
      error:
        "That is not readable as JSON. It should start with [ and end with ], " +
        `with one object per verse. (${e instanceof Error ? e.message : String(e)})`,
    };
  }

  if (!Array.isArray(parsed)) {
    return {
      ok: false,
      error:
        "That is JSON, but not an array of verses. The whole paste should be [ { … }, { … } ].",
    };
  }
  if (parsed.length === 0) {
    return {
      ok: false,
      error: "The array is empty — there are no verses in it.",
    };
  }
  if (parsed.length > MAX_VERSE_COUNT) {
    return {
      ok: false,
      error: `That is ${parsed.length} verses, more than one chapter. Paste a single chapter at a time.`,
    };
  }

  const verses: CustomVerseInput[] = [];
  const seen = new Set<number>();
  let book = "";
  let chapter = 0;

  for (const [index, entry] of parsed.entries()) {
    const where = `Verse ${index + 1} in the list`;

    if (typeof entry !== "object" || entry === null || Array.isArray(entry)) {
      return { ok: false, error: `${where} is not an object like { … }.` };
    }
    const row = entry as Record<string, unknown>;

    const extra = Object.keys(row).filter(
      (key) => !ALLOWED_FIELDS.includes(key),
    );
    if (extra.length > 0) {
      return {
        ok: false,
        error: `${where} has extra fields (${extra.join(", ")}). Only ${ALLOWED_FIELDS.join(", ")} are expected.`,
      };
    }

    const rowBook = typeof row.book === "string" ? row.book.trim() : "";
    if (rowBook.length === 0) {
      return { ok: false, error: `${where} has no "book".` };
    }

    const rowChapter = toPositiveInteger(row.chapter);
    if (rowChapter === null) {
      return { ok: false, error: `${where} has no chapter number.` };
    }

    const rowVerse = toPositiveInteger(row.verse);
    if (rowVerse === null || rowVerse > MAX_VERSE_NUMBER) {
      return {
        ok: false,
        error: `${where} has an impossible verse number (${String(row.verse)}).`,
      };
    }

    if (typeof row.text !== "string") {
      return { ok: false, error: `${where} has no "text".` };
    }
    const verseText = row.text.replace(/\s+/g, " ").trim();
    if (verseText.length === 0) {
      return { ok: false, error: `${where} has no words in its "text".` };
    }
    if (verseText.length > MAX_VERSE_CHARS) {
      return {
        ok: false,
        error: `${where} is ${verseText.length} characters, which is longer than a verse.`,
      };
    }

    if (book === "") {
      book = rowBook;
      chapter = rowChapter;
    } else if (rowBook.toLowerCase() !== book.toLowerCase()) {
      return {
        ok: false,
        error: `The list mixes "${book}" with "${rowBook}". Save one chapter at a time.`,
      };
    } else if (rowChapter !== chapter) {
      return {
        ok: false,
        error: `The list mixes chapter ${chapter} with chapter ${rowChapter}. Save one chapter at a time.`,
      };
    }

    if (seen.has(rowVerse)) {
      return { ok: false, error: `${where} repeats verse ${rowVerse}.` };
    }
    seen.add(rowVerse);

    verses.push({
      book: rowBook,
      chapter: rowChapter,
      verse: rowVerse,
      text: verseText,
    });
  }

  verses.sort((a, b) => a.verse - b.verse);
  const numbers = [...seen].sort((a, b) => a - b);

  return {
    ok: true,
    value: { book, chapter, verses, missing: gaps(numbers) },
  };
}


/**
 * Checks the words typed into the single-verse editor.
 *
 * The rule is deliberately strict: only the verse message belongs here. A line
 * that still carries its own reference (`John 11:35 — Jesus wept`) is the most
 * common mistake, and it would be projected to the congregation verbatim.
 */
export function validateSingleVerse(input: string): {
  ok: boolean;
  text: string;
  error: string | null;
} {
  const text = input.replace(/\s+/g, " ").trim();

  if (text.length === 0) {
    return { ok: false, text, error: "Type the verse words." };
  }
  if (text.length > MAX_SINGLE_VERSE_CHARS) {
    return {
      ok: false,
      text,
      error: `That is ${text.length} characters — longer than a verse. Use the whole-chapter box for a chapter.`,
    };
  }
  if (text.startsWith("{") || text.startsWith("[")) {
    return {
      ok: false,
      text,
      error: "Only the words of the verse belong here — not code or an array.",
    };
  }
  if (!/[A-Za-z\u00c0-\u024f\u0400-\u04ff]/.test(text)) {
    return { ok: false, text, error: "That has no words in it." };
  }
  if (looksLikeReference(text)) {
    return {
      ok: false,
      text,
      error:
        "Remove the reference from the words. Type only the verse message — for example, Jesus wept.",
    };
  }
  if (hasVersePrefix(text)) {
    return {
      ok: false,
      text,
      error:
        'Remove the leading "verse:" label. Only the verse message belongs here.',
    };
  }

  return { ok: true, text, error: null };
}

/** Whether a line begins with something like `John 11:35` or `Ps 23:1`. */
function looksLikeReference(text: string): boolean {
  return /^[1-3]?\s*[A-Za-z]{2,}\.?\s+\d{1,3}\s*:\s*\d{1,3}/.test(
    text.slice(0, 40),
  );
}

/** Whether a line begins with `verse:` or `verse 1:`. */
function hasVersePrefix(text: string): boolean {
  return /^verse\s*\d*\s*[:.]/i.test(text);
}

/** Verse numbers missing between the lowest and highest supplied. */
function gaps(verses: number[]): number[] {
  if (verses.length === 0) {
    return [];
  }
  const lowest = verses[0];
  const highest = verses[verses.length - 1];
  const missing: number[] = [];
  for (let number = lowest; number <= highest; number += 1) {
    if (!verses.includes(number)) {
      missing.push(number);
    }
  }
  return missing;
}

/** Accepts a number or a numeric string; rejects anything else. */
function toPositiveInteger(value: unknown): number | null {
  if (typeof value === "number") {
    return Number.isInteger(value) && value >= 1 ? value : null;
  }
  if (typeof value === "string" && /^\d+$/.test(value.trim())) {
    const parsed = Number(value.trim());
    return parsed >= 1 ? parsed : null;
  }
  return null;
}
