import { BookOpen, MonitorPlay, Pencil, X } from "lucide-react";

import { EmptyHint } from "@/components/PageHeader";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { formatReference } from "@/lib/reference";
import type { DetectedContent } from "@/types";

/**
 * Detected-content review panel.
 *
 * Safety-critical: detected Scripture is a *suggestion*. The operator must
 * press Display to send it to the projector (spec §24 — never auto-project
 * uncertain content).
 */
export default function DetectedPanel({
  detection,
  bookNames,
  translationLabel,
  working,
  editing,
  editTitle,
  editText,
  onEditTitle,
  onEditText,
  onStartEdit,
  onCancelEdit,
  onDisplay,
  onIgnore,
}: {
  detection?: { content: DetectedContent; confidence?: number };
  bookNames: ReadonlyMap<number, string>;
  translationLabel?: string;
  working: boolean;
  editing: boolean;
  editTitle: string;
  editText: string;
  onEditTitle: (value: string) => void;
  onEditText: (value: string) => void;
  onStartEdit: () => void;
  onCancelEdit: () => void;
  onDisplay: (overrideText?: string) => void;
  onIgnore: () => void;
}) {
  const reference = detection?.content.reference;

  return (
    <Card>
      <CardHeader>
        <CardTitle>Detected content</CardTitle>
        {detection?.confidence !== undefined ? (
          <Badge variant="success">
            {Math.round(detection.confidence * 100)}% match
          </Badge>
        ) : null}
      </CardHeader>
      <CardContent className="space-y-3">
        {!detection ? (
          <EmptyHint>
            No Scripture detected yet. References found in speech appear here for
            confirmation — nothing is projected automatically.
          </EmptyHint>
        ) : (
          <>
            <div className="rounded-lg border border-success/30 bg-success/5 p-4">
              <div className="flex items-center gap-2 text-[10px] font-semibold tracking-[0.16em] text-success uppercase">
                <BookOpen className="size-3" />
                Scripture
              </div>
              <p className="mt-1.5 text-2xl font-semibold tracking-tight">
                {reference
                  ? formatReference(reference, bookNames)
                  : (detection.content.text ?? "Text")}
              </p>
              {translationLabel ? (
                <p className="mt-1 text-xs tracking-[0.2em] text-muted-foreground uppercase">
                  {translationLabel}
                </p>
              ) : null}
            </div>

            {editing ? (
              <div className="space-y-2">
                <Input
                  value={editTitle}
                  onChange={(e) => onEditTitle(e.target.value)}
                  placeholder="Reference label (e.g. John 3:16)"
                />
                <Textarea
                  value={editText}
                  onChange={(e) => onEditText(e.target.value)}
                  placeholder="Override the passage text before displaying. Leave blank to use the stored translation text."
                />
              </div>
            ) : null}

            <div className="flex flex-wrap items-center gap-2">
              <Button
                variant="success"
                size="sm"
                disabled={working}
                onClick={() =>
                  onDisplay(
                    editing && editText.trim().length > 0
                      ? editText
                      : undefined,
                  )
                }
              >
                <MonitorPlay className="size-3.5" />
                Display
              </Button>
              {editing ? (
                <Button variant="ghost" size="sm" onClick={onCancelEdit}>
                  <X className="size-3.5" />
                  Cancel edit
                </Button>
              ) : (
                <Button variant="outline" size="sm" onClick={onStartEdit}>
                  <Pencil className="size-3.5" />
                  Edit
                </Button>
              )}
              <Button variant="ghost" size="sm" onClick={onIgnore}>
                <X className="size-3.5" />
                Ignore
              </Button>
            </div>
          </>
        )}
      </CardContent>
    </Card>
  );
}
