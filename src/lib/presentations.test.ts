import { describe, expect, it } from "vitest";

import { editingItemFrom, summariseItem } from "./presentations";
import type { PresentationItemRecord } from "@/types";

function record(
  typeName: string,
  payload: string,
): PresentationItemRecord {
  return {
    id: "item-1",
    presentationId: "deck-1",
    typeName,
    position: 1,
    payload,
  };
}

describe("reading a saved item back for editing", () => {
  it("brings back the heading and words that were typed", () => {
    const item = editingItemFrom(
      record(
        "announcement",
        JSON.stringify({ kind: "text", heading: "Notices", text: "Tea after" }),
      ),
    );
    expect(item).toEqual({
      id: "item-1",
      typeName: "announcement",
      heading: "Notices",
      body: "Tea after",
    });
  });

  it("falls back to the reference for a verse saved without a heading", () => {
    // The composer writes the reference into `reference`, not `heading`.
    const item = editingItemFrom(
      record(
        "scripture",
        JSON.stringify({
          kind: "scripture",
          reference: "John 3:16",
          translation: "WEB",
          text: "For God so loved",
        }),
      ),
    );
    expect(item.heading).toBe("John 3:16");
    expect(item.body).toBe("For God so loved");
  });

  it("offers an empty form rather than throwing on unreadable content", () => {
    const item = editingItemFrom(record("text", "not json"));
    expect(item.heading).toBe("");
    expect(item.body).toBe("");
    // The type is still carried so an edit does not change what the row is.
    expect(item.typeName).toBe("text");
  });
});

describe("summarising a stored item for the list", () => {
  it("shows the heading the operator typed", () => {
    const summary = summariseItem(
      JSON.stringify({ kind: "text", heading: "Welcome", text: "Good morning" }),
    );
    expect(summary.heading).toBe("Welcome");
    expect(summary.detail).toBe("Good morning");
  });

  it("flattens long words into one short line", () => {
    const summary = summariseItem(
      JSON.stringify({
        kind: "text",
        heading: "Long",
        text: "word ".repeat(100),
      }),
    );
    expect(summary.detail.length).toBeLessThanOrEqual(120);
    expect(summary.detail).not.toContain("\n");
  });

  it("falls back to the file name for media with no caption", () => {
    const summary = summariseItem(
      JSON.stringify({ kind: "media", path: "/tmp/baptism.png" }),
    );
    expect(summary.heading).toBe("No heading");
    expect(summary.detail).toBe("/tmp/baptism.png");
  });

  it("reports unreadable content instead of pretending it is empty", () => {
    const summary = summariseItem("{not json");
    expect(summary.heading).toBe("Unreadable item");
    expect(summary.detail).toContain("not json");
  });
});
