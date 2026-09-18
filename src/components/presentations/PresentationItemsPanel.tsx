import { Trash2 } from "lucide-react";

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
import type { Presentation } from "@/types";

/** Ordered item list for the selected presentation. */
export default function PresentationItemsPanel({
  presentation,
  busy,
  onDelete,
  onRemoveItem,
}: {
  presentation: Presentation | null;
  busy: boolean;
  onDelete: () => void;
  onRemoveItem: (itemId: string) => void | Promise<void>;
}) {
  const items = presentation?.items ?? [];

  return (
    <Panel
      title={presentation ? presentation.name : "Presentation"}
      actions={
        presentation ? (
          <Button
            variant="ghost"
            size="sm"
            disabled={busy}
            onClick={onDelete}
          >
            <Trash2 className="size-3.5" />
            Delete
          </Button>
        ) : null
      }
    >
      {!presentation ? (
        <EmptyHint>
          Select a presentation to see its items, or create a new one.
        </EmptyHint>
      ) : items.length === 0 ? (
        <EmptyHint>This presentation has no items yet. Add one below.</EmptyHint>
      ) : (
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="w-12">#</TableHead>
              <TableHead className="w-28">Type</TableHead>
              <TableHead>Payload</TableHead>
              <TableHead className="w-16 text-right">·</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {items.map((item) => (
              <TableRow key={item.id}>
                <TableCell className="font-mono text-xs text-muted-foreground">
                  {item.position}
                </TableCell>
                <TableCell>
                  <Badge variant="muted">{item.typeName}</Badge>
                </TableCell>
                <TableCell className="max-w-md">
                  <span className="selectable block truncate font-mono text-xs text-muted-foreground">
                    {item.payload}
                  </span>
                </TableCell>
                <TableCell className="text-right">
                  <Button
                    variant="ghost"
                    size="icon"
                    aria-label="Remove item"
                    disabled={busy}
                    onClick={() => void onRemoveItem(item.id)}
                  >
                    <Trash2 className="size-4" />
                  </Button>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      )}
    </Panel>
  );
}
