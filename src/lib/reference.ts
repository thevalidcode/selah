import type { ScriptureReference } from "@/types";

/**
 * Formats a Scripture reference for display.
 *
 * Book names come from the canonical registry via the `books` table, so
 * nothing is hardcoded here — pass the map returned by {@link useBooks}.
 */
export function formatReference(
  reference: ScriptureReference,
  bookNames: ReadonlyMap<number, string>,
): string {
  const book = bookNames.get(reference.bookId) ?? `Book ${reference.bookId}`;
  const { chapter, startVerse, endVerse } = reference;

  if (startVerse === undefined) {
    return `${book} ${chapter}`;
  }
  if (endVerse === undefined || endVerse === startVerse) {
    return `${book} ${chapter}:${startVerse}`;
  }
  return `${book} ${chapter}:${startVerse}-${endVerse}`;
}

/** Short label for the passage header, e.g. "PSALM 23". */
export function uppercaseReference(label: string): string {
  return label.toUpperCase();
}
