import { Eraser } from "lucide-react";

import { EmptyHint } from "@/components/PageHeader";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import type { PresentationItem } from "@/types";

/** What is on the projector right now, plus a one-click clear. */
export default function NowProjectingPanel({
  current,
  projected,
  working,
  onClear,
}: {
  current: PresentationItem | null;
  projected: boolean;
  working: boolean;
  onClear: () => void;
}) {
  const text =
    current?.payload.kind === "scripture" || current?.payload.kind === "text"
      ? current.payload.text
      : current?.payload.kind === "media"
        ? current.payload.path
        : undefined;

  return (
    <Card>
      <CardHeader>
        <CardTitle>Presentation</CardTitle>
        <div className="flex items-center gap-2">
          <Badge variant={projected ? "success" : "muted"}>
            {projected ? "display open" : "display closed"}
          </Badge>
          <Button
            variant="ghost"
            size="sm"
            onClick={onClear}
            disabled={working || !current}
          >
            <Eraser className="size-3.5" />
            Clear
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        {!current ? (
          <EmptyHint>Nothing currently displayed.</EmptyHint>
        ) : (
          <div className="space-y-2">
            <p className="text-[10px] font-semibold tracking-[0.16em] text-muted-foreground uppercase">
              {current.contentType}
            </p>
            <p className="text-base font-medium">{current.title}</p>
            <p className="selectable line-clamp-4 text-sm text-muted-foreground">
              {text}
            </p>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
