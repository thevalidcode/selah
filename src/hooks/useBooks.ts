import { useEffect, useMemo, useState } from "react";

import { bibleApi } from "@/lib/api";
import type { BibleBook } from "@/types";

/**
 * Loads the canonical book registry from SQLite.
 *
 * Books are read from the database rather than duplicated in TypeScript, so
 * the frontend can never disagree with the Rust registry.
 */
export function useBooks(): {
  books: BibleBook[];
  bookNames: ReadonlyMap<number, string>;
  loading: boolean;
} {
  const [books, setBooks] = useState<BibleBook[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    bibleApi
      .listBooks()
      .then(setBooks)
      .catch(() => setBooks([]))
      .finally(() => setLoading(false));
  }, []);

  const bookNames = useMemo(
    () => new Map(books.map((b) => [b.id, b.name])),
    [books],
  );

  return { books, bookNames, loading };
}
