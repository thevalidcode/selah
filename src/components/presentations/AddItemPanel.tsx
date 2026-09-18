import { useState } from "react";
import { Braces, Plus } from "lucide-react";

import { EmptyHint, Panel } from "@/components/PageHeader";
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

/**
 * Composer for adding an item to a saved presentation.
 *
 * Payloads are JSON so the `presentation_items` table never needs a schema
 * change when a new content type is introduced.
 */
export default function AddItemPanel({
  presentationId,
  busy,
  onAdded,
  onError,
}: {
  presentationId?: string;
  busy: boolean;
  onAdded: () => void;
  onError: (message: string | null) => void;
}) {
  const [type, setType] = useState("text");
  const [title, setTitle] = useState("");
  const [body, setBody] = useState("");
  const [saving, setSaving] = useState(false);

  async function add() {
    if (!presentationId || title.trim().length === 0) {
      return;
    }
    setSaving(true);
    try {
      const payload =
        type === "scripture"
          ? JSON.stringify({
              kind: "scripture",
              reference: title.trim(),
              translation: "",
              text: body,
            })
          : JSON.stringify({ kind: "text", text: body });

      await presentationApi.addPresentationItem(presentationId, type, payload);
      setTitle("");
      setBody("");
      onError(null);
      onAdded();
    } catch (e: unknown) {
      onError(e instanceof Error ? e.message : String(e));
    } finally {
      setSaving(false);
    }
  }

  return (
    <Panel title="Add item">
      {!presentationId ? (
        <EmptyHint>Create or select a presentation first.</EmptyHint>
      ) : (
        <div className="space-y-3">
          <div className="grid gap-3 sm:grid-cols-[10rem_1fr]">
            <div className="space-y-1.5">
              <Label htmlFor="item-type">Type</Label>
              <Select value={type} onValueChange={setType}>
                <SelectTrigger id="item-type">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="text">Text</SelectItem>
                  <SelectItem value="scripture">Scripture</SelectItem>
                  <SelectItem value="announcement">Announcement</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="item-title">
                {type === "scripture" ? "Reference" : "Title"}
              </Label>
              <Input
                id="item-title"
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                placeholder={type === "scripture" ? "John 3:16" : "Welcome"}
              />
            </div>
          </div>

          <div className="space-y-1.5">
            <Label htmlFor="item-body">Body</Label>
            <Textarea
              id="item-body"
              value={body}
              onChange={(e) => setBody(e.target.value)}
              placeholder="Text shown on the presentation screen"
            />
          </div>

          <div className="flex items-center gap-3">
            <Button
              variant="success"
              disabled={busy || saving || title.trim().length === 0}
              onClick={() => void add()}
            >
              <Plus className="size-4" />
              Add item
            </Button>
            <p className="flex items-center gap-1.5 text-xs text-muted-foreground">
              <Braces className="size-3" />
              Stored as a JSON payload
            </p>
          </div>
        </div>
      )}
    </Panel>
  );
}
