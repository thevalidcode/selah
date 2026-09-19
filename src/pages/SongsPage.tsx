import { useCallback, useEffect, useMemo, useState } from "react";
import {
  ChevronLeft,
  ChevronRight,
  MonitorPlay,
  Music,
  Plus,
  Save,
  Search,
  Trash2,
} from "lucide-react";

import PageHeader, { EmptyHint, Panel } from "@/components/PageHeader";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { Textarea } from "@/components/ui/textarea";
import { presentationApi, songsApi } from "@/lib/api";
import { EVENTS, useTauriEvent } from "@/lib/events";
import type { PresentationItem, Song, SongInput } from "@/types";

/** A song being edited. Sections carry no id: the store assigns those. */
type SongDraft = {
  /** Absent while the song is new. */
  id?: string;
  title: string;
  author: string;
  sections: { label: string; text: string }[];
};

function emptyDraft(): SongDraft {
  return {
    title: "",
    author: "",
    sections: [{ label: "Verse 1", text: "" }],
  };
}

function draftFromSong(song: Song): SongDraft {
  return {
    id: song.id,
    title: song.title,
    author: song.author ?? "",
    sections:
      song.sections && song.sections.length > 0
        ? song.sections.map((section) => ({
            label: section.label ?? "",
            text: section.text,
          }))
        : [{ label: "Verse 1", text: "" }],
  };
}

/**
 * Songs library.
 *
 * A song is stored as ordered sections (verse, chorus, bridge…) and presented
 * one section at a time, exactly the way Bible verses are presented — so a long
 * song never has to be squeezed onto one screen. Next/Back step through it on
 * the projector.
 */
export default function SongsPage() {
  const [songs, setSongs] = useState<Song[]>([]);
  const [selectedId, setSelectedId] = useState<string | undefined>();
  const [draft, setDraft] = useState<SongDraft>(emptyDraft);
  const [query, setQuery] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [onScreen, setOnScreen] = useState<PresentationItem | null>(null);

  const reload = useCallback(async () => {
    try {
      const list = await songsApi.listSongs();
      setSongs(list);
      return list;
    } catch (e: unknown) {
      setSongs([]);
      setError(e instanceof Error ? e.message : String(e));
      return [];
    }
  }, []);

  useEffect(() => {
    void reload();
  }, [reload]);

  useTauriEvent(EVENTS.presentationChanged, (payload) => {
    setOnScreen(payload.item ?? null);
  });

  /** Loads a song into the editor, with its sections. */
  const openSong = useCallback(async (id: string) => {
    setSelectedId(id);
    setBusy(true);
    try {
      const song = await songsApi.getSong(id);
      if (song) {
        setDraft(draftFromSong(song));
      }
      setError(null);
      setNotice(null);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }, []);

  const newSong = useCallback(() => {
    setSelectedId(undefined);
    setDraft(emptyDraft());
    setNotice(null);
    setError(null);
  }, []);

  function updateSection(index: number, patch: { label?: string; text?: string }) {
    setDraft((current) => ({
      ...current,
      sections: current.sections.map((section, position) =>
        position === index ? { ...section, ...patch } : section,
      ),
    }));
  }

  function addSection() {
    setDraft((current) => ({
      ...current,
      sections: [
        ...current.sections,
        { label: `Verse ${current.sections.length + 1}`, text: "" },
      ],
    }));
  }

  function removeSection(index: number) {
    setDraft((current) => ({
      ...current,
      sections: current.sections.filter((_, position) => position !== index),
    }));
  }

  async function save() {
    const input: SongInput = {
      title: draft.title,
      author: draft.author || undefined,
      sections: draft.sections.map((section) => ({
        label: section.label || undefined,
        text: section.text,
      })),
    };

    setBusy(true);
    try {
      const saved = await songsApi.saveSong(input, draft.id);
      await reload();
      setSelectedId(saved.id);
      setDraft(draftFromSong(saved));
      setError(null);
      setNotice(
        `“${saved.title}” saved with ${saved.sections?.length ?? 0} sections.`,
      );
    } catch (e: unknown) {
      setNotice(null);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function remove(id: string) {
    setBusy(true);
    try {
      await songsApi.deleteSong(id);
      if (selectedId === id) {
        newSong();
      }
      await reload();
      setError(null);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  /** Shows the song from `section` (1-based) and queues the rest. */
  const show = useCallback(async (songId: string, section?: number) => {
    setBusy(true);
    try {
      await songsApi.projectSong(songId, section);
      setError(null);
      setNotice(
        section && section > 1
          ? `Showing from section ${section} — use Next for the rest.`
          : "Showing this song — use Next for the following sections.",
      );
    } catch (e: unknown) {
      setNotice(null);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }, []);

  const step = useCallback(async (direction: "next" | "previous") => {
    setBusy(true);
    try {
      await (direction === "next"
        ? presentationApi.showNextItem()
        : presentationApi.showPreviousItem());
      setError(null);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }, []);

  const filtered = useMemo(() => {
    const needle = query.trim().toLowerCase();
    if (!needle) {
      return songs;
    }
    return songs.filter(
      (song) =>
        song.title.toLowerCase().includes(needle) ||
        (song.author ?? "").toLowerCase().includes(needle),
    );
  }, [songs, query]);

  const canSave =
    draft.title.trim().length > 0 &&
    draft.sections.some((section) => section.text.trim().length > 0);

  return (
    <>
      <PageHeader
        title="Songs"
        subtitle="Words you sing, split into verses so you can step through them one at a time"
        actions={<Badge variant="muted">{songs.length} songs</Badge>}
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

      <div className="grid gap-4 lg:grid-cols-[19rem_1fr]">
        <div className="flex flex-col gap-4">
          <Panel
            title="New song"
            actions={
              <Button
                variant="outline"
                size="sm"
                onClick={newSong}
                disabled={busy}
              >
                <Plus className="size-3.5" />
                Blank
              </Button>
            }
          >
            <p className="text-xs text-muted-foreground">
              Start a blank song, or pick one below to edit it.
            </p>
          </Panel>

          <Panel title="Your songs">
            <div className="space-y-3">
              <div className="relative">
                <Search className="absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2 text-muted-foreground" />
                <Input
                  value={query}
                  onChange={(e) => setQuery(e.target.value)}
                  placeholder="Search songs"
                  className="pl-8"
                  aria-label="Search songs"
                />
              </div>

              {filtered.length === 0 ? (
                <EmptyHint>
                  {songs.length === 0
                    ? "Nothing saved yet. Write a song and its verses on the right."
                    : "No song matches that search."}
                </EmptyHint>
              ) : (
                <ScrollArea className="max-h-[26rem]">
                  <ul className="space-y-1 pr-2">
                    {filtered.map((song) => (
                      <li key={song.id} className="flex items-center gap-1">
                        <button
                          type="button"
                          onClick={() => void openSong(song.id)}
                          className={`flex min-w-0 flex-1 items-center gap-2 rounded-md px-2.5 py-2 text-left text-sm transition-colors hover:bg-accent ${
                            selectedId === song.id ? "bg-accent" : ""
                          }`}
                        >
                          <Music className="size-4 shrink-0 text-muted-foreground" />
                          <span className="min-w-0">
                            <span className="block truncate font-medium">
                              {song.title}
                            </span>
                            {song.author ? (
                              <span className="block truncate text-xs text-muted-foreground">
                                {song.author}
                              </span>
                            ) : null}
                          </span>
                        </button>
                        <Button
                          variant="ghost"
                          size="icon"
                          aria-label={`Delete ${song.title}`}
                          disabled={busy}
                          onClick={() => void remove(song.id)}
                        >
                          <Trash2 className="size-4" />
                        </Button>
                      </li>
                    ))}
                  </ul>
                </ScrollArea>
              )}
            </div>
          </Panel>
        </div>

        <div className="flex flex-col gap-4">
          <SongEditor
            draft={draft}
            busy={busy}
            canSave={canSave}
            onDraft={setDraft}
            onSection={updateSection}
            onAddSection={addSection}
            onRemoveSection={removeSection}
            onSave={() => void save()}
            onShow={(section) => {
              if (draft.id) {
                void show(draft.id, section);
              }
            }}
            onDelete={() => {
              if (draft.id) {
                void remove(draft.id);
              }
            }}
          />

          <NowShowing
            item={onScreen}
            busy={busy}
            onPrevious={() => void step("previous")}
            onNext={() => void step("next")}
          />
        </div>
      </div>
    </>
  );
}

/** Editor for one song: title, author and its ordered sections. */
function SongEditor({
  draft,
  busy,
  canSave,
  onDraft,
  onSection,
  onAddSection,
  onRemoveSection,
  onSave,
  onShow,
  onDelete,
}: {
  draft: SongDraft;
  busy: boolean;
  canSave: boolean;
  onDraft: (next: SongDraft) => void;
  onSection: (index: number, patch: { label?: string; text?: string }) => void;
  onAddSection: () => void;
  onRemoveSection: (index: number) => void;
  onSave: () => void;
  onShow: (section?: number) => void;
  onDelete: () => void;
}) {
  const filled = draft.sections.filter(
    (section) => section.text.trim().length > 0,
  ).length;

  return (
    <Panel
      title={draft.id ? draft.title || "Untitled song" : "New song"}
      actions={
        <div className="flex items-center gap-1">
          <Button
            variant="success"
            size="sm"
            disabled={busy || !canSave || !draft.id}
            onClick={() => onShow(1)}
          >
            <MonitorPlay className="size-3.5" />
            Show on screen
          </Button>
          <Button
            variant="default"
            size="sm"
            disabled={busy || !canSave}
            onClick={onSave}
          >
            <Save className="size-3.5" />
            Save
          </Button>
          {draft.id ? (
            <Button variant="ghost" size="sm" disabled={busy} onClick={onDelete}>
              <Trash2 className="size-3.5" />
              Delete
            </Button>
          ) : null}
        </div>
      }
    >
      <div className="space-y-4">
        <div className="grid gap-3 sm:grid-cols-2">
          <div className="space-y-1.5">
            <Label htmlFor="song-title">Song title</Label>
            <Input
              id="song-title"
              value={draft.title}
              onChange={(e) => onDraft({ ...draft, title: e.target.value })}
              placeholder="Amazing Grace"
            />
          </div>
          <div className="space-y-1.5">
            <Label htmlFor="song-author">Written by (optional)</Label>
            <Input
              id="song-author"
              value={draft.author}
              onChange={(e) => onDraft({ ...draft, author: e.target.value })}
              placeholder="John Newton"
            />
          </div>
        </div>

        <p className="text-xs text-muted-foreground">
          {filled} section{filled === 1 ? "" : "s"} with words. Each section is a
          separate screen on the projector.
        </p>

        <Separator />

        <ul className="space-y-3">
          {draft.sections.map((section, index) => (
            <li key={index} className="rounded-lg border border-border/60 p-3">
              <div className="mb-2 flex items-center gap-2">
                <Badge variant="muted">{index + 1}</Badge>
                <Input
                  value={section.label}
                  onChange={(e) => onSection(index, { label: e.target.value })}
                  placeholder="Verse 1 / Chorus / Bridge"
                  aria-label={`Label for section ${index + 1}`}
                  className="h-8 max-w-56"
                />
                <div className="ml-auto flex items-center gap-1">
                  <Button
                    variant="outline"
                    size="sm"
                    disabled={busy || !draft.id}
                    onClick={() => onShow(index + 1)}
                  >
                    <MonitorPlay className="size-3.5" />
                    Show
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    aria-label={`Remove section ${index + 1}`}
                    disabled={busy || draft.sections.length === 1}
                    onClick={() => onRemoveSection(index)}
                  >
                    <Trash2 className="size-4" />
                  </Button>
                </div>
              </div>
              <Textarea
                value={section.text}
                onChange={(e) => onSection(index, { text: e.target.value })}
                placeholder="Words for this section"
                aria-label={`Words for section ${index + 1}`}
              />
            </li>
          ))}
        </ul>

        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            disabled={busy}
            onClick={onAddSection}
          >
            <Plus className="size-3.5" />
            Add a section
          </Button>
          <p className="text-xs text-muted-foreground">
            Save the song before showing it on the screen.
          </p>
        </div>
      </div>
    </Panel>
  );
}


/**
 * What is on the projector right now, with Back/Next.
 *
 * This is how a song is walked through verse by verse: every section was queued
 * when the song started, so Next simply takes the following one.
 */
function NowShowing({
  item,
  busy,
  onPrevious,
  onNext,
}: {
  item: PresentationItem | null;
  busy: boolean;
  onPrevious: () => void;
  onNext: () => void;
}) {
  const payload = item?.payload;

  return (
    <Panel
      title="On screen now"
      actions={
        <div className="flex items-center gap-1">
          <Button variant="ghost" size="sm" disabled={busy} onClick={onPrevious}>
            <ChevronLeft className="size-3.5" />
            Back
          </Button>
          <Button variant="ghost" size="sm" disabled={busy} onClick={onNext}>
            Next
            <ChevronRight className="size-3.5" />
          </Button>
        </div>
      }
    >
      {!item ? (
        <EmptyHint>The screen is blank right now.</EmptyHint>
      ) : (
        <div className="space-y-1.5">
          <p className="text-sm font-medium">{item.title || "Untitled"}</p>
          {payload?.kind === "song" ? (
            <p className="text-xs tracking-[0.14em] text-muted-foreground uppercase">
              Section {payload.index} of {payload.total}
              {payload.label ? ` · ${payload.label}` : ""}
            </p>
          ) : null}
          <p className="selectable line-clamp-3 text-sm text-muted-foreground">
            {payload?.kind === "media" ? payload.path : payload?.text}
          </p>
        </div>
      )}
    </Panel>
  );
}

