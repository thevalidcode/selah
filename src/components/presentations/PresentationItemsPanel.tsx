import { ChevronLeft, ChevronRight, Pencil, Play, Trash2 } from "lucide-react";

import { EmptyHint, Panel } from "@/components/PageHeader";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { friendlyContentType } from "@/lib/content";
import { summariseItem } from "@/lib/presentations";
import type { Presentation } from "@/types";

/**
 * Ordered item list for the selected presentation.
 *
 * The play buttons here are what actually put saved content on the screen:
 * saved rows are stored as JSON, so they have to be rebuilt into something
 * projectable before they can be shown. Edit puts one back into the composer
 * without losing its place in the list.
 */
export default function PresentationItemsPanel({
  presentation,
  busy,
  editingItemId,
  onDelete,
  onEditItem,
  onRemoveItem,
  onShowItem,
  onShowAll,
  onNext,
  onPrevious,
}: {
  presentation: Presentation | null;
  busy: boolean;
  /** The row currently open in the composer, if any. */
  editingItemId?: string;
  onDelete: () => void;
  onEditItem: (itemId: string) => void;
  onRemoveItem: (itemId: string) => void | Promise<void>;
  onShowItem: (itemId: string) => void | Promise<void>;
  onShowAll: () => void | Promise<void>;
  onNext: () => void | Promise<void>;
  onPrevious: () => void | Promise<void>;
}) {
  const items = presentation?.items ?? [];
  const hasItems = items.length > 0;

  return (
    <Panel
      title={presentation ? presentation.name : "Presentation"}
      actions={
        presentation ? (
          <div className="flex items-center gap-1">
            <Button
              variant="ghost"
              size="sm"
              disabled={busy}
              onClick={() => void onPrevious()}
            >
              <ChevronLeft className="size-3.5" />
              Back
            </Button>
            <Button
              variant="ghost"
              size="sm"
              disabled={busy}
              onClick={() => void onNext()}
            >
              Next
              <ChevronRight className="size-3.5" />
            </Button>
            <Button
              variant="success"
              size="sm"
              disabled={busy || !hasItems}
              onClick={() => void onShowAll()}
            >
              <Play className="size-3.5" />
              Show on screen
            </Button>
            <Button variant="ghost" size="sm" disabled={busy} onClick={onDelete}>
              <Trash2 className="size-3.5" />
              Delete
            </Button>
          </div>
        ) : null
      }
    >
      {!presentation ? (
        <EmptyHint>
          Choose a presentation on the left to see what is in it, or create a
          new one.
        </EmptyHint>
      ) : !hasItems ? (
        <EmptyHint>
          Nothing here yet. Add something using the box below.
        </EmptyHint>
      ) : (
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="w-12">#</TableHead>
              <TableHead className="w-28">Kind</TableHead>
              <TableHead>Content</TableHead>
              <TableHead className="w-24 text-right">·</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {items.map((item) => (
              <TableRow
                key={item.id}
                className={editingItemId === item.id ? "bg-accent/60" : undefined}
              >
                <TableCell className="font-mono text-xs text-muted-foreground">
                  {item.position}
                </TableCell>
                <TableCell>
                  {/*
                    Notices are labelled in their own colour: the whole point of
                    the kind is that an operator can tell announcements from
                    Bible verses at a glance.
                  */}
                  <Badge
                    variant={item.typeName === "announcement" ? "warning" : "muted"}
                  >
                    {friendlyContentType(item.typeName)}
                  </Badge>
                </TableCell>
                <TableCell className="max-w-md">
                  <span className="block truncate text-sm">
                    {summariseItem(item.payload).heading}
                  </span>
                  <span className="block truncate text-xs text-muted-foreground">
                    {summariseItem(item.payload).detail}
                  </span>
                </TableCell>
                <TableCell className="text-right">
                  <div className="flex items-center justify-end gap-1">
                    <Button
                      variant="ghost"
                      size="icon"
                      aria-label="Edit this item"
                      disabled={busy}
                      onClick={() => onEditItem(item.id)}
                    >
                      <Pencil className="size-4" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      aria-label="Show this on screen"
                      disabled={busy}
                      onClick={() => void onShowItem(item.id)}
                    >
                      <Play className="size-4" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      aria-label="Remove item"
                      disabled={busy}
                      onClick={() => void onRemoveItem(item.id)}
                    >
                      <Trash2 className="size-4" />
                    </Button>
                  </div>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      )}
    </Panel>
  );
}
