import { describe, expect, it } from "vitest";

import {
  DEFAULT_FONT_FAMILY,
  FONT_OPTIONS,
  isLightColor,
  resolveFontFamily,
} from "./fonts";

describe("font catalog", () => {
  it("offers Creato Display first, because it is Selah's own typeface", () => {
    expect(FONT_OPTIONS[0].family).toBe(DEFAULT_FONT_FAMILY);
  });

  it("lists every bundled family exactly once", () => {
    const families = FONT_OPTIONS.map((option) => option.family);
    expect(new Set(families).size).toBe(families.length);
    expect(families).toContain("Inter");
    expect(families).toContain("Lora");
    expect(families).toContain("Oswald");
    expect(families).toContain("JetBrains Mono");
  });

  it("falls back when a stored family is unknown", () => {
    // Settings are a free-form document, so a hand-edited or stale value must
    // never leave the projector without a usable font.
    expect(resolveFontFamily("Comic Sans MS")).toBe(DEFAULT_FONT_FAMILY);
    expect(resolveFontFamily(undefined)).toBe(DEFAULT_FONT_FAMILY);
    expect(resolveFontFamily("")).toBe(DEFAULT_FONT_FAMILY);
    expect(resolveFontFamily("Lora")).toBe("Lora");
  });
});

describe("background contrast", () => {
  it("treats a dark projector background as dark", () => {
    expect(isLightColor("#000000")).toBe(false);
    expect(isLightColor("#141B2E")).toBe(false);
  });

  it("treats a pale background as light, so the lettering turns dark", () => {
    expect(isLightColor("#FFFFFF")).toBe(true);
    expect(isLightColor("#F4EDE0")).toBe(true);
  });

  it("is not fooled by a malformed colour", () => {
    // A value like this cannot be drawn, so it is treated as the default dark.
    expect(isLightColor("#06666")).toBe(false);
    expect(isLightColor("not a colour")).toBe(false);
  });
});
