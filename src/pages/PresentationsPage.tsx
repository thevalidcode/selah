import { useCallback, useEffect, useState } from "react";
import { Plus } from "lucide-react";

import PageHeader, { EmptyHint, Panel } from "@/components/PageHeader";
import AddItemPanel from "@/components/presentations/AddItemPanel";
import PresentationItemsPanel from "@/components/presentations/PresentationItemsPanel";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { presentationApi } from "@/lib/api";
import type { Presentation } from "@/types";

/**
 * Saved presentations.
 *
 * This phase stores and edits presentations; the presentation engine can
 * project any item. A full rundown/playlist editor is a later phase — the
 * schema and repository already support it.
 */
export default function PresentationsPage() {
  const [presentations, setPresentations] = useState<Presentation[]>([]);
  const [selectedId, setSelectedId] = useState<string | undefined>();
  const [selected, setSelected] = useState<Presentation | null>(null);
  const [newName, setNewName] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(() => {
    presentationApi
      .listPresentations()
      .then((list) => {
        setPresentations(list);
        setSelectedId((prev) => prev ?? list[0]?.id);
      })
      .catch(() => setPresentations([]));
  }, []);

  useEffect(reload, [reload]);

  const refreshSelected = useCallback(async (id: string) => {
    try {
      setSelected(await presentationApi.getPresentation(id));
    } catch {
      setSelected(null);
    }
  }, []);

  useEffect(() => {
    if (selectedId) {
      void refreshSelected(selectedId);
    } else {
      setSelected(null);
    }
  }, [selectedId, refreshSelected]);

  async function create() {
    if (newName.trim().length === 0) {
      return;
    }
    setBusy(true);
    try {
      const created = await presentationApi.createPresentation(newName.trim());
      setNewName("");
      reload();
      setSelectedId(created.id);
      setError(null);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function remove(id: string) {
    setBusy(true);
    try {
      await presentationApi.deletePresentation(id);
      if (selectedId === id) {
        setSelectedId(undefined);
      }
      reload();
      setError(null);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <>
      <PageHeader
        title="Presentations"
        subtitle="Saved service content, stored in the local database"
        actions={<Badge variant="muted">{presentations.length} saved</Badge>}
      />

      {error ? (
        <p className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
          {error}
        </p>
      ) : null}

      <div className="grid gap-4 lg:grid-cols-[19rem_1fr]">
        <div className="flex flex-col gap-4">
          <Panel title="New presentation">
            <div className="flex gap-2">
              <Input
                value={newName}
                onChange={(e) => setNewName(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    void create();
                  }
                }}
                placeholder="Sunday Morning"
              />
              <Button
                size="icon"
                aria-label="Create presentation"
                disabled={busy || newName.trim().length === 0}
                onClick={() => void create()}
              >
                <Plus className="size-4" />
              </Button>
            </div>
          </Panel>

          <Panel title="Saved">
            {presentations.length === 0 ? (
              <EmptyHint>No presentations saved yet.</EmptyHint>
            ) : (
              <ul className="space-y-1">
                {presentations.map((item) => (
                  <li key={item.id}>
                    <button
                      type="button"
                      onClick={() => setSelectedId(item.id)}
                      className={`flex w-full items-center justify-between gap-2 rounded-md px-3 py-2 text-left text-sm transition-colors hover:bg-accent ${
                        selectedId === item.id ? "bg-accent" : ""
                      }`}
                    >
                      <span className="truncate">{item.name}</span>
                      <span className="text-xs text-muted-foreground">
                        {item.updatedAt.slice(0, 10)}
                      </span>
                    </button>
                  </li>
                ))}
              </ul>
            )}
          </Panel>
        </div>

        <div className="flex flex-col gap-4">
          <PresentationItemsPanel
            presentation={selected}
            busy={busy}
            onDelete={() => {
              if (selected) {
                void remove(selected.id);
              }
            }}
            onRemoveItem={async (itemId) => {
              if (!selected) {
                return;
              }
              await presentationApi.removePresentationItem(selected.id, itemId);
              await refreshSelected(selected.id);
            }}
          />

          <AddItemPanel
            presentationId={selected?.id}
            busy={busy}
            onAdded={() => {
              if (selected) {
                void refreshSelected(selected.id);
              }
            }}
            onError={setError}
          />
        </div>
      </div>
    </>
  );
}
