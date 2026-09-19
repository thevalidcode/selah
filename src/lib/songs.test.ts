import { describe, expect, it } from "vitest";

import { MAX_VERSE_CHARS, splitIntoVerses } from "./songs";

describe("pasting a paragraph as verses", () => {
  it("splits numbered verses however the numbers are written", () => {
    const pasted = `1. Amazing grace, how sweet the sound
2) That saved a wretch like me
[3] I once was lost but now am found`;

    const verses = splitIntoVerses(pasted);
    expect(verses).toHaveLength(3);
    expect(verses[0]).toEqual({
      label: "Verse 1",
      text: "Amazing grace, how sweet the sound",
    });
    expect(verses[2].text).toBe("I once was lost but now am found");
  });

  it("splits on blank lines when there are no numbers", () => {
    const verses = splitIntoVerses(
      "Amazing grace, how sweet the sound\n\nThat saved a wretch like me",
    );
    expect(verses.map((verse) => verse.text)).toEqual([
      "Amazing grace, how sweet the sound",
      "That saved a wretch like me",
    ]);
  });

  it("treats one verse per line as separate verses", () => {
    const verses = splitIntoVerses("Holy, holy, holy\nLord God almighty");
    expect(verses).toHaveLength(2);
    expect(verses[1].label).toBe("Verse 2");
  });

  it("cuts one long paragraph into sentence-sized verses", () => {
    // The case this feature exists for: a single pasted block with no verse
    // marks at all.
    const pasted =
      "Amazing grace how sweet the sound that saved a wretch like me. " +
      "I once was lost but now am found, was blind but now I see. " +
      "Twas grace that taught my heart to fear, and grace my fears relieved.";

    const verses = splitIntoVerses(pasted);
    expect(verses.length).toBeGreaterThan(1);
    for (const verse of verses) {
      expect(verse.text.length).toBeLessThanOrEqual(MAX_VERSE_CHARS);
      expect(verse.text.startsWith(".")).toBe(false);
    }
    expect(verses[0].text.endsWith("like me.")).toBe(true);
  });

  it("never leaves a verse longer than one screen", () => {
    const words = Array.from({ length: 80 }, (_, index) => `word${index}`).join(" ");
    const verses = splitIntoVerses(`1. ${words}\n2. short verse`);
    for (const verse of verses) {
      expect(verse.text.length).toBeLessThanOrEqual(MAX_VERSE_CHARS);
    }
    expect(verses[verses.length - 1].text).toBe("short verse");
  });

  it("labels verses in order, starting at one", () => {
    const verses = splitIntoVerses("one\n\ntwo\n\nthree");
    expect(verses.map((verse) => verse.label)).toEqual([
      "Verse 1",
      "Verse 2",
      "Verse 3",
    ]);
  });

  it("reports nothing for an empty paste", () => {
    expect(splitIntoVerses("")).toEqual([]);
    expect(splitIntoVerses("   \n\n  ")).toEqual([]);
  });

  it("does not mistake ordinary numbers for verse numbers", () => {
    // A sentence containing numbers must stay one verse, not be cut in three.
    const verses = splitIntoVerses(
      "We sang 3 songs in 2024 and it was wonderful.",
    );
    expect(verses).toHaveLength(1);
    expect(verses[0].text).toBe("We sang 3 songs in 2024 and it was wonderful.");
  });

  it("trims stray whitespace and ignores blank pieces", () => {
    const verses = splitIntoVerses("1.   first   verse  \n\n\n2.  second verse ");
    expect(verses.map((verse) => verse.text)).toEqual([
      "first verse",
      "second verse",
    ]);
  });
});
