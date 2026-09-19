import { useCallback, useEffect, useMemo, useState } from "react";
import {
  CheckCircle2,
  ClipboardCopy,
  Code2,
  Download,
  Pencil,
  Plus,
  Save,
  Search,
  ShieldCheck,
  Trash2,
  Wand2,
} from "lucide-react";

import { EmptyHint, Panel } from "@/components/PageHeader";
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
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";
import { bibleApi } from "@/lib/api";
import {
  buildAiPrompt,
  parseVerseArray,
  validateSingleVerse,
  VERSE_ARRAY_EXAMPLE,
} from "@/lib/customVerses";
import type {
  BibleBook,
  CatalogueEntry,
  CustomVerseImportResult,
  CustomVerseInput,
  TranslationStatus,
} from "@/types";

/** Which of the two verse-entry styles is open. */
type Mode = "single" | "chapter";

/**
 * The "Add a Translation" screen.
 *
 * Selah ships three public-domain Bibles (WEB, KJV, ASV) and no other Bible
 * text at all. This is where every other translation comes from, in three
 * parts:
 *
 *   1. **published translations** — MSG, NIV, ESV and the rest, listed so the
 *      operator can add one in a press instead of typing a name and hoping it
 *      matches something in the picker;
 *   2. **a translation of your own** — for anything Selah does not list;
 *   3. **the verses themselves** — a translation with no verses is an empty
 *      shell, so this is where its words are typed, or pasted from an AI reply
 *      in the format Selah checks before storing anything.
 *
 * The three bundled Bibles are marked as Selah's own and cannot be renamed or
 * removed; everything added here can be edited and deleted again.
 */
export default function AddTranslationPanel({
  books,
  onSaved,
  onError,
}: {
  books: BibleBook[];
  /** Called after verses are stored, so the parent can refresh its list. */
  onSaved: (result: CustomVerseImportResult) => void;
  onError: (message: string | null) => void;
}) {
  const [catalogue, setCatalogue] = useState<CatalogueEntry[]>([]);
  const [installed, setInstalled] = useState<TranslationStatus[]>([]);
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [query, setQuery] = useState("");

  // The translation verses are stored under, and the verse-entry mode.
  const [translationId, setTranslationId] = useState("");
  const [mode, setMode] = useState<Mode>("single");
  const [book, setBook] = useState("");
  const [chapter, setChapter] = useState(1);
  const [verse, setVerse] = useState(3);
  const [singleText, setSingleText] = useState("");
  const [pasted, setPasted] = useState("");
  const [copied, setCopied] = useState(false);
  const [saved, setSaved] = useState<CustomVerseImportResult | null>(null);

  const reload = useCallback(async () => {
    try {
      const [entries, translations] = await Promise.all([
        bibleApi.listTranslationCatalogue(),
        bibleApi.listBibleTranslations(),
      ]);
      setCatalogue(entries);
      setInstalled(translations);

      // Verses are only ever written into a translation the operator added:
      // Selah's own three are read-only.
      const writable = translations.filter((t) => !t.translation.builtin);
      setTranslationId((current) =>
        writable.some((t) => t.translation.id === current)
          ? current
          : (writable[0]?.translation.id ?? ""),
      );
    } catch (e: unknown) {
      setCatalogue([]);
      setInstalled([]);
      onError(e instanceof Error ? e.message : String(e));
    }
  }, [onError]);

  useEffect(() => {
    void reload();
  }, [reload]);

  const selectedBook = books.find((entry) => entry.name === book);
  const chapterCount = Math.max(selectedBook?.chapters ?? 1, 1);
  const writableTranslations = installed.filter((t) => !t.translation.builtin);
  const target = installed.find((t) => t.translation.id === translationId);

  const single = useMemo(() => validateSingleVerse(singleText), [singleText]);
  const parsed = useMemo(
    () => (pasted.trim().length === 0 ? null : parseVerseArray(pasted)),
    [pasted],
  );

  const prompt = buildAiPrompt(book, chapter, target?.translation.name);

  /** Adds a translation, whether published or one of the operator's own. */
  async function add(request: {
    id: string;
    name?: string;
    abbreviation?: string;
  }) {
    setBusy(true);
    try {
      const added = await bibleApi.addTranslation(request);
      await reload();
      setTranslationId(added.translation.id);
      setNotice(
        `${added.translation.name} added. Add its verses below — a translation with no verses shows nothing when you look it up.`,
      );
      setSaved(null);
      onError(null);
    } catch (e: unknown) {
      setNotice(null);
      onError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function remove(status: TranslationStatus) {
    setBusy(true);
    try {
      await bibleApi.deleteTranslation(status.translation.id);
      await reload();
      setNotice(
        `${status.translation.name} removed, along with its ${status.verseCount} verse${
          status.verseCount === 1 ? "" : "s"
        }.`,
      );
      setSaved(null);
      onError(null);
    } catch (e: unknown) {
      setNotice(null);
      onError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function copyPrompt() {
    try {
      await navigator.clipboard.writeText(prompt);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 2500);
    } catch {
      // A webview can refuse the clipboard. The prompt is on screen to copy by
      // hand, so this is only a convenience — say so rather than failing.
      onError(
        "Selah could not reach the clipboard. Select the prompt below and copy it by hand.",
      );
    }
  }

  /** Sends validated verses to the backend and reports the outcome. */
  async function persist(verses: CustomVerseInput[]) {
    if (!target) {
      onError("Add a translation first, then its verses can be stored.");
      return;
    }
    setBusy(true);
    try {
      const result = await bibleApi.saveCustomVerses({
        translationId: target.translation.id,
        verses,
      });
      setSaved(result);
      setSingleText("");
      setPasted("");
      setNotice(null);
      onError(null);
      await reload();
      onSaved(result);
    } catch (e: unknown) {
      setSaved(null);
      onError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function saveSingle() {
    if (!selectedBook || !single.ok) {
      return;
    }
    await persist([
      { book: selectedBook.name, chapter, verse, text: single.text },
    ]);
  }

  async function saveChapter() {
    if (!parsed?.ok) {
      return;
    }
    await persist(parsed.value.verses);
  }

  const filtered = useMemo(() => {
    const needle = query.trim().toLowerCase();
    if (!needle) {
      return catalogue;
    }
    return catalogue.filter(
      (entry) =>
        entry.name.toLowerCase().includes(needle) ||
        entry.abbreviation.toLowerCase().includes(needle) ||
        entry.id.includes(needle),
    );
  }, [catalogue, query]);

  /** The catalogue grouped for display, with the operator's own first. */
  const groups = useMemo(() => {
    const order = [
      "Selah's own",
      "Added by you",
      "Classic",
      "Modern",
      "Everyday",
    ];
    const byGroup = new Map<string, CatalogueEntry[]>();
    for (const entry of filtered) {
      byGroup.set(entry.group, [...(byGroup.get(entry.group) ?? []), entry]);
    }
    return [...byGroup.entries()].sort((a, b) => {
      const left = order.indexOf(a[0]);
      const right = order.indexOf(b[0]);
      return (left < 0 ? order.length : left) - (right < 0 ? order.length : right);
    });
  }, [filtered]);

  return (
    <div className="grid gap-4 lg:grid-cols-[24rem_1fr]">
      <div className="flex flex-col gap-4">
        <Panel
          title="Add a published translation"
          actions={<Badge variant="muted">{installed.length} here now</Badge>}
        >
          <div className="space-y-3">
            <p className="text-xs text-muted-foreground">
              These are published Bibles, listed so the name always matches.
              Selah does not download Bible text and does not ship any — adding
              one here creates it, ready for verses you add on the right (from
              a file you own, or pasted using the prompt Selah writes).
            </p>

            <div className="relative">
              <Search className="absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2 text-muted-foreground" />
              <Input
                value={query}
                onChange={(event) => setQuery(event.target.value)}
                placeholder="Search translations"
                className="pl-8"
                aria-label="Search translations"
              />
            </div>

            <ScrollArea className="max-h-96">
              <div className="space-y-3 pr-2">
                {groups.length === 0 ? (
                  <EmptyHint>No translation matches that search.</EmptyHint>
                ) : null}

                {groups.map(([group, entries]) => (
                  <div key={group} className="space-y-1">
                    <p className="text-xs font-medium tracking-wide text-muted-foreground uppercase">
                      {group}
                    </p>
                    <ul className="space-y-1">
                      {entries.map((entry) => (
                        <li
                          key={entry.id}
                          className="flex items-center gap-2 rounded-md border border-border/60 px-2.5 py-1.5"
                        >
                          <span className="min-w-0 flex-1">
                            <span className="flex items-center gap-1.5">
                              <span className="truncate text-sm">
                                {entry.name}
                              </span>
                              <span className="shrink-0 text-xs text-muted-foreground">
                                {entry.abbreviation}
                              </span>
                              {entry.builtin ? (
                                <ShieldCheck
                                  className="size-3.5 shrink-0 text-muted-foreground"
                                  aria-label="Comes with Selah"
                                />
                              ) : null}
                            </span>
                            <span className="block truncate text-xs text-muted-foreground">
                              {entry.installed
                                ? entry.verseCount > 0
                                  ? `${entry.verseCount} verses stored`
                                  : "added, no verses yet"
                                : entry.publicDomain
                                  ? "public domain"
                                  : "published translation"}
                            </span>
                          </span>

                          {entry.installed ? (
                            <Badge variant="success">added</Badge>
                          ) : (
                            <Button
                              variant="outline"
                              size="sm"
                              disabled={busy || entry.builtin}
                              onClick={() => void add({ id: entry.id })}
                            >
                              <Plus className="size-3.5" />
                              Add
                            </Button>
                          )}
                        </li>
                      ))}
                    </ul>
                  </div>
                ))}
              </div>
            </ScrollArea>
          </div>
        </Panel>

        <OwnTranslationForm busy={busy} onAdd={add} />

        <Panel
          title="Edit or remove a translation"
          actions={
            <Badge variant="muted">
              {writableTranslations.length} editable
            </Badge>
          }
        >
          <div className="space-y-3">
            <p className="text-xs text-muted-foreground">
              The three Bibles that come with Selah — WEB, KJV and ASV — are
              always here and cannot be changed. Anything you added can be
              renamed, or removed along with its verses.
            </p>

            {writableTranslations.length === 0 ? (
              <EmptyHint>
                Nothing added yet. Add a published translation above, or one of
                your own.
              </EmptyHint>
            ) : (
              <ul className="space-y-1">
                {writableTranslations.map((status) => (
                  <li key={status.translation.id}>
                    <TranslationRow
                      status={status}
                      busy={busy}
                      onRename={async (name, abbreviation) => {
                        setBusy(true);
                        try {
                          await bibleApi.updateTranslation({
                            id: status.translation.id,
                            name,
                            abbreviation,
                          });
                          await reload();
                          setNotice(`${name} updated.`);
                          onError(null);
                        } catch (e: unknown) {
                          setNotice(null);
                          onError(e instanceof Error ? e.message : String(e));
                        } finally {
                          setBusy(false);
                        }
                      }}
                      onRemove={() => void remove(status)}
                    />
                  </li>
                ))}
              </ul>
            )}
          </div>
        </Panel>
      </div>

      <div className="flex flex-col gap-4">
        {notice ? (
          <p className="rounded-md border border-success/30 bg-success/10 px-3 py-2 text-sm text-success">
            {notice}

        <Panel
          title="Verses to store"
          actions={
            target ? (
              <Badge variant="muted">{target.translation.name}</Badge>
            ) : (
              <Badge variant="warning">no translation yet</Badge>
            )
          }
        >
          <div className="space-y-3">
            {writableTranslations.length === 0 ? (
              <p className="rounded-md border border-warning/30 bg-warning/10 px-2.5 py-1.5 text-xs text-warning">
                Add a translation first — Selah will not write into the Bibles
                that come with it.
              </p>
            ) : (
              <div className="space-y-1.5">
                <Label htmlFor="verse-translation">Which translation</Label>
                <Select value={translationId} onValueChange={setTranslationId}>
                  <SelectTrigger id="verse-translation">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {writableTranslations.map((status) => (
                      <SelectItem
                        key={status.translation.id}
                        value={status.translation.id}
                      >
                        {status.translation.name} · {status.verseCount} verses
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            )}

            <div className="grid gap-3 sm:grid-cols-2">
              <SwitchRow
                label="Enter one verse"
                hint="The words of a single verse, typed by hand."
                checked={mode === "single"}
                onChecked={() => setMode("single")}
              />
              <SwitchRow
                label="Paste a whole chapter"
                hint="An array of verses, one object per verse."
                checked={mode === "chapter"}
                onChecked={() => setMode("chapter")}
              />
            </div>

            <div className="grid grid-cols-3 gap-2">
              <div className="col-span-3 space-y-1.5">
                <Label htmlFor="verse-book">Book</Label>
                <Select value={book} onValueChange={setBook}>
                  <SelectTrigger id="verse-book">
                    <SelectValue placeholder="Choose a book" />
                  </SelectTrigger>
                  <SelectContent>
                    {books.map((entry) => (
                      <SelectItem key={entry.id} value={entry.name}>
                        {entry.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-1.5">
                <Label htmlFor="verse-chapter">Chapter</Label>
                <Input
                  id="verse-chapter"
                  type="number"
                  min={1}
                  max={chapterCount}
                  value={chapter}
                  onChange={(event) =>
                    setChapter(Math.max(1, Number(event.target.value) || 1))
                  }
                />
              </div>
              <div className="space-y-1.5">
                <Label htmlFor="verse-number">Verse</Label>
                <Input
                  id="verse-number"
                  type="number"
                  min={1}
                  value={verse}
                  disabled={mode === "chapter"}
                  onChange={(event) =>
                    setVerse(Math.max(1, Number(event.target.value) || 1))
                  }
                />
              </div>
              <div className="flex items-end">
                <p className="text-xs text-muted-foreground">
                  {selectedBook ? `${chapterCount} chapters` : "pick a book"}
                </p>
              </div>
            </div>
          </div>
        </Panel>

        {saved ? (
          <Panel title="Saved">
            <div className="space-y-1.5">
              <p className="flex items-center gap-2 text-sm font-medium text-success">
                <CheckCircle2 className="size-4" />
                {saved.reference}
              </p>
              <p className="text-xs text-muted-foreground">
                {saved.versesSaved} verse
                {saved.versesSaved === 1 ? "" : "s"} added. Look them up on the
                Look up tab — the translation appears in the Bible list.
              </p>
              {saved.missingVerses.length > 0 ? (
                <p className="rounded-md border border-warning/30 bg-warning/10 px-2.5 py-1.5 text-xs text-warning">
                  Verse{saved.missingVerses.length === 1 ? "" : "s"}{" "}
                  {saved.missingVerses.join(", ")} were not included. That is
                  fine — the rest is saved.
                </p>
              ) : null}
            </div>
          </Panel>
        ) : null}

          </p>
        ) : null}


        {mode === "single" ? (
          <SingleVerseEditor
            book={book}
            chapter={chapter}
            verse={verse}
            text={singleText}
            onText={setSingleText}
            verdict={single}
            busy={busy || !selectedBook || !target}
            onSave={() => void saveSingle()}
          />
        ) : (
          <ChapterPasteEditor
            prompt={prompt}
            copied={copied}
            onCopy={() => void copyPrompt()}
            pasted={pasted}
            onPasted={setPasted}
            parsed={parsed}
            busy={busy || !target}
            onSave={() => void saveChapter()}
          />
        )}
      </div>
    </div>
  );
}

/**
 * Adding a translation Selah does not list.
 *
 * The id is what the verses are filed under, so it is sanitised the same way on
 * both sides of the bridge.
 */
function OwnTranslationForm({
  busy,
  onAdd,
}: {
  busy: boolean;
  onAdd: (request: {
    id: string;
    name?: string;
    abbreviation?: string;
  }) => Promise<void>;
}) {
  const [id, setId] = useState("");
  const [name, setName] = useState("");
  const [abbreviation, setAbbreviation] = useState("");

  const ready = id.trim().length > 0 && name.trim().length > 0;

  async function submit() {
    if (!ready) {
      return;
    }
    await onAdd({ id, name, abbreviation });
    setId("");
    setName("");
    setAbbreviation("");
  }

  return (
    <Panel title="A translation Selah does not list">
      <div className="space-y-3">
        <p className="text-xs text-muted-foreground">
          For anything else — a study Bible, a local-language version, your own
          printed edition.
        </p>

        <div className="grid gap-3 sm:grid-cols-2">
          <div className="space-y-1.5">
            <Label htmlFor="new-translation-id">Short id</Label>
            <Input
              id="new-translation-id"
              value={id}
              onChange={(event) => setId(event.target.value)}
              placeholder="my-bible"
            />
          </div>
          <div className="space-y-1.5">
            <Label htmlFor="new-translation-name">Name</Label>
            <Input
              id="new-translation-name"
              value={name}
              onChange={(event) => setName(event.target.value)}
              placeholder="The church study Bible"
            />
          </div>
          <div className="space-y-1.5">
            <Label htmlFor="new-translation-abbr">Short name (optional)</Label>
            <Input
              id="new-translation-abbr"
              value={abbreviation}
              onChange={(event) => setAbbreviation(event.target.value)}
              placeholder="CSB"
            />
          </div>
        </div>

        <Button
          variant="success"
          size="sm"
          disabled={busy || !ready}
          onClick={() => void submit()}
        >
          <Download className="size-3.5" />
          Add this translation
        </Button>
      </div>
    </Panel>
  );
}

/** One installed translation, with inline rename and removal. */
function TranslationRow({
  status,
  busy,
  onRename,
  onRemove,
}: {
  status: TranslationStatus;
  busy: boolean;
  onRename: (name: string, abbreviation: string) => Promise<void>;
  onRemove: () => void;
}) {
  const [editing, setEditing] = useState(false);
  const [name, setName] = useState(status.translation.name);
  const [abbreviation, setAbbreviation] = useState(
    status.translation.abbreviation ?? "",
  );
  const [confirming, setConfirming] = useState(false);

  if (editing) {
    return (
      <div className="space-y-2 rounded-md border border-border/60 px-2.5 py-2">
        <div className="grid gap-2 sm:grid-cols-2">
          <div className="space-y-1.5">
            <Label htmlFor={`rename-${status.translation.id}`}>Name</Label>
            <Input
              id={`rename-${status.translation.id}`}
              value={name}
              onChange={(event) => setName(event.target.value)}
            />
          </div>
          <div className="space-y-1.5">
            <Label htmlFor={`rename-abbr-${status.translation.id}`}>
              Short name
            </Label>
            <Input
              id={`rename-abbr-${status.translation.id}`}
              value={abbreviation}
              onChange={(event) => setAbbreviation(event.target.value)}
            />
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Button
            size="sm"
            disabled={busy || name.trim().length === 0}
            onClick={() => {
              void onRename(name.trim(), abbreviation.trim()).then(() =>
                setEditing(false),
              );
            }}
          >
            <Save className="size-3.5" />
            Save name
          </Button>
          <Button
            variant="ghost"
            size="sm"
            disabled={busy}
            onClick={() => {
              setName(status.translation.name);
              setAbbreviation(status.translation.abbreviation ?? "");
              setEditing(false);
            }}
          >
            Cancel
          </Button>
        </div>
      </div>
    );
  }

  return (
    <div className="flex items-center gap-2 rounded-md border border-border/60 px-2.5 py-2">
      <span className="min-w-0 flex-1">
        <span className="block truncate text-sm">
          {status.translation.name}
        </span>
        <span className="block truncate text-xs text-muted-foreground">
          {status.translation.abbreviation ?? status.translation.id} ·{" "}
          {status.verseCount} verse{status.verseCount === 1 ? "" : "s"} ·{" "}
          {status.translation.origin === "catalogue"
            ? "from the published list"
            : "added by you"}
        </span>
      </span>

      {confirming ? (
        <div className="flex items-center gap-1">
          <span className="text-xs text-destructive">
            Remove {status.verseCount} verse
            {status.verseCount === 1 ? "" : "s"}?
          </span>
          <Button
            variant="destructive"
            size="sm"
            disabled={busy}
            onClick={onRemove}
          >
            Remove
          </Button>
          <Button
            variant="ghost"
            size="sm"
            disabled={busy}
            onClick={() => setConfirming(false)}
          >
            Keep
          </Button>
        </div>
      ) : (
        <div className="flex items-center gap-1">
          <Button
            variant="ghost"
            size="icon"
            aria-label={`Rename ${status.translation.name}`}
            disabled={busy}
            onClick={() => setEditing(true)}
          >
            <Pencil className="size-4" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            aria-label={`Remove ${status.translation.name}`}
            disabled={busy}
            onClick={() => setConfirming(true)}
          >
            <Trash2 className="size-4" />
          </Button>
        </div>
      )}
    </div>
  );
}

/**
 * A labelled switch.
 *
 * The pair of switches are answers to one question, so the one in use cannot be
 * switched off: turning it off would only pick the other, which is not what a
 * switch means.
 */
function SwitchRow({
  label,
  hint,
  checked,
  onChecked,
}: {
  label: string;
  hint: string;
  checked: boolean;
  onChecked: () => void;
}) {
  return (
    <div className="flex items-center justify-between gap-3 rounded-lg border border-border/60 px-3 py-2">
      <div className="min-w-0">
        <p className="text-sm font-medium">{label}</p>
        <p className="text-xs text-muted-foreground">{hint}</p>
      </div>
      <Switch
        checked={checked}
        onCheckedChange={(on) => {
          if (on) {
            onChecked();
          }
        }}
        aria-label={label}
      />
    </div>
  );
}

/**
 * The single-verse editor.
 *
 * Everything about this box says one thing: only the verse message goes here.
 * The checks run as the operator types, so the mistake is caught while it is
 * still obvious rather than after pressing Save.
 */
function SingleVerseEditor({
  book,
  chapter,
  verse,
  text,
  onText,
  verdict,
  busy,
  onSave,
}: {
  book: string;
  chapter: number;
  verse: number;
  text: string;
  onText: (next: string) => void;
  verdict: { ok: boolean; text: string; error: string | null };
  busy: boolean;
  onSave: () => void;
}) {
  const empty = text.trim().length === 0;

  return (
    <Panel
      title="One verse"
      actions={
        <Badge variant="muted">
          {book ? `${book} ${chapter}:${verse}` : "no book chosen"}
        </Badge>
      }
    >
      <div className="space-y-3">
        <div className="space-y-1.5">
          <Label htmlFor="single-verse-text">The verse message</Label>
          <Textarea
            id="single-verse-text"
            value={text}
            onChange={(event) => onText(event.target.value)}
            placeholder="Jesus wept."
            aria-describedby="single-verse-help"
          />
          <p id="single-verse-help" className="text-xs text-muted-foreground">
            Only the words of the verse — nothing else. No book name, no chapter
            and verse number, no quotation marks. For example:{" "}
            <span className="text-foreground">Jesus wept.</span>
          </p>
        </div>

        {!empty && verdict.error ? (
          <p className="rounded-md border border-destructive/30 bg-destructive/10 px-2.5 py-1.5 text-xs text-destructive">
            {verdict.error}
          </p>
        ) : null}

        <div className="flex flex-wrap items-center gap-3">
          <Button
            variant="success"
            disabled={busy || !verdict.ok}
            onClick={onSave}
          >
            <Save className="size-4" />
            Save this verse
          </Button>
          <p className="text-xs text-muted-foreground">
            Stored as {book || "the chosen book"} {chapter}:{verse} in the
            translation chosen above.
          </p>
        </div>
      </div>
    </Panel>
  );
}


/**
 * The whole-chapter editor.
 *
 * The prompt is the clever part: rather than hoping an operator knows how to
 * phrase the request, Selah writes it for them — naming both the chapter and
 * the translation, because the words differ between versions. The example array
 * is on screen both as the format to expect and as something to compare a reply
 * against.
 */
function ChapterPasteEditor({
  prompt,
  copied,
  onCopy,
  pasted,
  onPasted,
  parsed,
  busy,
  onSave,
}: {
  prompt: string;
  copied: boolean;
  onCopy: () => void;
  pasted: string;
  onPasted: (next: string) => void;
  parsed: { ok: boolean; error?: string } | null;
  busy: boolean;
  onSave: () => void;
}) {
  const ready = parsed?.ok === true;
  const verseCount = ready ? countVerses(pasted) : 0;

  return (
    <>
      <Panel
        title="Ask an AI for the chapter"
        actions={
          <Button variant="outline" size="sm" onClick={onCopy}>
            <ClipboardCopy className="size-3.5" />
            {copied ? "Copied" : "Copy the prompt"}
          </Button>
        }
      >
        <div className="space-y-3">
          <p className="text-xs text-muted-foreground">
            Copy this and give it to an AI chat, then paste the reply into the
            box below. It names the chapter <em>and</em> the translation, so the
            words come back in the version you chose.
          </p>
          <pre className="max-h-64 overflow-auto rounded-md border border-border/60 bg-muted/40 p-3 font-mono text-xs whitespace-pre-wrap">
            {prompt}
          </pre>
        </div>
      </Panel>

      <Panel
        title="Paste the reply"
        actions={
          <Badge variant={ready ? "success" : parsed ? "destructive" : "muted"}>
            {ready
              ? `${verseCount} verse${verseCount === 1 ? "" : "s"} ready`
              : parsed
                ? "needs fixing"
                : "waiting"}
          </Badge>
        }
      >
        <div className="space-y-3">
          <div className="space-y-1.5">
            <Label htmlFor="chapter-array">Verses as an array</Label>
            <Textarea
              id="chapter-array"
              value={pasted}
              onChange={(event) => onPasted(event.target.value)}
              placeholder={VERSE_ARRAY_EXAMPLE}
              className="min-h-40 font-mono text-xs"
              spellCheck={false}
            />
          </div>

          {parsed && !parsed.ok ? (
            <p className="rounded-md border border-destructive/30 bg-destructive/10 px-2.5 py-1.5 text-xs text-destructive">
              {parsed.error}
            </p>
          ) : null}

          {ready ? (
            <p className="flex items-center gap-2 text-xs text-success">
              <CheckCircle2 className="size-3.5" />
              Every verse checked out: one book, one chapter, no duplicates.
            </p>
          ) : null}

          <details className="rounded-md border border-border/60 p-3">
            <summary className="flex cursor-pointer items-center gap-2 text-xs font-medium">
              <Code2 className="size-3.5" />
              The exact format expected
            </summary>
            <pre className="mt-2 overflow-auto font-mono text-xs whitespace-pre-wrap">
              {VERSE_ARRAY_EXAMPLE}
            </pre>
            <ul className="mt-2 list-disc space-y-1 pl-4 text-xs text-muted-foreground">
              <li>One object per verse, in one array.</li>
              <li>
                Keys are exactly <code>book</code>, <code>chapter</code>,{" "}
                <code>verse</code>, <code>text</code>.
              </li>
              <li>
                <code>text</code> holds the verse words only — no verse number
                inside it.
              </li>
              <li>
                One book and one chapter per paste; Selah checks this before
                saving anything.
              </li>
            </ul>
          </details>

          <div className="flex flex-wrap items-center gap-3">
            <Button variant="success" disabled={busy || !ready} onClick={onSave}>
              <Wand2 className="size-4" />
              Check and save the chapter
            </Button>
            <p className="text-xs text-muted-foreground">
              Nothing is saved until every verse passes the checks.
            </p>
          </div>
        </div>
      </Panel>
    </>
  );
}

/** Number of objects in a parsed array, for the "ready" badge. */
function countVerses(pasted: string): number {
  const parsed = parseVerseArray(pasted);
  return parsed.ok ? parsed.value.verses.length : 0;
}
