import { describe, expect, it } from "vitest";

import {
  buildAiPrompt,
  parseVerseArray,
  validateSingleVerse,
  VERSE_ARRAY_EXAMPLE,
} from "./customVerses";

describe("the prompt to hand an AI agent", () => {
  it("names the chapter and demands nothing but the array", () => {
    const prompt = buildAiPrompt("Genesis", 1);
    expect(prompt).toContain("Return every verse of Genesis 1");
    expect(prompt).toContain("nothing else");
    expect(prompt).toContain("no code fences");
    expect(prompt).toContain("book, chapter, verse, text");
    // The format is shown, so the reply can be pasted straight back.
    expect(prompt).toContain(VERSE_ARRAY_EXAMPLE);
  });

  it("asks for the translation that was chosen", () => {
    // The words differ between versions, so the version has to be in the ask.
    const prompt = buildAiPrompt("John", 3, "The Message");
    expect(prompt).toContain("in the The Message");
    expect(buildAiPrompt("John", 3)).not.toContain("in the");
  });
});

describe("pasting a whole chapter", () => {
  const chapter = JSON.stringify([
    { book: "John", chapter: 3, verse: 16, text: "For God so loved the world" },
    { book: "John", chapter: 3, verse: 17, text: "For God sent not his Son" },
  ]);

  it("accepts the documented array and orders it", () => {
    const result = parseVerseArray(
      JSON.stringify([
        { book: "John", chapter: 3, verse: 17, text: "second" },
        { book: "John", chapter: 3, verse: 16, text: "first" },
      ]),
    );
    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value.book).toBe("John");
      expect(result.value.chapter).toBe(3);
      expect(result.value.verses.map((verse) => verse.verse)).toEqual([16, 17]);
      expect(result.value.missing).toEqual([]);
    }
  });

  it("tolerates a code fence around the array", () => {
    // Agents like to wrap replies in ```json — refusing that helps nobody.
    const result = parseVerseArray("```json\n" + chapter + "\n```");
    expect(result.ok).toBe(true);
  });

  it("reports gaps rather than pretending the chapter is complete", () => {
    const result = parseVerseArray(
      JSON.stringify([
        { book: "John", chapter: 3, verse: 16, text: "one" },
        { book: "John", chapter: 3, verse: 19, text: "four" },
      ]),
    );
    expect(result.ok).toBe(true);
    if (result.ok) {
      expect(result.value.missing).toEqual([17, 18]);
    }
  });

  it("rejects JSON that is not an array", () => {
    const result = parseVerseArray('{"book":"John"}');
    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.error).toContain("not an array");
    }
  });

  it("explains that unreadable text is not JSON", () => {
    const result = parseVerseArray("John 3:16 For God so loved");
    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.error).toContain("not readable as JSON");
    }
  });

  it("rejects extra fields", () => {
    const result = parseVerseArray(
      JSON.stringify([
        { book: "John", chapter: 3, verse: 16, text: "words", note: "hello" },
      ]),
    );
    expect(result.ok).toBe(false);
    if (!result.ok) {
      expect(result.error).toContain("extra fields (note)");
    }
  });

  it("rejects a paste that mixes books or chapters", () => {
    const mixedBooks = parseVerseArray(
      JSON.stringify([
        { book: "John", chapter: 3, verse: 16, text: "one" },
        { book: "Luke", chapter: 3, verse: 17, text: "two" },
      ]),
    );
    expect(mixedBooks.ok).toBe(false);
    if (!mixedBooks.ok) {
      expect(mixedBooks.error).toContain("one chapter at a time");
    }

    const mixedChapters = parseVerseArray(
      JSON.stringify([
        { book: "John", chapter: 3, verse: 16, text: "one" },
        { book: "John", chapter: 4, verse: 1, text: "two" },
      ]),
    );
    expect(mixedChapters.ok).toBe(false);
  });

  it("rejects impossible numbers, duplicate verses and empty words", () => {
    const impossible = parseVerseArray(
      JSON.stringify([{ book: "John", chapter: 3, verse: "sixteen", text: "x" }]),
    );
    expect(impossible.ok).toBe(false);

    const duplicate = parseVerseArray(
      JSON.stringify([
        { book: "John", chapter: 3, verse: 16, text: "one" },
        { book: "John", chapter: 3, verse: 16, text: "again" },
      ]),
    );
    expect(duplicate.ok).toBe(false);
    if (!duplicate.ok) {
      expect(duplicate.error).toContain("repeats verse 16");
    }

    const empty = parseVerseArray(
      JSON.stringify([{ book: "John", chapter: 3, verse: 16, text: "   " }]),
    );
    expect(empty.ok).toBe(false);
    if (!empty.ok) {
      expect(empty.error).toContain("no words");
    }
  });

  it("rejects an empty paste and an empty array", () => {
    expect(parseVerseArray("   ").ok).toBe(false);
    expect(parseVerseArray("[]").ok).toBe(false);
  });
});

describe("typing a single verse", () => {
  it("accepts the verse message on its own", () => {
    const result = validateSingleVerse("  Jesus   wept. ");
    expect(result.ok).toBe(true);
    expect(result.text).toBe("Jesus wept.");
  });

  it("refuses a line that still carries its reference", () => {
    // The most common mistake: the whole line pasted, reference and all.
    for (const input of [
      "John 11:35 — Jesus wept",
      "Ps 23:1 The Lord is my shepherd",
      "1 Cor 13:4 Love is patient",
    ]) {
      const result = validateSingleVerse(input);
      expect(result.ok, input).toBe(false);
      expect(result.error).toContain("reference");
    }
  });

  it("refuses labels, code and wordless input", () => {
    expect(validateSingleVerse("verse: Jesus wept").ok).toBe(false);
    expect(validateSingleVerse('[{"book":"John"}]').ok).toBe(false);
    expect(validateSingleVerse("12345").ok).toBe(false);
    expect(validateSingleVerse("").ok).toBe(false);
  });
});
