import type { PresentationItemRecord } from "@/types";

/**
 * Reading a saved presentation item back out of its stored JSON.
 *
 * `presentation_items` keeps its content as JSON so a new content kind never
 * needs a schema change. The cost is that the operator-facing pieces — the
 * heading they typed, the words, and what to put back in the edit form — all
 * have to be unpacked here rather than read from columns.
 */

/** An item open in the composer, ready to be saved again. */
export interface EditingItem {
  id: string;
  /** `text`, `scripture` or `announcement` — the projector styles by kind. */
  typeName: string;
  heading: string;
  body: string;
}

/** Reads a stored row back into the edit form. */
export function editingItemFrom(record: PresentationItemRecord): EditingItem {
  let heading = "";
  let body = "";
  try {
    const parsed = JSON.parse(record.payload) as Record<string, unknown>;
    const text = (value: unknown) => (typeof value === "string" ? value : "");
    heading =
      text(parsed.heading) || text(parsed.reference) || text(parsed.title);
    body = text(parsed.text);
  } catch {
    // An unreadable payload is edited from scratch rather than silently lost.
  }

  return { id: record.id, typeName: record.typeName, heading, body };
}

/**
 * A one-line summary of a stored item for the items table.
 *
 * Items are stored as JSON, so showing the raw payload made the heading the
 * operator typed invisible — which is exactly what this unpacks.
 */
export function summariseItem(payload: string): {
  heading: string;
  detail: string;
} {
  try {
    const parsed = JSON.parse(payload) as Record<string, unknown>;
    const text = (value: unknown) =>
      typeof value === "string" ? value.trim() : "";

    const heading =
      text(parsed.heading) || text(parsed.reference) || text(parsed.title);
    const detail = text(parsed.text) || text(parsed.path) || payload;

    return {
      heading: heading || "No heading",
      detail: detail.replace(/\s+/g, " ").slice(0, 120),
    };
  } catch {
    return { heading: "Unreadable item", detail: payload.slice(0, 120) };
  }
}
