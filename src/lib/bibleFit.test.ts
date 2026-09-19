import { describe, expect, it } from "vitest";

import { estimateFit } from "./bibleFit";

const screen = { width: 1920, height: 1080 };

describe("will the passage fit the screen", () => {
  it("says nothing when the screen size is unknown", () => {
    // Warning about a screen Selah cannot see would be guesswork.
    expect(
      estimateFit({
        text: "For God so loved the world",
        heading: "John 3:16",
        headingSize: 83,
        textSize: 64,
        screen: null,
      }),
    ).toBeNull();
  });

  it("is happy with a short verse at a normal size", () => {
    const fit = estimateFit({
      text: "Jesus wept.",
      heading: "John 11:35",
      headingSize: 83,
      textSize: 64,
      screen,
    });
    expect(fit?.overflows).toBe(false);
    expect(fit?.warning).toBeNull();
    expect(fit?.lines).toBe(1);
  });

  it("warns when a whole chapter is asked to fit", () => {
    const chapter = Array.from(
      { length: 31 },
      (_, index) =>
        `And verse ${index + 1} of this chapter says a good many words indeed.`,
    ).join(" ");

    const fit = estimateFit({
      text: chapter,
      heading: "Genesis 1",
      headingSize: 83,
      textSize: 64,
      screen,
    });
    expect(fit?.overflows).toBe(true);
    expect(fit?.warning).toContain("too long for the screen");
    expect(fit?.requiredHeight).toBeGreaterThan(fit!.availableHeight);
  });

  it("accounts for the branding overlay taking space", () => {
    const text = "The Lord is my shepherd; I shall not want.".repeat(6);
    const free = estimateFit({
      text,
      heading: "Psalm 23",
      headingSize: 83,
      textSize: 64,
      screen,
    });
    const branded = estimateFit({
      text,
      heading: "Psalm 23",
      headingSize: 83,
      textSize: 64,
      screen,
      brandingHeight: 400,
    });
    expect(branded!.availableHeight).toBe(free!.availableHeight - 400);
  });

  it("notices that a smaller size fits where a larger one does not", () => {
    const text = "Amazing grace how sweet the sound that saved a wretch like me";
    const big = estimateFit({
      text,
      heading: "Hymn",
      headingSize: 200,
      textSize: 180,
      screen,
    });
    const small = estimateFit({
      text,
      heading: "Hymn",
      headingSize: 60,
      textSize: 40,
      screen,
    });
    expect(big?.overflows).toBe(true);
    expect(small?.overflows).toBe(false);
  });

  it("counts explicit line breaks as separate lines", () => {
    const fit = estimateFit({
      text: "line one\nline two\nline three",
      heading: "",
      headingSize: 83,
      textSize: 64,
      screen,
    });
    expect(fit?.lines).toBe(3);
  });
});
