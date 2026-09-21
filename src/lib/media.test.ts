import { describe, expect, it } from "vitest";

import {
  clipEndAction,
  clipFromMetadata,
  describeClip,
  formatClipTime,
  mediaKindFromPath,
} from "./media";
import type { MediaClip } from "@/types";

describe("media kind from a file name", () => {
  it("recognises pictures, video and sound", () => {
    expect(mediaKindFromPath("/tmp/slide.png")).toBe("image");
    expect(mediaKindFromPath("/tmp/CLIP.MP4")).toBe("video");
    expect(mediaKindFromPath("/tmp/hymn.wav")).toBe("audio");
  });

  it("assumes a picture for anything it does not recognise", () => {
    // Only reached for files registered before the kind was stored; an <img>
    // that fails to render is a clearer failure than a black video player.
    expect(mediaKindFromPath("/tmp/mystery")).toBe("image");
  });
});

describe("a chosen time range", () => {
  const range: MediaClip = { startMs: 6_000, endMs: 30_000, repeat: false };

  it("is read back out of a media record", () => {
    expect(
      clipFromMetadata({ size: 10, kind: "video", clip: range }),
    ).toEqual(range);
  });

  it("is ignored when the metadata is not what Selah wrote", () => {
    // Metadata is free-form JSON, so a hand-edited or half-written value must
    // not break the screen that read it.
    expect(clipFromMetadata(undefined)).toBeUndefined();
    expect(clipFromMetadata({})).toBeUndefined();
    expect(clipFromMetadata({ clip: "6-30" })).toBeUndefined();
    expect(clipFromMetadata({ clip: { endMs: 30_000 } })).toBeUndefined();
  });

  it("is described in minutes and seconds", () => {
    expect(formatClipTime(6_000)).toBe("0:06");
    expect(formatClipTime(90_500)).toBe("1:31");
    expect(describeClip(range)).toBe("0:06 – 0:30");
    expect(describeClip({ ...range, repeat: true })).toBe("0:06 – 0:30 · repeats");
    expect(describeClip({ startMs: 6_000, repeat: false })).toBe("0:06 – the end");
    // A whole file has nothing worth saying on a card.
    expect(describeClip(undefined)).toBeNull();
    expect(describeClip({ startMs: 0, repeat: true })).toBeNull();
  });

  it("tells the projector when to go back and when to stop", () => {
    // Still inside the range: keep going.
    expect(clipEndAction(6_000, range, 60_000)).toBe("continue");
    expect(clipEndAction(29_999, range, 60_000)).toBe("continue");

    // Reached the end of the range: stop, or start again when asked.
    expect(clipEndAction(30_000, range, 60_000)).toBe("end");
    expect(clipEndAction(31_000, { ...range, repeat: true }, 60_000)).toBe(
      "restart",
    );

    // No end chosen: the file's own end decides, and the player reports that
    // itself — so this never answers "end" for an open-ended range.
    const openEnded: MediaClip = { startMs: 6_000, repeat: false };
    expect(clipEndAction(6_000, openEnded, 60_000)).toBe("continue");
    expect(clipEndAction(59_000, openEnded, 60_000)).toBe("continue");
    expect(clipEndAction(60_000, { ...openEnded, repeat: true }, 60_000)).toBe(
      "restart",
    );
  });
});
