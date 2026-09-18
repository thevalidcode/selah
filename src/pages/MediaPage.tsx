import { useEffect, useState } from "react";
import { FileImage, FileVideo, Import, Music, Trash2 } from "lucide-react";

import PageHeader, { EmptyHint, Panel } from "@/components/PageHeader";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ScrollArea } from "@/components/ui/scroll-area";
import { mediaApi } from "@/lib/api";
import { friendlyContentType } from "@/lib/content";
import type { MediaItem } from "@/types";

/**
 * Media library.
 *
 * Selah stores only metadata in SQLite; files stay on disk and are referenced
 * by absolute path. Selah does not request broad filesystem permissions, so
 * importing means pasting (or dropping) a path the operator already knows.
 */
export default function MediaPage() {
  const [items, setItems] = useState<MediaItem[]>([]);
  const [path, setPath] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  function reload() {
    mediaApi
      .listMedia()
      .then(setItems)
      .catch(() => setItems([]));
  }

  useEffect(reload, []);

  async function importFromPath() {
    if (path.trim().length === 0) {
      return;
    }
    setBusy(true);
    setError(null);
    try {
      await mediaApi.importMedia(path.trim());
      setPath("");
      reload();
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

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

      <Panel title="Add a file">
        <div className="flex flex-col gap-3 sm:flex-row sm:items-end">
          <div className="flex-1 space-y-1.5">
            <Label htmlFor="media-path">File on this computer</Label>
            <Input
              id="media-path"
              value={path}
              onChange={(e) => setPath(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  void importFromPath();
                }
              }}
              placeholder="/Users/you/Pictures/sermon-slide.png"
            />
          </div>
          <Button
            variant="success"
            disabled={busy || path.trim().length === 0}
            onClick={() => void importFromPath()}
          >
            <Import className="size-4" />
            Add
          </Button>
        </div>
        <p className="mt-2 text-xs text-muted-foreground">
          Pictures, video and sound files all work.
        </p>
      </Panel>

      <Panel title="Your files">
        {items.length === 0 ? (
          <EmptyHint>
            Nothing added yet. Add a file and it becomes available to put on the
            screen.
          </EmptyHint>
        ) : (
          <ScrollArea className="max-h-[26rem]">
            <ul className="space-y-1.5 pr-2">
              {items.map((item) => (
                <li
                  key={item.id}
                  className="flex items-center gap-3 rounded-lg border border-border/60 px-3 py-2"
                >
                  <span className="grid size-8 shrink-0 place-items-center rounded-md bg-muted">
                    <MediaIcon kind={item.kind} />
                  </span>
                  <div className="min-w-0 flex-1">
                    <p className="truncate text-sm font-medium">{item.name}</p>
                    <p className="selectable truncate text-xs text-muted-foreground">
                      {item.path}
                    </p>
                  </div>
                  <Badge variant="muted">{friendlyContentType(item.kind)}</Badge>
                  <Button
                    variant="ghost"
                    size="icon"
                    aria-label={`Remove ${item.name}`}
                    disabled={busy}
                    onClick={() => void remove(item.id)}
                  >
                    <Trash2 className="size-4" />
                  </Button>
                </li>
              ))}
            </ul>
          </ScrollArea>
        )}
      </Panel>
    </>
  );
}

function MediaIcon({ kind }: { kind: string }) {
  if (kind === "video") {
    return <FileVideo className="size-4 text-muted-foreground" />;
  }
  if (kind === "audio") {
    return <Music className="size-4 text-muted-foreground" />;
  }
  return <FileImage className="size-4 text-muted-foreground" />;
}
