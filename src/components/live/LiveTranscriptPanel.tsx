import { ArrowRight, Eraser, Radio } from "lucide-react";

import { EmptyHint } from "@/components/PageHeader";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import type { LiveTranscriptEntry } from "@/hooks/useLiveSession";

/**
 * Live transcript feed.
 *
 * Shows the most recent utterance prominently (this is what the operator
 * actually watches) with a scrollable history beneath it.
 */
export default function LiveTranscriptPanel({
  feed,
  listening,
  onClear,
}: {
  feed: LiveTranscriptEntry[];
  listening: boolean;
  onClear: () => void;
}) {
  const latest = feed[feed.length - 1];

  return (
    <Card className="min-h-[22rem]">
      <CardHeader>
        <CardTitle>What Selah hears</CardTitle>
        <div className="flex items-center gap-2">
          <Badge variant={listening ? "success" : "muted"}>
            <Radio className="size-3" />
            {listening ? "listening" : "paused"}
          </Badge>
          <Button
            variant="ghost"
            size="sm"
            onClick={onClear}
            disabled={feed.length === 0}
          >
            <Eraser className="size-3.5" />
            Clear list
          </Button>
        </div>
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="rounded-lg border border-border/60 bg-background/60 p-4">
          {latest && !latest.empty ? (
            <p className="selectable text-lg leading-relaxed">
              “{latest.text}”
            </p>
          ) : (
            <EmptyHint>
              {listening
                ? "Waiting for someone to speak…"
                : "Press Listen to let Selah hear the room."}
            </EmptyHint>
          )}
        </div>

        <Separator />

        <div>
          <div className="mb-2 flex items-center justify-between">
            <span className="text-[10px] font-semibold tracking-[0.14em] text-muted-foreground uppercase">
              Earlier
            </span>
            <span className="text-[10px] text-muted-foreground">
              {feed.length} line{feed.length === 1 ? "" : "s"}
            </span>
          </div>
          <ScrollArea className="h-40">
            {feed.length === 0 ? (
              <EmptyHint>Nothing heard yet.</EmptyHint>
            ) : (
              <ul className="space-y-2 pr-3">
                {[...feed].reverse().map((entry) => (
                  <li
                    key={entry.id}
                    className="selectable flex items-start gap-2 text-sm text-muted-foreground"
                  >
                    <ArrowRight className="mt-0.5 size-3 shrink-0 opacity-50" />
                    <span className={entry.empty ? "italic" : undefined}>
                      {entry.empty ? "(nothing was heard)" : entry.text}
                    </span>
                  </li>
                ))}
              </ul>
            )}
          </ScrollArea>
        </div>
      </CardContent>
    </Card>
  );
}
