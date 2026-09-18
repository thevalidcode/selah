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
        <CardTitle>We heard a verse</CardTitle>
        {detection?.confidence !== undefined ? (
          <Badge variant="success">
            {Math.round(detection.confidence * 100)}% sure
          </Badge>
        ) : null}
      </CardHeader>
      <CardContent className="space-y-3">
        {!detection ? (
          <EmptyHint>
            Nothing yet. When someone says a Bible verse, it appears here for
            you to approve — nothing goes on screen on its own.
          </EmptyHint>
        ) : (
          <>
            <div className="rounded-lg border border-brand/30 bg-brand/5 p-4">
              <div className="flex items-center gap-2 text-[10px] font-semibold tracking-[0.16em] text-brand uppercase">
                <BookOpen className="size-3" />
                Bible verse
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
                  placeholder="What to show at the top (for example, John 3:16)"
                />
                <Textarea
                  value={editText}
                  onChange={(e) => onEditText(e.target.value)}
                  placeholder="Type your own words, or leave this blank to use the Bible text you have."
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
                Show on screen
              </Button>
              {editing ? (
                <Button variant="ghost" size="sm" onClick={onCancelEdit}>
                  <X className="size-3.5" />
                  Cancel
                </Button>
              ) : (
                <Button variant="outline" size="sm" onClick={onStartEdit}>
                  <Pencil className="size-3.5" />
                  Edit words
                </Button>
              )}
              <Button variant="ghost" size="sm" onClick={onIgnore}>
                <X className="size-3.5" />
                Not now
              </Button>
            </div>
          </>
        )}
      </CardContent>
    </Card>
  );
}
