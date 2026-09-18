import { useCallback, useEffect, useMemo, useState } from "react";
import { BookOpen, MonitorPlay, Search } from "lucide-react";

import PageHeader, { EmptyHint, Panel } from "@/components/PageHeader";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Separator } from "@/components/ui/separator";
import { useBooks } from "@/hooks/useBooks";
import { bibleApi, presentationApi } from "@/lib/api";
import { formatReference } from "@/lib/reference";
import type {
  Passage,
  ScriptureReference,
  TranslationStatus,
  Verse,
} from "@/types";

/**
 * Bible screen: browse the local SQLite translation and send a passage to the
 * projector. No verse text is hardcoded here — everything is read through the
 * repository.
 */
export default function BiblePage() {
  const { books, bookNames } = useBooks();
  const [translations, setTranslations] = useState<TranslationStatus[]>([]);
  const [translationId, setTranslationId] = useState<string>("");
  const [bookId, setBookId] = useState<number | undefined>();
  const [chapter, setChapter] = useState(1);
  const [verse, setVerse] = useState<number | undefined>();
  const [endVerse, setEndVerse] = useState<number | undefined>();

  const [passage, setPassage] = useState<Passage | null>(null);
  const [loading, setLoading] = useState(false);
  const [projecting, setProjecting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [query, setQuery] = useState("");
  const [results, setResults] = useState<Verse[]>([]);
  const [searching, setSearching] = useState(false);

  // Load translations once, then default to the flagged/configured one.
  useEffect(() => {
    bibleApi
      .listBibleTranslations()
      .then((list) => {
        setTranslations(list);
        const preferred =
          list.find((t) => t.translation.isDefault)?.translation.id ??
          list[0]?.translation.id;
        if (preferred) {
          setTranslationId(preferred);
        }
      })
      .catch(() => setTranslations([]));
  }, []);

  // Default to the first book so the screen is usable immediately.
  useEffect(() => {
    if (bookId === undefined && books.length > 0) {
      setBookId(books[0].id);
    }
  }, [books, bookId]);

  const reference: ScriptureReference | null = useMemo(
    () =>
      bookId === undefined
        ? null
        : {
            bookId,
            chapter,
            startVerse: verse,
            endVerse: endVerse ?? verse,
          },
    [bookId, chapter, verse, endVerse],
  );

  const loadPassage = useCallback(async () => {
    if (!reference || !translationId) {
      return;
    }
    setLoading(true);
    setError(null);
    try {
      setPassage(
        await bibleApi.getPassage({
          translationId,
          bookId: reference.bookId,
          chapter: reference.chapter,
          startVerse: reference.startVerse,
          endVerse: reference.endVerse,
        }),
      );
    } catch (e: unknown) {
      setPassage(null);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [reference, translationId]);

  useEffect(() => {
    void loadPassage();
  }, [loadPassage]);

  async function display() {
    if (!passage) {
      return;
    }
    setProjecting(true);
    try {
      await presentationApi.projectPassage(passage);
      setError(null);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setProjecting(false);
    }
  }

  async function runSearch() {
    if (!translationId || query.trim().length < 2) {
      setResults([]);
      return;
    }
    setSearching(true);
    try {
      setResults(await bibleApi.searchBible(translationId, query.trim()));
    } catch (e: unknown) {
      setResults([]);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setSearching(false);
    }
  }

  const selectedBook = books.find((b) => b.id === bookId);
  const chapterCount = Math.max(selectedBook?.chapters ?? 1, 1);

  return (
    <>
      <PageHeader
        title="Bible"
        subtitle="Look up a verse and send it to the screen"
        actions={
          translations.length === 0 ? (
            <Badge variant="warning">no translation installed</Badge>
          ) : (
            <Badge variant="muted">{translations.length} installed</Badge>
          )
        }
      />

      {error ? (
        <p className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
          {error}
        </p>
      ) : null}

      <div className="grid gap-4 lg:grid-cols-[19rem_1fr]">
        <div className="flex flex-col gap-4">
          <Panel title="Find a verse">
            <div className="space-y-3">
              <div className="space-y-1.5">
                <Label htmlFor="bible-translation">Bible</Label>
                <Select value={translationId} onValueChange={setTranslationId}>
                  <SelectTrigger id="bible-translation">
                    <SelectValue placeholder="Pick a Bible" />
                  </SelectTrigger>
                  <SelectContent>
                    {translations.map(({ translation, verseCount }) => (
                      <SelectItem key={translation.id} value={translation.id}>
                        {translation.abbreviation ?? translation.name}
                        {verseCount === 0 ? " — no text added yet" : ""}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              <div className="space-y-1.5">
                <Label htmlFor="bible-book">Book</Label>
                <Select
                  value={bookId === undefined ? undefined : String(bookId)}
                  onValueChange={(value) => {
                    setBookId(Number(value));
                    setChapter(1);
                    setVerse(undefined);
                    setEndVerse(undefined);
                  }}
                >
                  <SelectTrigger id="bible-book">
                    <SelectValue placeholder="Select a book" />
                  </SelectTrigger>
                  <SelectContent>
                    {books.map((book) => (
                      <SelectItem key={book.id} value={String(book.id)}>
                        {book.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              <div className="grid grid-cols-3 gap-2">
                <div className="space-y-1.5">
                  <Label htmlFor="bible-chapter">Chapter</Label>
                  <Input
                    id="bible-chapter"
                    type="number"
                    min={1}
                    max={chapterCount}
                    value={chapter}
                    onChange={(e) =>
                      setChapter(Math.max(1, Number(e.target.value) || 1))
                    }
                  />
                </div>
                <div className="space-y-1.5">
                  <Label htmlFor="bible-verse">Verse</Label>
                  <Input
                    id="bible-verse"
                    type="number"
                    min={1}
                    placeholder="—"
                    value={verse ?? ""}
                    onChange={(e) =>
                      setVerse(
                        e.target.value === ""
                          ? undefined
                          : Number(e.target.value),
                      )
                    }
                  />
                </div>
                <div className="space-y-1.5">
                  <Label htmlFor="bible-end">To</Label>
                  <Input
                    id="bible-end"
                    type="number"
                    min={1}
                    placeholder="—"
                    value={endVerse ?? ""}
                    onChange={(e) =>
                      setEndVerse(
                        e.target.value === ""
                          ? undefined
                          : Number(e.target.value),
                      )
                    }
                  />
                </div>
              </div>

              <p className="text-xs text-muted-foreground">
                Leave the verse fields blank for a whole chapter. “To” creates a
                range.
                {selectedBook
                  ? ` ${selectedBook.name} has ${chapterCount} chapters.`
                  : ""}
              </p>
            </div>
          </Panel>

          <Panel title="Search words">
            <div className="flex gap-2">
              <Input
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    void runSearch();
                  }
                }}
                placeholder="Search verse text…"
              />
              <Button
                variant="outline"
                size="icon"
                aria-label="Search"
                disabled={searching}
                onClick={() => void runSearch()}
              >
                <Search className="size-4" />
              </Button>
            </div>
            {results.length > 0 ? (
              <>
                <Separator className="my-3" />
                <ScrollArea className="h-44">
                  <ul className="space-y-1 pr-2">
                    {results.map((found) => (
                      <li
                        key={`${found.bookId}-${found.chapter}-${found.verse}`}
                      >
                        <button
                          type="button"
                          className="w-full rounded-md px-2 py-1.5 text-left text-xs transition-colors hover:bg-accent"
                          onClick={() => {
                            setBookId(found.bookId);
                            setChapter(found.chapter);
                            setVerse(found.verse);
                            setEndVerse(found.verse);
                          }}
                        >
                          <span className="font-medium">
                            {bookNames.get(found.bookId)} {found.chapter}:
                            {found.verse}
                          </span>
                          <span className="block truncate text-muted-foreground">
                            {found.text}
                          </span>
                        </button>
                      </li>
                    ))}
                  </ul>
                </ScrollArea>
              </>
            ) : null}
          </Panel>
        </div>

        <PassagePanel
          passage={passage}
          loading={loading}
          projecting={projecting}
          hasTranslations={translations.length > 0}
          emptyReference={reference ? formatReference(reference, bookNames) : ""}
          onDisplay={() => void display()}
        />
      </div>
    </>
  );
}

/** Passage preview + Display action. */
function PassagePanel({
  passage,
  loading,
  projecting,
  hasTranslations,
  emptyReference,
  onDisplay,
}: {
  passage: Passage | null;
  loading: boolean;
  projecting: boolean;
  hasTranslations: boolean;
  emptyReference: string;
  onDisplay: () => void;
}) {
  return (
    <Panel
      title="Verse"
      actions={
        <Button
          variant="success"
          size="sm"
          disabled={!passage || projecting}
          onClick={onDisplay}
        >
          <MonitorPlay className="size-3.5" />
          Show on screen
        </Button>
      }
    >
      {loading ? (
        <EmptyHint>Loading…</EmptyHint>
      ) : !passage ? (
        <EmptyHint>
          {hasTranslations
            ? "Pick a verse on the left to see it here."
            : "No Bible text added yet. Add one from Settings → Bible text."}
        </EmptyHint>
      ) : passage.verses.length === 0 ? (
        <EmptyHint>
          There is no text saved for {emptyReference}. Choose a different verse.
        </EmptyHint>
      ) : (
        <article className="space-y-4">
          <header className="flex flex-wrap items-baseline justify-between gap-3">
            <h2 className="flex items-center gap-2 text-2xl font-semibold tracking-tight">
              <BookOpen className="size-5 text-muted-foreground" />
              {passage.reference}
            </h2>
            <span className="text-xs tracking-[0.2em] text-muted-foreground uppercase">
              {passage.translationId}
            </span>
          </header>
          <p className="selectable text-lg leading-relaxed">{passage.text}</p>
          <Separator />
          <ul className="space-y-2">
            {passage.verses.map((verse) => (
              <li key={verse.verse} className="flex gap-3 text-sm">
                <span className="w-8 shrink-0 text-right font-mono text-muted-foreground">
                  {verse.verse}
                </span>
                <span className="selectable">{verse.text}</span>
              </li>
            ))}
          </ul>
        </article>
      )}
    </Panel>
  );
}
