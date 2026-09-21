import { useCallback, useEffect, useRef, useState } from "react";
import {
  ArrowUp,
  FileImage,
  FileVideo,
  Folder,
  FolderOpen,
  MonitorPlay,
  Music,
  Pause,
  Play,
  RefreshCw,
  RotateCcw,
  EyeOff,
  Trash2,
} from "lucide-react";

import VideoClipDialog from "@/components/media/VideoClipDialog";
import PageHeader, { EmptyHint, Panel } from "@/components/PageHeader";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import { useSettings } from "@/hooks/useSettings";
import { mediaApi, presentationApi } from "@/lib/api";
import { friendlyContentType } from "@/lib/content";
import { EVENTS, useTauriEvent } from "@/lib/events";
import { clipFromMetadata, describeClip, mediaUrl } from "@/lib/media";
import type {
  DirectoryListing,
  MediaClip,
  MediaItem,
  MediaPlaybackState,
  PresentationItem,
} from "@/types";

/**
 * Media library.
 *
 * Selah stores only metadata in SQLite; the files stay on disk. Nothing is
 * hardcoded: the operator picks a folder at runtime and Selah reads the
 * pictures, videos and sound files inside it — sub-folders too, when the switch
 * is on. That folder is also what the projector window is allowed to read.
 *
 * The list below shows what is *in that folder*, which is the question the page
 * is answering ("what can I put up tonight?"); the whole library is one switch
 * away for anyone who wants to see everything Selah has ever read.
 */
export default function MediaPage() {
  const [items, setItems] = useState<MediaItem[]>([]);
  const [directory, setDirectory] = useState<string | undefined>();
  const [recursive, setRecursive] = useState(false);
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [pickerOpen, setPickerOpen] = useState(false);

  // What the last read of the folder found. `null` means "no folder read in
  // this session", which is when the library list is the only thing to show.
  const [folderFiles, setFolderFiles] = useState<MediaItem[] | null>(null);
  const [showEveryFile, setShowEveryFile] = useState(false);
  // The video open in the preview / trim dialog, if any.
  const [trimming, setTrimming] = useState<MediaItem | null>(null);

  const { settings } = useSettings();

  // What is on the congregation's screen, and (for a video or a piece of
  // music) whether it is playing. Only the projector window knows that for
  // certain, so it reports back over `media://playback-state`.
  const [onScreen, setOnScreen] = useState<PresentationItem | null>(null);
  const [playback, setPlayback] = useState<MediaPlaybackState>({
    playing: false,
    positionMs: 0,
    durationMs: 0,
    ended: false,
  });

  // The path of whatever is on the screen, so the library can mark it.
  const onScreenPath =
    onScreen?.payload.kind === "media" ? onScreen.payload.path : undefined;

  useTauriEvent(EVENTS.presentationChanged, (payload) => {
    setOnScreen(payload.item ?? null);
  });

  useTauriEvent(EVENTS.mediaPlaybackState, (payload) => setPlayback(payload));

  // Closing the projector window takes everything off the screen, so the panel
  // must not keep claiming something is showing.
  useTauriEvent(EVENTS.presentationDisplayClosed, () => {
    setOnScreen(null);
    setPlayback({
      playing: false,
      positionMs: 0,
      durationMs: 0,
      ended: false,
    });
  });

  useEffect(() => {
    mediaApi
      .getMediaPlaybackState()
      .then(setPlayback)
      .catch(() => undefined);
  }, []);

  /** Sends a play/pause/restart/stop instruction to the projector. */
  const control = useCallback(
    async (action: "play" | "pause" | "restart" | "stop") => {
      setBusy(true);
      try {
        setPlayback(await mediaApi.controlMediaPlayback(action));
        setError(null);
      } catch (e: unknown) {
        setError(e instanceof Error ? e.message : String(e));
      } finally {
        setBusy(false);
      }
    },
    [],
  );

  async function hideScreen() {
    setBusy(true);
    try {
      await presentationApi.clearPresentation();
      setError(null);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  const reload = useCallback(() => {
    mediaApi
      .listMedia()
      .then(setItems)
      .catch(() => setItems([]));
  }, []);

  useEffect(reload, [reload]);

  /**
   * Loads a folder: grants access, registers what is inside, remembers it.
   *
   * The files the scan reports are kept as the folder's own list, so the page
   * shows exactly what the folder holds — including everything in its
   * sub-folders when the switch is on — rather than every file Selah has ever
   * read.
   */
  const loadFolder = useCallback(
    async (path: string, includeSubFolders: boolean) => {
      setBusy(true);
      try {
        const scan = await mediaApi.loadMediaDirectory(path, includeSubFolders);
        setDirectory(scan.directory);
        setFolderFiles(scan.items);
        reload();

        // Anything the scan could not read is said out loud: files that are
        // simply missing from a list are the hardest thing to explain.
        const notes: string[] = [];
        if (scan.added > 0) {
          notes.push(`${scan.added} added`);
        }
        if (scan.skipped > 0) {
          notes.push(
            `${scan.skipped} folder${scan.skipped === 1 ? "" : "s"} could not be read`,
          );
        }
        if (scan.truncated) {
          notes.push("Selah stopped early — there is more here than one pass");
        }

        setError(null);
        setNotice(
          `${scan.total} file${scan.total === 1 ? "" : "s"} ready${
            notes.length > 0 ? ` — ${notes.join(", ")}` : ""
          }.`,
        );
      } catch (e: unknown) {
        setNotice(null);
        setError(e instanceof Error ? e.message : String(e));
      } finally {
        setBusy(false);
      }
    },
    [reload],
  );

  // The folder the operator chose last time is remembered in settings, so the
  // page opens where they left it. Reading it again straight away is what makes
  // files added since — in the folder or in any of its sub-folders — appear
  // without having to pick the folder all over again.
  const restored = useRef(false);
  useEffect(() => {
    if (restored.current || !settings) {
      return;
    }
    restored.current = true;

    const remembered = settings.media.directory;
    setRecursive(settings.media.recursive);
    if (remembered) {
      setDirectory(remembered);
      void loadFolder(remembered, settings.media.recursive);
    }
  }, [settings, loadFolder]);

  async function remove(id: string) {
    setBusy(true);
    try {
      await mediaApi.removeMedia(id);
      setError(null);
      reload();
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  /** Shows a file, using the range saved for it unless one was just chosen. */
  async function show(item: MediaItem, clip?: MediaClip) {
    setBusy(true);
    try {
      // A picture or a piece of music has no range: only video is trimmed, and
      // sound has always played once.
      const chosen =
        item.kind === "video" ? (clip ?? clipFromMetadata(item.metadata)) : undefined;

      // Only sound gets a caption: pictures and video fill the screen as they
      // are, without the file name printed over the top.
      await mediaApi.projectMedia(
        item.path,
        item.kind === "audio" ? item.name : undefined,
        chosen,
      );
      setError(null);
      const range = clipNote(chosen);
      setNotice(
        range
          ? `${item.name} is on the screen — ${range}.`
          : `${item.name} is on the screen.`,
      );
    } catch (e: unknown) {
      setNotice(null);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  /**
   * Remembers (or forgets) the part of a video to show.
   *
   * The range is kept with the file, so Show uses it from then on — and the
   * lists are updated in place, so the page never shows a range that is no
   * longer the truth.
   */
  async function saveClip(item: MediaItem, clip: MediaClip | null) {
    setBusy(true);
    try {
      const updated = await mediaApi.setMediaClip(item.id, clip);
      const withClip = (list: MediaItem[]) =>
        list.map((entry) => (entry.id === item.id ? updated : entry));

      setItems(withClip);
      setFolderFiles((current) => (current ? withClip(current) : current));
      setTrimming((current) => (current?.id === item.id ? updated : current));
      setError(null);
      setNotice(
        clip
          ? `${item.name} will play ${describeClip(clip) ?? "whole"}.`
          : `${item.name} plays whole again.`,
      );
    } catch (e: unknown) {
      setNotice(null);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  // The folder view is the default: it answers "what is in the folder I chose".
  // The whole library stays one switch away for anyone who wants it.
  const folderView = !showEveryFile && folderFiles !== null;
  const visibleItems = folderView ? folderFiles : items;

  return (
    <>
      <PageHeader
        title="Media"
        subtitle="Your pictures, videos and sound files stay where they are — Selah only remembers where to find them"
        actions={
          <Badge variant="muted">
            {visibleItems.length} file{visibleItems.length === 1 ? "" : "s"}
          </Badge>
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

      <OnScreenNow
        item={onScreen}
        playback={playback}
        busy={busy}
        onControl={(action) => void control(action)}
        onHide={() => void hideScreen()}
      />

      <Panel title="Folder to read from">
        <div className="flex flex-col gap-3 sm:flex-row sm:items-end">
          <div className="flex-1 space-y-1.5">
            <Label htmlFor="media-folder">Folder on this computer</Label>
            <input
              id="media-folder"
              readOnly
              value={directory ?? ""}
              placeholder="No folder chosen yet"
              className="border-input h-9 w-full rounded-md border bg-transparent px-3 py-1 text-sm shadow-xs"
            />
          </div>
          <Button variant="success" onClick={() => setPickerOpen(true)}>
            <FolderOpen className="size-4" />
            Choose folder…
          </Button>
          <Button
            variant="outline"
            disabled={busy || !directory}
            onClick={() => directory && void loadFolder(directory, recursive)}
          >
            <RefreshCw className="size-4" />
            Read again
          </Button>
        </div>

        <div className="mt-3 flex items-center justify-between rounded-lg border border-border/60 px-3 py-2">
          <div>
            <p className="text-sm font-medium">Include sub-folders</p>
            <p className="text-xs text-muted-foreground">
              Also read folders inside the one you picked. Changing this reads
              the folder again straight away.
            </p>
          </div>
          <Switch
            checked={recursive}
            onCheckedChange={(value) => {
              setRecursive(value);
              if (directory) {
                void loadFolder(directory, value);
              }
            }}
          />
        </div>
        <p className="mt-2 text-xs text-muted-foreground">
          Pictures, video and sound files all work. Selah never copies them — it
          reads them from the folder you choose, and shows you what it found.
        </p>
      </Panel>

      <Panel
        title={folderView ? "Files in this folder" : "Your files"}
        actions={
          <div className="flex items-center gap-3">
            <Badge variant="muted">
              {visibleItems.length} file{visibleItems.length === 1 ? "" : "s"}
            </Badge>
            <label className="flex items-center gap-2 text-xs text-muted-foreground">
              Show every file
              <Switch
                checked={showEveryFile}
                onCheckedChange={setShowEveryFile}
                aria-label="Show every file in the library"
              />
            </label>
          </div>
        }
      >
        {visibleItems.length === 0 ? (
          <EmptyHint>
            {folderView
              ? "Nothing Selah can present is in this folder yet. Turn on “Include sub-folders” above if your files sit in folders inside it, or choose another folder."
              : "Nothing here yet. Choose a folder above and Selah will read the pictures, videos and sound files inside it."}
          </EmptyHint>
        ) : (
          <ScrollArea className="max-h-[30rem]">
            <ul className="grid gap-3 pr-2 sm:grid-cols-2 lg:grid-cols-3">
              {visibleItems.map((item) => {
                const clip =
                  item.kind === "video"
                    ? clipFromMetadata(item.metadata)
                    : undefined;
                const range = clipNote(clip);

                return (
                <li
                  key={item.id}
                  className="flex flex-col overflow-hidden rounded-lg border border-border/60"
                >
                  {item.kind === "image" ? (
                    <img
                      src={mediaUrl(item.path)}
                      alt={item.name}
                      className="h-32 w-full bg-black object-contain"
                    />
                  ) : (
                    <div className="grid h-32 w-full place-items-center bg-muted">
                      <MediaIcon kind={item.kind} className="size-6" />
                    </div>
                  )}

                  <div className="flex flex-1 flex-col gap-2 p-2.5">
                    <div className="min-w-0 flex-1">
                      <p
                        className="truncate text-sm font-medium"
                        title={item.name}
                      >
                        {item.name}
                      </p>
                      <p className="selectable truncate text-xs text-muted-foreground">
                        {item.path}
                      </p>
                      {range ? (
                        // Which part of a video will play: worth saying on the
                        // card, so nobody has to open the file to find out.
                        <p className="mt-0.5 text-xs text-brand">{range}</p>
                      ) : null}
                    </div>
                    <div className="flex items-center justify-between gap-1">
                      {onScreenPath === item.path ? (
                        // The file that is actually on the congregation's screen
                        // is marked here, and says so when it is playing.
                        <Badge
                          variant={
                            item.kind === "image"
                              ? "warning"
                              : playback.playing
                                ? "success"
                                : "warning"
                          }
                        >
                          {item.kind === "image"
                            ? "on screen"
                            : playback.playing
                              ? "playing"
                              : "on screen"}
                        </Badge>
                      ) : (
                        <Badge variant="muted">
                          {friendlyContentType(item.kind)}
                        </Badge>
                      )}
                      <div className="flex items-center gap-1">
                        {item.kind === "video" ? (
                          <Button
                            variant="outline"
                            size="sm"
                            disabled={busy}
                            onClick={() => setTrimming(item)}
                          >
                            <Play className="size-3.5" />
                            Preview
                          </Button>
                        ) : null}
                        <Button
                          variant="success"
                          size="sm"
                          disabled={busy}
                          onClick={() => void show(item)}
                        >
                          <MonitorPlay className="size-3.5" />
                          Show
                        </Button>
                        <Button
                          variant="ghost"
                          size="icon"
                          aria-label={`Remove ${item.name}`}
                          disabled={busy}
                          onClick={() => void remove(item.id)}
                        >
                          <Trash2 className="size-4" />
                        </Button>
                      </div>
                    </div>
                  </div>
                </li>
              );
              })}
            </ul>
          </ScrollArea>
        )}
      </Panel>

      <FolderPicker
        open={pickerOpen}
        busy={busy}
        onOpenChange={setPickerOpen}
        onUseFolder={(path) => {
          setPickerOpen(false);
          void loadFolder(path, recursive);
        }}
      />

      {/*
        Previewing and trimming a video. The dialog owns the draft range; the
        page owns saving it and putting it on the screen, so both go through the
        same code as the Show button on a card.
      */}
      <VideoClipDialog
        item={trimming}
        savedClip={
          trimming ? clipFromMetadata(trimming.metadata) : undefined
        }
        repeatByDefault={settings?.presentation.repeatVideos ?? true}
        open={trimming !== null}
        busy={busy}
        onOpenChange={(open) => {
          if (!open) {
            setTrimming(null);
          }
        }}
        onSave={(clip) => {
          if (trimming) {
            void saveClip(trimming, clip);
          }
        }}
        onShow={(clip) => {
          if (trimming) {
            void show(trimming, clip);
          }
        }}
      />
    </>
  );

/**
 * Walks the filesystem so a folder can be picked.
 *
 * Selah does not request broad filesystem permissions and no folder path is
 * built into the application: this browser is how the operator points Selah at
 * the folder their media lives in.
 */
function FolderPicker({
  open,
  busy,
  onOpenChange,
  onUseFolder,
}: {
  open: boolean;
  busy: boolean;
  onOpenChange: (open: boolean) => void;
  onUseFolder: (path: string) => void;
}) {
  const [listing, setListing] = useState<DirectoryListing | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const browse = useCallback(async (path?: string) => {
    setLoading(true);
    try {
      setListing(await mediaApi.browseDirectory(path));
      setError(null);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  // Start at the home folder each time the picker is opened.
  useEffect(() => {
    if (open) {
      void browse();
    }
  }, [open, browse]);

  const folders = listing?.entries.filter((entry) => entry.isDir) ?? [];
  const files = listing?.entries.filter((entry) => !entry.isDir) ?? [];
  const presentable = files.filter((entry) => entry.mediaKind).length;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle>Where are your pictures and videos?</DialogTitle>
          <DialogDescription>
            Open folders until you are in the right one, then press “Read this
            folder”.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-3">
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              disabled={!listing?.parent || loading}
              onClick={() => listing?.parent && void browse(listing.parent)}
            >
              <ArrowUp className="size-3.5" />
              Up
            </Button>
            <code className="selectable min-w-0 flex-1 truncate rounded-md bg-muted px-2 py-1 text-xs">
              {listing?.path ?? "…"}
            </code>
          </div>

          {error ? (
            <p className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
              {error}
            </p>
          ) : null}

          <ScrollArea className="h-72 rounded-md border border-border/60">
            <ul className="p-1.5">
              {folders.map((entry) => (
                <li key={entry.path}>
                  <button
                    type="button"
                    onClick={() => void browse(entry.path)}
                    className="flex h-8 w-full items-center gap-2 rounded-md px-2 text-left text-sm transition-colors hover:bg-accent"
                  >
                    <Folder className="size-4 shrink-0 text-muted-foreground" />
                    <span className="truncate">{entry.name}</span>
                  </button>
                </li>
              ))}
              {files.map((entry) => (
                <li
                  key={entry.path}
                  className="flex h-8 items-center gap-2 px-2 text-sm text-muted-foreground"
                >
                  <MediaIcon kind={entry.mediaKind ?? "other"} />
                  <span className="truncate">{entry.name}</span>
                </li>
              ))}
              {folders.length === 0 && files.length === 0 && !loading ? (
                <li className="px-2 py-3 text-sm text-muted-foreground">
                  This folder is empty. Use “Up” to go back. Folders inside it
                  are listed above.
                </li>
              ) : null}
            </ul>
          </ScrollArea>

          <p className="text-xs text-muted-foreground">
            {presentable} picture/video/sound file
            {presentable === 1 ? "" : "s"} directly in this folder.
            {listing?.entries.some((entry) => entry.isDir)
              ? " The folders listed above are read as well when “Include sub-folders” is on."
              : ""}
          </p>
        </div>

        <DialogFooter>
          <Button
            variant="success"
            disabled={!listing || busy}
            onClick={() => listing && onUseFolder(listing.path)}
          >
            <FolderOpen className="size-4" />
            Read this folder
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

/**
 * What is on the congregation's screen, with playback control.
 *
 * A picture simply sits there; a video (or a piece of music) keeps moving, and
 * the operator needs to be able to start it again, hold it, or send it back to
 * the beginning without walking to the projector. Whether it is *really*
 * playing is reported by the projector window, not guessed here.
 */
function OnScreenNow({
  item,
  playback,
  busy,
  onControl,
  onHide,
}: {
  item: PresentationItem | null;
  playback: MediaPlaybackState;
  busy: boolean;
  onControl: (action: "play" | "pause" | "restart" | "stop") => void;
  onHide: () => void;
}) {
  const payload = item?.payload;
  const isMedia = payload?.kind === "media";
  const kind = isMedia ? (payload.mediaKind ?? "image") : undefined;
  const playing = isMedia && playback.playing;

  /**
   * The part of a video that is playing, when the operator chose one.
   *
   * It is shown here because "6s → 30s" is not something anyone should have to
   * remember while a service is running.
   */
  const clip: MediaClip | undefined =
    payload?.kind === "media" && kind === "video"
      ? {
          startMs: payload.startMs ?? 0,
          endMs: payload.endMs,
          repeat: payload.repeat ?? false,
        }
      : undefined;
  const range = clipNote(clip);

  /** Where the file has got to, as `0:42 / 2:10`. */
  const progress =
    playback.durationMs > 0
      ? `${formatTime(playback.positionMs)} / ${formatTime(playback.durationMs)}`
      : playback.positionMs > 0
        ? formatTime(playback.positionMs)
        : null;

  return (
    <Panel
      title="On the screen now"
      actions={
        item ? (
          <div className="flex items-center gap-1">
            <Badge variant={playing ? "success" : "muted"}>
              {isMedia
                ? kind === "video"
                  ? playing
                    ? "playing"
                    : playback.ended
                      ? "finished"
                      : "paused"
                  : kind === "audio"
                    ? playing
                      ? "playing"
                      : "stopped"
                    : "showing"
                : "showing"}
            </Badge>
            <Button variant="ghost" size="sm" disabled={busy} onClick={onHide}>
              <EyeOff className="size-3.5" />
              Take it off
            </Button>
          </div>
        ) : null
      }
    >
      {!item ? (
        <EmptyHint>
          The screen is blank. Press Show on a file to put it up.
        </EmptyHint>
      ) : (
        <div className="space-y-3">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div className="min-w-0">
              <p className="truncate text-sm font-medium">
                {item.title || "Untitled"}
              </p>
              <p className="text-xs text-muted-foreground">
                {friendlyContentType(item.contentType)}
                {isMedia && payload.kind === "media"
                  ? ` · ${payload.path}`
                  : ""}
              </p>
              {range ? (
                <p className="text-xs text-brand">
                  Showing {range} — the rest of the file is skipped.
                </p>
              ) : null}
            </div>
            {progress ? (
              <p className="font-mono text-xs text-muted-foreground">
                {progress}
              </p>
            ) : null}
          </div>

          {/*
            Only video and sound can be played, so only they get buttons. A
            picture with a play button would be a lie.
          */}
          {isMedia && kind !== "image" ? (
            <>
              <Separator />
              <div className="flex flex-wrap items-center gap-2">
                <Button
                  variant="success"
                  size="sm"
                  disabled={busy || playing}
                  onClick={() => onControl("play")}
                >
                  <Play className="size-3.5" />
                  Play
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  disabled={busy || !playing}
                  onClick={() => onControl("pause")}
                >
                  <Pause className="size-3.5" />
                  Pause
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  disabled={busy}
                  onClick={() => onControl("restart")}
                >
                  <RotateCcw className="size-3.5" />
                  Start again
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  disabled={busy}
                  onClick={() => onControl("stop")}
                >
                  Stop
                </Button>
                <p className="text-xs text-muted-foreground">
                  {playing
                    ? "It is playing on the screen now."
                    : playback.ended
                      ? "It has reached the end. Start it again, or take it off."
                      : "It is on the screen, held at the beginning."}
                </p>
              </div>
            </>
          ) : (
            <p className="text-xs text-muted-foreground">
              A picture stays as it is until you show something else or take it
              off.
            </p>
          )}
        </div>
      )}
    </Panel>
  );
}

/** `m:ss` for a progress readout. */
function formatTime(ms: number): string {
  const totalSeconds = Math.max(0, Math.round(ms / 1000));
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes}:${String(seconds).padStart(2, "0")}`;
}

/**
 * The part of a video that will play, as one short line.
 *
 * `describeClip` says nothing about a whole file, which is right for a card —
 * but a whole file that stops at its end, or repeats, is a real choice and is
 * worth saying out loud.
 */
function clipNote(clip: MediaClip | undefined): string | null {
  if (!clip) {
    return null;
  }
  return describeClip(clip) ?? (clip.repeat ? "the whole video, repeating" : null);
}


function MediaIcon({ kind, className }: { kind: string; className?: string }) {
  const style = className ?? "size-4 shrink-0 text-muted-foreground";
  if (kind === "video") {
    return <FileVideo className={style} />;
  }
  if (kind === "audio") {
    return <Music className={style} />;
  }
  return <FileImage className={style} />;
}

}
