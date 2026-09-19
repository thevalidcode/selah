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
  const [heading, setHeading] = useState("");
  const [body, setBody] = useState("");
  const [saving, setSaving] = useState(false);

  async function add() {
    if (!presentationId || heading.trim().length === 0) {
      return;
    }
    setSaving(true);
    try {
      // The heading travels inside the payload, which is what keeps it on the
      // projector later: the list row stores JSON, not a separate column.
      const payload =
        type === "scripture"
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

      await presentationApi.addPresentationItem(presentationId, type, payload);
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

  return (
    <Panel title="Add to this presentation">
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
              onClick={() => void add()}
            >
              <Plus className="size-4" />
              Add
            </Button>
            <p className="flex items-center gap-1.5 text-xs text-muted-foreground">
              <Braces className="size-3" />
              Saved with this presentation
            </p>
          </div>
        </div>
      )}
    </Panel>
  );
}
