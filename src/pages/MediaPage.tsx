import { useCallback, useEffect, useState } from "react";
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
import { mediaApi, presentationApi } from "@/lib/api";
import { friendlyContentType } from "@/lib/content";
import { EVENTS, useTauriEvent } from "@/lib/events";
import { mediaUrl } from "@/lib/media";
import type {
  DirectoryListing,
  MediaItem,
  MediaPlaybackState,
  PresentationItem,
} from "@/types";

/**
 * Media library.
 *
 * Selah stores only metadata in SQLite; the files stay on disk. Nothing is
 * hardcoded: the operator picks a folder at runtime and Selah reads the
 * pictures, videos and sound files inside it. That folder is also what the
 * projector window is allowed to read.
 */
export default function MediaPage() {
  const [items, setItems] = useState<MediaItem[]>([]);
  const [directory, setDirectory] = useState<string | undefined>();
  const [recursive, setRecursive] = useState(false);
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [pickerOpen, setPickerOpen] = useState(false);

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

  /** Loads a folder: grants access, registers what is inside, remembers it. */
  const loadFolder = useCallback(
    async (path: string, includeSubFolders: boolean) => {
      setBusy(true);
      try {
        const scan = await mediaApi.loadMediaDirectory(path, includeSubFolders);
        setDirectory(scan.directory);
        reload();
        setError(null);
        setNotice(
          `${scan.total} file${scan.total === 1 ? "" : "s"} ready${
            scan.added > 0 ? ` — ${scan.added} added` : ""
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

  async function show(item: MediaItem) {
    setBusy(true);
    try {
      // Only sound gets a caption: pictures and video fill the screen as they
      // are, without the file name printed over the top.
      await mediaApi.projectMedia(
        item.path,
        item.kind === "audio" ? item.name : undefined,
      );
      setError(null);
      setNotice(`${item.name} is on the screen.`);
    } catch (e: unknown) {
      setNotice(null);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <>
      <PageHeader
        title="Media"
        subtitle="Your pictures, videos and sound files stay where they are — Selah only remembers where to find them"
        actions={<Badge variant="muted">{items.length} files</Badge>}
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
              Also read folders inside the one you picked.
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
          reads them from the folder you choose.
        </p>
      </Panel>

      <Panel title={directory ? "Files in this folder" : "Your files"}>
        {items.length === 0 ? (
          <EmptyHint>
            Nothing here yet. Choose a folder above and Selah will read the
            pictures and videos inside it.
          </EmptyHint>
        ) : (
          <ScrollArea className="max-h-[30rem]">
            <ul className="grid gap-3 pr-2 sm:grid-cols-2 lg:grid-cols-3">
              {items.map((item) => (
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
              ))}
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
                  This folder is empty. Use “Up” to go back.
                </li>
              ) : null}
            </ul>
          </ScrollArea>

          <p className="text-xs text-muted-foreground">
            {presentable} picture/video/sound file
            {presentable === 1 ? "" : "s"} directly in this folder.
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
