import { useCallback, useEffect, useMemo, useState } from "react";
import {
  AlertTriangle,
  BookOpen,
  Lightbulb,
  MonitorPlay,
  RotateCcw,
  Search,
  Type,
} from "lucide-react";

import AddTranslationPanel from "@/components/bible/AddTranslationPanel";
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
import { Switch } from "@/components/ui/switch";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useBooks } from "@/hooks/useBooks";
import { useSettings } from "@/hooks/useSettings";
import { bibleApi, presentationApi } from "@/lib/api";
import { estimateFit } from "@/lib/bibleFit";
import { formatReference } from "@/lib/reference";
import type {
  BibleSearchResult,
  DisplayInfo,
  Passage,
  ScriptureReference,
  TranslationStatus,
} from "@/types";

/** Default reference the screen opens on, so it is usable immediately. */
const DEFAULT_CHAPTER = 1;
const DEFAULT_VERSE = 3;
/** Smallest/largest sizes the Bible screen offers, in CSS pixels. */
const MIN_ITEM_SIZE = 20;
const MAX_ITEM_SIZE = 260;
/** Space to leave free for branding at the top or bottom of the screen. */
const BRANDING_ALLOWANCE = 90;

/**
 * Bible screen: browse the local SQLite translation and send a passage to the
 * projector. No verse text is hardcoded here — everything is read through the
 * repository.
 *
 * Two tabs, because there are two jobs:
 *   * **Look up** — find a passage and put it on the screen;
 *   * **Add a Translation** — hold the Bibles Selah does not ship (MSG, NIV,
 *     ESV and the rest), plus the verses for them.
 */
export default function BiblePage() {
  const { books, bookNames } = useBooks();
  const { settings } = useSettings();
  const [tab, setTab] = useState("lookup");

  const [translations, setTranslations] = useState<TranslationStatus[]>([]);
  const [translationId, setTranslationId] = useState<string>("");
  const [bookId, setBookId] = useState<number | undefined>();
  const [chapter, setChapter] = useState(DEFAULT_CHAPTER);
  const [verse, setVerse] = useState<number | undefined>(DEFAULT_VERSE);
  const [endVerse, setEndVerse] = useState<number | undefined>(DEFAULT_VERSE);

  const [passage, setPassage] = useState<Passage | null>(null);
  const [loading, setLoading] = useState(false);
  const [projecting, setProjecting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  // Sizes for this passage only: `null` means "whatever Settings says".
  const [headingSize, setHeadingSize] = useState<number | null>(null);
  const [textSize, setTextSize] = useState<number | null>(null);

  const [query, setQuery] = useState("");
  const [searching, setSearching] = useState(false);
  const [searched, setSearched] = useState(false);
  const [everywhere, setEverywhere] = useState(true);
  const [results, setResults] = useState<BibleSearchResult>({
    verses: [],
    related: false,
    suggestions: [],
  });

  // The screen the projector will use, so the overflow estimate is about the
  // real thing rather than a guess.
  const [displays, setDisplays] = useState<DisplayInfo[]>([]);

  const reloadTranslations = useCallback(
    () =>
      bibleApi
        .listBibleTranslations()
        .then((list) => {
          setTranslations(list);
          return list;
        })
        .catch(() => {
          setTranslations([]);
          return [];
        }),
    [],
  );

  // Load translations once, then default to the flagged/configured one.
  useEffect(() => {
    void reloadTranslations().then((list) => {
      const preferred =
        list.find((t) => t.translation.isDefault)?.translation.id ??
        list[0]?.translation.id;
      if (preferred) {
        setTranslationId((current) => current || preferred);
      }
    });
  }, [reloadTranslations]);

  useEffect(() => {
    presentationApi
      .listDisplays()
      .then(setDisplays)
      .catch(() => setDisplays([]));
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

  /** Effective sizes: the saved projector size unless overridden here. */
  const savedFontSize = settings?.presentation.fontSize ?? 64;
  const effectiveHeadingSize = headingSize ?? Math.round(savedFontSize * 1.3);
  const effectiveTextSize = textSize ?? savedFontSize;
  const usingCustomSizes = headingSize !== null || textSize !== null;

  /** The screen the projector will use, for the overflow estimate. */
  const projectorScreen = useMemo(() => {
    if (displays.length === 0) {
      return null;
    }
    const target = settings?.presentation.displayIndex;
    const chosen =
      displays.find((display) => display.index === target) ??
      displays.find((display) => display.isPrimary) ??
      displays[0];
    return { width: chosen.size[0], height: chosen.size[1] };
  }, [displays, settings?.presentation.displayIndex]);

  /**
   * Whether what is loaded will fit the screen at the chosen sizes.
   *
   * This is a warning, not a gate: the operator can see their room and Selah
   * cannot, so it never stops them showing a passage.
   */
  const fit = useMemo(
    () =>
      passage && passage.verses.length > 0
        ? estimateFit({
            text: passage.text,
            heading: passage.reference,
            headingSize: effectiveHeadingSize,
            textSize: effectiveTextSize,
            screen: projectorScreen,
            // Branding along the top or bottom costs height; down the side it
            // does not, so nothing is reserved for it there.
            brandingHeight:
              settings?.presentation.branding?.position === "left" ||
              settings?.presentation.branding?.position === "right"
                ? 0
                : BRANDING_ALLOWANCE,
          })
        : null,
    [
      passage,
      effectiveHeadingSize,
      effectiveTextSize,
      projectorScreen,
      settings?.presentation.branding?.position,
    ],
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
      // "This verse is not in this Bible" is an ordinary thing to hit — the
      // default reference is chapter 1 verse 3, and not every chapter has three
      // verses. It is answered with the friendly empty state, not an error
      // banner, because nothing has actually gone wrong.
      const message = e instanceof Error ? e.message : String(e);
      setPassage(null);
      setError(message.includes("scriptureNotFound") ? null : message);
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
      // The sizes chosen on this screen travel with the passage; leaving them
      // alone keeps whatever Settings holds.
      await presentationApi.projectPassage(passage, {
        headingSize: headingSize ?? undefined,
        textSize: textSize ?? undefined,
      });
      setError(null);
      setNotice(
        fit?.overflows
          ? "Shown on the screen — watch for it running off the bottom."
          : "Shown on the screen.",
      );
    } catch (e: unknown) {
      setNotice(null);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setProjecting(false);
    }
  }

  /**
   * Searches verse text.
   *
   * `translationId` is `null` when searching everywhere, which is the default:
   * an operator hunting for a phrase rarely remembers which version it came
   * from. When nothing matches exactly the backend comes back with the closest
   * matches, or with themes to try instead — so a search never dead-ends.
   */
  async function runSearch(scope?: string | null) {
    const trimmed = query.trim();
    if (trimmed.length < 2) {
      setResults({ verses: [], related: false, suggestions: [] });
      setSearched(false);
      return;
    }

    setSearching(true);
    try {
      const found = await bibleApi.searchBible(
        scope === undefined ? (everywhere ? null : translationId) : scope,
        trimmed,
      );
      setResults(found);
      setSearched(true);
      setError(null);
    } catch (e: unknown) {
      setResults({ verses: [], related: false, suggestions: [] });
      setSearched(false);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setSearching(false);
    }
  }

  const selectedBook = books.find((b) => b.id === bookId);
  const chapterCount = Math.max(selectedBook?.chapters ?? 1, 1);

  /** Jumps to a verse the operator picked from a result or a suggestion. */
  function goTo(found: {
    bookId: number;
    chapter: number;
    verse: number;
  }) {
    setBookId(found.bookId);
    setChapter(found.chapter);
    setVerse(found.verse);
    setEndVerse(found.verse);
  }

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
      {notice ? (
        <p className="rounded-md border border-success/30 bg-success/10 px-3 py-2 text-sm text-success">
          {notice}
        </p>
      ) : null}

      <Tabs value={tab} onValueChange={setTab} className="space-y-4">
        <TabsList>
          <TabsTrigger value="lookup">Look up a verse</TabsTrigger>
          <TabsTrigger value="custom">Add a Translation</TabsTrigger>
        </TabsList>

        <TabsContent value="lookup" className="space-y-4">
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
                    // Back to the screen's defaults, so a book change never
                    // leaves a reference that does not exist in the new book.
                    setChapter(DEFAULT_CHAPTER);
                    setVerse(DEFAULT_VERSE);
                    setEndVerse(DEFAULT_VERSE);
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

          <Panel
            title="Search words"
            actions={
              <Badge variant={everywhere ? "success" : "muted"}>
                {everywhere ? "all Bibles" : "this Bible only"}
              </Badge>
            }
          >
            <div className="space-y-3">
              <div className="flex gap-2">
                <Input
                  value={query}
                  onChange={(e) => setQuery(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") {
                      void runSearch();
                    }
                  }}
                  placeholder="A phrase, a few words…"
                  aria-label="Search verse text"
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

              <div className="flex items-center justify-between gap-3">
                <div className="min-w-0">
                  <p className="text-xs font-medium">
                    Look in every Bible installed
                  </p>
                  <p className="text-xs text-muted-foreground">
                    {translations.length > 1
                      ? `Useful when you cannot remember which version it was. ${translations.length} are installed.`
                      : "Turn this off to search only the Bible chosen above."}
                  </p>
                </div>
                <Switch
                  checked={everywhere}
                  onCheckedChange={(on) => {
                    setEverywhere(on);
                    // Re-run straight away, so the toggle is visibly real.
                    if (searched) {
                      void runSearch(on ? null : translationId);
                    }
                  }}
                  aria-label="Look in every Bible installed"
                />
              </div>

              {searching ? (
                <p className="text-xs text-muted-foreground">Looking…</p>
              ) : null}

              {results.related && results.verses.length > 0 ? (
                <p className="rounded-md border border-warning/30 bg-warning/10 px-2.5 py-1.5 text-xs text-warning">
                  Those exact words were not found. These are the closest
                  matches — check one before showing it.
                </p>
              ) : null}
              {results.verses.length > 0 ? (
                <>
                  <Separator />
                  <ScrollArea className="h-44">
                    <ul className="space-y-1 pr-2">
                      {results.verses.map((found) => (
                        <li
                          key={`${found.translationId}-${found.bookId}-${found.chapter}-${found.verse}`}
                        >
                          <button
                            type="button"
                            className="w-full rounded-md px-2 py-1.5 text-left text-xs transition-colors hover:bg-accent"
                            onClick={() => goTo(found)}
                          >
                            <span className="flex items-center gap-2">
                              <span className="font-medium">
                                {bookNames.get(found.bookId)} {found.chapter}:
                                {found.verse}
                              </span>
                              {/* Which Bible it came from matters when the
                                  search covered every translation. */}
                              <span className="text-muted-foreground uppercase">
                                {found.translationId}
                              </span>
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

              {/*
                Nothing matched at all. Rather than leaving a dead end, the
                operator gets passages about the same subject — references
                only, because Selah ships no Bible text.
              */}
              {searched && results.verses.length === 0 ? (
                <>
                  <Separator />
                  <div className="space-y-2">
                    <p className="flex items-center gap-1.5 text-xs text-muted-foreground">
                      <Lightbulb className="size-3.5" />
                      {results.suggestions.length > 0
                        ? "Nothing matched. These passages are about the same subject:"
                        : "Nothing matched, and no similar subject was recognised. Try one or two words instead."}
                    </p>
                    <ul className="space-y-1">
                      {results.suggestions.map((suggestion) => (
                        <li key={`${suggestion.topic}-${suggestion.reference}`}>
                          <button
                            type="button"
                            className="w-full rounded-md border border-border/60 px-2 py-1.5 text-left text-xs transition-colors hover:bg-accent"
                            onClick={() => goTo(suggestion)}
                          >
                            <span className="font-medium">
                              {suggestion.topic}
                            </span>
                            <span className="block truncate text-muted-foreground">
                              {suggestion.reference} — {suggestion.why}
                            </span>
                          </button>
                        </li>
                      ))}
                    </ul>
                  </div>
                </>
              ) : null}
            </div>
          </Panel>
        </div>

        <PassagePanel
          passage={passage}
          loading={loading}
          projecting={projecting}
          hasTranslations={translations.length > 0}
          emptyReference={reference ? formatReference(reference, bookNames) : ""}
          onDisplay={() => void display()}
          fit={fit}
          headingSize={effectiveHeadingSize}
          textSize={effectiveTextSize}
          savedFontSize={savedFontSize}
          usingCustomSizes={usingCustomSizes}
          onHeadingSize={setHeadingSize}
          onTextSize={setTextSize}
          onResetSizes={() => {
            setHeadingSize(null);
            setTextSize(null);
          }}
        />
      </div>
        </TabsContent>

        <TabsContent value="custom" className="space-y-4">
          <AddTranslationPanel
            books={books}
            onError={setError}
            onSaved={(result) => {
              // Refresh the Bible list so the new translation appears, then
              // jump straight to what was just saved — the fastest way to be
              // sure it worked is to see it.
              void reloadTranslations().then(() => {
                setTranslationId(result.translationId);
                setBookId(result.bookId);
                setChapter(result.chapter);
                setVerse(1);
                setEndVerse(1);
                setNotice(
                  `${result.reference} saved — it is on screen below.`,
                );
                setTab("lookup");
              });
            }}
          />
        </TabsContent>
      </Tabs>
    </>
  );
}

/**
 * Passage preview + the size controls + the Display action.
 *
 * The sizes here are for **this passage only**: they travel with it to the
 * projector and leave the saved Settings untouched. The warning underneath is
 * an estimate of whether it will fit the chosen screen — a nudge, never a
 * block, because the operator can see the room and Selah cannot.
 */
function PassagePanel({
  passage,
  loading,
  projecting,
  hasTranslations,
  emptyReference,
  onDisplay,
  fit,
  headingSize,
  textSize,
  savedFontSize,
  usingCustomSizes,
  onHeadingSize,
  onTextSize,
  onResetSizes,
}: {
  passage: Passage | null;
  loading: boolean;
  projecting: boolean;
  hasTranslations: boolean;
  emptyReference: string;
  onDisplay: () => void;
  fit: ReturnType<typeof estimateFit>;
  headingSize: number;
  textSize: number;
  savedFontSize: number;
  usingCustomSizes: boolean;
  onHeadingSize: (value: number | null) => void;
  onTextSize: (value: number | null) => void;
  onResetSizes: () => void;
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
      <div className="space-y-4">
        <div className="grid gap-3 rounded-lg border border-border/60 p-3 sm:grid-cols-[1fr_1fr_auto]">
          <div className="space-y-1.5">
            <Label htmlFor="bible-heading-size">
              <span className="flex items-center gap-1.5">
                <Type className="size-3.5" />
                Heading size — {headingSize}px
              </span>
            </Label>
            <input
              id="bible-heading-size"
              type="range"
              min={MIN_ITEM_SIZE}
              max={MAX_ITEM_SIZE}
              step={2}
              value={headingSize}
              onChange={(event) => onHeadingSize(Number(event.target.value))}
              className="h-9 w-full cursor-pointer accent-[var(--brand)]"
            />
          </div>

          <div className="space-y-1.5">
            <Label htmlFor="bible-text-size">
              <span className="flex items-center gap-1.5">
                <Type className="size-3.5" />
                Verse size — {textSize}px
              </span>
            </Label>
            <input
              id="bible-text-size"
              type="range"
              min={MIN_ITEM_SIZE}
              max={MAX_ITEM_SIZE}
              step={2}
              value={textSize}
              onChange={(event) => onTextSize(Number(event.target.value))}
              className="h-9 w-full cursor-pointer accent-[var(--brand)]"
            />
          </div>

          <div className="flex items-end gap-2">
            <Button
              variant="ghost"
              size="sm"
              disabled={!usingCustomSizes}
              onClick={onResetSizes}
            >
              <RotateCcw className="size-3.5" />
              Saved sizes
            </Button>
          </div>

          <p className="text-xs text-muted-foreground sm:col-span-3">
            {usingCustomSizes
              ? `Only this passage — Settings still uses ${savedFontSize}px.`
              : `Following Settings, which uses ${savedFontSize}px. Move a slider to change just this passage.`}
          </p>
        </div>

        {fit?.overflows ? (
          <p className="flex items-start gap-2 rounded-md border border-warning/30 bg-warning/10 px-3 py-2 text-sm text-warning">
            <AlertTriangle className="mt-0.5 size-4 shrink-0" />
            <span>{fit.warning}</span>
          </p>
        ) : null}

        {loading ? (
          <EmptyHint>Loading…</EmptyHint>
        ) : !passage ? (
          <EmptyHint>
            {hasTranslations
              ? `Nothing is saved for ${emptyReference || "that reference"} in this Bible. Choose a different verse, or add the verses yourself on the “Add a Translation” tab.`
              : "No Bible text added yet. Add one from Settings → Bible text."}
          </EmptyHint>
        ) : passage.verses.length === 0 ? (
          <EmptyHint>
            There is no text saved for {emptyReference}. Choose a different
            verse.
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
            {fit ? (
              <p className="text-xs text-muted-foreground">
                About {fit.lines} projected line
                {fit.lines === 1 ? "" : "s"} · needs roughly{" "}
                {fit.requiredHeight}px of the {fit.availableHeight}px available.
              </p>
            ) : (
              <p className="text-xs text-muted-foreground">
                Show the screen once so Selah can measure it against your
                display.
              </p>
            )}
          </article>
        )}
      </div>
    </Panel>
  );
}
