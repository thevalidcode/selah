import { useEffect, useState } from "react";
import { Braces, Megaphone, Pencil, Plus, Save } from "lucide-react";

import { EmptyHint, Panel } from "@/components/PageHeader";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { presentationApi } from "@/lib/api";
import type { EditingItem } from "@/lib/presentations";

/**
 * Composer for adding an item to a saved presentation, and for editing one that
 * is already in it.
 *
 * Payloads are JSON so the `presentation_items` table never needs a schema
 * change when a new content type is introduced.
 */
export default function AddItemPanel({
  presentationId,
  busy,
  editing,
  onAdded,
  onCancelEdit,
  onError,
}: {
  presentationId?: string;
  busy: boolean;
  /** The item being edited; omit (or pass `null`) to add a new one. */
  editing?: EditingItem | null;
  onAdded: () => void;
  onCancelEdit?: () => void;
  onError: (message: string | null) => void;
}) {
  const [type, setType] = useState("text");
  const [heading, setHeading] = useState("");
  const [body, setBody] = useState("");
  const [saving, setSaving] = useState(false);

  // Switching into edit mode fills the form from the stored item; leaving it
  // clears the form so the next "Add" does not inherit edited words.
  useEffect(() => {
    if (editing) {
      setType(editing.typeName);
      setHeading(editing.heading);
      setBody(editing.body);
    } else {
      setHeading("");
      setBody("");
    }
  }, [editing]);

  /** The JSON stored in the row, built from whatever is in the form. */
  function buildPayload(): string {
    return type === "scripture"
      ? JSON.stringify({
          kind: "scripture",
          reference: heading.trim(),
          translation: "",
          heading: heading.trim(),
          text: body,
        })
      : JSON.stringify({
          kind: "text",
          heading: heading.trim(),
          text: body,
        });
  }

  async function add() {
    if (!presentationId || heading.trim().length === 0) {
      return;
    }
    setSaving(true);
    try {
      // The heading travels inside the payload, which is what keeps it on the
      // projector later: the list row stores JSON, not a separate column.
      await presentationApi.addPresentationItem(
        presentationId,
        type,
        buildPayload(),
      );
      setHeading("");
      setBody("");
      onError(null);
      onAdded();
    } catch (e: unknown) {
      onError(e instanceof Error ? e.message : String(e));
    } finally {
      setSaving(false);
    }
  }

  /** Saves an edit in place: the item keeps its position in the list. */
  async function saveEdit() {
    if (!presentationId || !editing || heading.trim().length === 0) {
      return;
    }
    setSaving(true);
    try {
      await presentationApi.updatePresentationItem({
        presentationId,
        itemId: editing.id,
        type,
        payload: buildPayload(),
      });
      onError(null);
      onAdded();
      onCancelEdit?.();
    } catch (e: unknown) {
      onError(e instanceof Error ? e.message : String(e));
    } finally {
      setSaving(false);
    }
  }

  return (
    <Panel
      title={editing ? "Edit this item" : "Add to this presentation"}
      actions={
        editing ? (
          <Badge variant="warning">
            <Pencil className="size-3" />
            editing
          </Badge>
        ) : null
      }
    >
      {!presentationId ? (
        <EmptyHint>Create or choose a presentation first.</EmptyHint>
      ) : (
        <div className="space-y-3">
          <div className="grid gap-3 sm:grid-cols-[10rem_1fr]">
            <div className="space-y-1.5">
              <Label htmlFor="item-type">What kind</Label>
              <Select value={type} onValueChange={setType}>
                <SelectTrigger id="item-type">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="text">Words</SelectItem>
                  <SelectItem value="scripture">Bible verse</SelectItem>
                  <SelectItem value="announcement">Notice</SelectItem>
                </SelectContent>
              </Select>
              {type === "announcement" ? (
                <p className="flex items-center gap-1.5 text-xs text-muted-foreground">
                  <Megaphone className="size-3" />
                  Notices look different on screen.
                </p>
              ) : null}
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="item-title">
                {type === "scripture" ? "Verse" : "Heading"}
              </Label>
              <Input
                id="item-title"
                value={heading}
                onChange={(e) => setHeading(e.target.value)}
                placeholder={
                  type === "scripture" ? "John 3:16" : "Welcome everyone"
                }
              />
              <p className="text-xs text-muted-foreground">
                This is the heading shown above the words on the screen.
              </p>
            </div>
          </div>

          <div className="space-y-1.5">
            <Label htmlFor="item-body">Words</Label>
            <Textarea
              id="item-body"
              value={body}
              onChange={(e) => setBody(e.target.value)}
              placeholder="What people will read on the screen"
            />
          </div>

          <div className="flex items-center gap-3">
            <Button
              variant="success"
              disabled={busy || saving || heading.trim().length === 0}
              onClick={() => void (editing ? saveEdit() : add())}
            >
              {editing ? <Save className="size-4" /> : <Plus className="size-4" />}
              {editing ? "Save changes" : "Add"}
            </Button>
            {editing ? (
              <Button
                variant="ghost"
                disabled={busy || saving}
                onClick={() => {
                  onCancelEdit?.();
                  onError(null);
                }}
              >
                Cancel
              </Button>
            ) : (
              <p className="flex items-center gap-1.5 text-xs text-muted-foreground">
                <Braces className="size-3" />
                Saved with this presentation
              </p>
            )}
          </div>
        </div>
      )}
    </Panel>
  );
}
