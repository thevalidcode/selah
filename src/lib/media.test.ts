import { describe, expect, it } from "vitest";

import { mediaKindFromPath } from "./media";

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
