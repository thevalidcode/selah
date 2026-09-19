/**
 * Turning one pasted block of words into verses.
 *
 * Song words usually arrive as a single paragraph: copied from a website, an
 * email, or a message — sometimes numbered, sometimes not. Typing them in verse
 * by verse is slow and error-prone, so [`splitIntoVerses`] does it instead.
 *
 * The rules are applied in order, most explicit first, so a paste that already
 * says where the verses are is never second-guessed:
 *
 *   1. **Numbered** — `1. … 2. …`, `1) …`, `[1] …`, `Verse 1 …`;
 *   2. **Blank lines** — paragraph blocks separated by an empty line;
 *   3. **Line breaks** — one verse per line;
 *   4. **Sentences** — a single long line is cut at sentence ends.
 *
 * Anything that still comes out too long is broken at the nearest comma, so no
 * verse is ever a wall of text that cannot fit a screen.
 */

/** A verse ready to be saved; labels match the "Verse 1" convention. */
export interface SplitVerse {
  label: string;
  text: string;
}

/** Longest a single verse may be before it is broken further. */
export const MAX_VERSE_CHARS = 180;

/**
 * Splits pasted words into verses.
 *
 * Numbers and whitespace are normalised, empty pieces are dropped, and a paste
 * that produces nothing returns an empty list so the caller can say so.
 */
export function splitIntoVerses(input: string): SplitVerse[] {
  const text = input.replace(/\r\n?/g, "\n").trim();
  if (text.length === 0) {
    return [];
  }

  const chunks = byNumbers(text) ?? byBlocks(text) ?? byLines(text) ?? bySentences(text);
  const broken = chunks.flatMap((chunk) => breakLongVerse(chunk));

  return broken
    // A verse is one paragraph of words: newlines and runs of spaces inside a
    // verse become single spaces, so a pasted block is not left full of gaps.
    .map((chunk) => chunk.replace(/\s+/g, " ").trim())
    .filter((chunk) => chunk.length > 0)
    .map((chunk, index) => ({ label: `Verse ${index + 1}`, text: chunk }));
}

/**
 * Numbered verses: `1.`, `1)`, `[1]`, `Verse 1`, `Verse 1:`.
 *
 * Requires at least two markers, otherwise a single stray number (a year, a
 * count) would look like a verse list.
 */
function byNumbers(text: string): string[] | null {
  const marker = /(?:^|\n)\s*(?:verse\s*)?(\[?\d{1,3}\]?)\s*[.):\]]?\s+/gi;
  const matches = [...text.matchAll(marker)];
  if (matches.length < 2) {
    return null;
  }

  // Only trust the numbering if the numbers actually increase; `2024 15 12` in
  // a sentence is not a verse list.
  const numbers = matches.map((match) => Number(match[1].replace(/\D/g, "")));
  const ascending = numbers.every(
    (value, index) => index === 0 || value === numbers[index - 1] + 1,
  );
  if (!ascending) {
    return null;
  }

  return matches.map((match, index) => {
    const start = (match.index ?? 0) + match[0].length;
    const end =
      index + 1 < matches.length ? (matches[index + 1].index ?? text.length) : text.length;
    return text.slice(start, end);
  });
}

/** Verses separated by a blank line. */
function byBlocks(text: string): string[] | null {
  const blocks = text
    .split(/\n\s*\n+/)
    .map((block) => block.trim())
    .filter((block) => block.length > 0);

  return blocks.length > 1 ? blocks : null;
}

/** Verses separated by line breaks (each line is a verse). */
function byLines(text: string): string[] | null {
  const lines = text
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.length > 0);

  return lines.length > 1 ? lines : null;
}

/** A single long line: cut at sentence ends. */
function bySentences(text: string): string[] {
  const sentences = text
    // A sentence ends at `.`, `!` or `?` (with any closing quote) followed by a
    // space and something that looks like the start of the next sentence.
    .split(/(?<=[.!?]["'”’)]?)\s+(?=[A-Z0-9"“'‘([])/)
    .map((sentence) => sentence.trim())
    .filter((sentence) => sentence.length > 0);

  return sentences.length > 0 ? sentences : [text];
}

/**
 * Breaks a verse that is still too long, at the most natural place available:
 * a sentence end, then a comma/colon/semicolon, then a space.
 */
function breakLongVerse(verse: string, depth = 0): string[] {
  const text = verse.trim();
  if (text.length <= MAX_VERSE_CHARS || depth > 4) {
    return [text];
  }

  const cut = bestCut(text);
  if (cut <= 0) {
    return [text];
  }

  const head = text.slice(0, cut).trim();
  const tail = text.slice(cut).trim();
  return [...breakLongVerse(head, depth + 1), ...breakLongVerse(tail, depth + 1)];
}

/** The index just past the best place to cut a long line. */
function bestCut(text: string): number {
  const limit = Math.min(text.length, MAX_VERSE_CHARS);
  const window = text.slice(0, limit);

  // Sentence end first, then clause punctuation, then any word boundary.
  for (const pattern of [/[.!?]["'”’)]?\s/g, /[,;:]\s/g, /\s/g]) {
    const matches = [...window.matchAll(pattern)];
    const last = matches[matches.length - 1];
    if (last && last.index !== undefined) {
      return last.index + last[0].length;
    }
  }
  return 0;
}
