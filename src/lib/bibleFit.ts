/**
 * Will this passage fit on the screen?
 *
 * Projected verses are the one place where too much content is invisible until
 * it is already on the wall — and by then the congregation is reading it. The
 * projector page scrolls nothing and clips nothing, so a verse that is too long
 * simply runs off the bottom.
 *
 * Nothing here talks to a screen; it is arithmetic on the numbers the Bible
 * screen already has (the chosen screen's pixel size and the text sizes). It is
 * therefore only an estimate — which is why it *warns* rather than blocking: an
 * operator who knows their room can still show it.
 */

/** Fraction of the screen's width the projector uses (it pads 6vw each side). */
const USABLE_WIDTH = 0.88;
/** Fraction of the screen's height available for text (5vh padding each side). */
const USABLE_HEIGHT = 0.9;
/** Line height for projected words, mirroring `.pres-content__text`. */
const TEXT_LINE_HEIGHT = 1.35;
/** Heading line height, mirroring `.pres-content__heading`. */
const HEADING_LINE_HEIGHT = 1.3;
/** How wide an average character is, relative to the font size. */
const AVERAGE_CHAR_WIDTH = 0.52;
/** Longest line the projector will draw, mirroring `max-width: 62ch`. */
const MAX_LINE_CHARS = 62;

export interface FitInput {
  /** The passage words, verses joined by spaces, as the projector receives them. */
  text: string;
  heading: string;
  /** Heading size in CSS pixels. */
  headingSize: number;
  /** Verse size in CSS pixels. */
  textSize: number;
  /** The chosen screen's size in pixels, when it is known. */
  screen: { width: number; height: number } | null;
  /** Space to leave free for the operator's branding overlay, in pixels. */
  brandingHeight?: number;
}

export interface FitEstimate {
  /** Estimated lines of verse text once wrapped. */
  lines: number;
  /** Space the heading and words need, in pixels. */
  requiredHeight: number;
  /** Space the screen actually offers, in pixels. */
  availableHeight: number;
  /** True when the estimate says it will not fit. */
  overflows: boolean;
  /** Plain-language warning, or `null` when there is nothing to say. */
  warning: string | null;
}

/**
 * Estimates whether a passage fits the chosen screen.
 *
 * With no screen known (no displays reported yet) it cannot know, so it says
 * nothing rather than crying wolf.
 */
export function estimateFit(input: FitInput): FitEstimate | null {
  if (!input.screen) {
    return null;
  }

  const usableWidth = input.screen.width * USABLE_WIDTH;
  const availableHeight =
    input.screen.height * USABLE_HEIGHT - (input.brandingHeight ?? 0);

  const textCharsPerLine = Math.max(
    10,
    Math.min(
      MAX_LINE_CHARS,
      Math.floor(usableWidth / (input.textSize * AVERAGE_CHAR_WIDTH)),
    ),
  );
  const headingCharsPerLine = Math.max(
    8,
    Math.floor(usableWidth / (input.headingSize * AVERAGE_CHAR_WIDTH)),
  );

  const lines = countLines(input.text, textCharsPerLine);
  const headingLines = countLines(input.heading, headingCharsPerLine);

  const requiredHeight =
    headingLines * input.headingSize * HEADING_LINE_HEIGHT +
    lines * input.textSize * TEXT_LINE_HEIGHT +
    // The label and counter sit outside the flowing text; a little slack keeps
    // the estimate from being optimistic.
    input.textSize * 0.4;

  const overflows = requiredHeight > availableHeight;
  if (!overflows) {
    return {
      lines,
      requiredHeight: Math.round(requiredHeight),
      availableHeight: Math.round(availableHeight),
      overflows: false,
      warning: null,
    };
  }

  return {
    lines,
    requiredHeight: Math.round(requiredHeight),
    availableHeight: Math.round(availableHeight),
    overflows: true,
    warning:
      `This looks too long for the screen: about ${lines} lines at ` +
      `${input.textSize}px need roughly ${Math.round(requiredHeight)}px, and the ` +
      `screen gives about ${Math.round(availableHeight)}px. Show fewer verses, or ` +
      `bring the text size down.`,
  };
}

/**
 * Lines a run of words occupies once wrapped, honouring explicit line breaks.
 */
function countLines(text: string, charsPerLine: number): number {
  const trimmed = text.trim();
  if (trimmed.length === 0) {
    return 0;
  }

  return trimmed.split(/\n+/).reduce((total, paragraph) => {
    const words = paragraph.trim().split(/\s+/).filter(Boolean);
    if (words.length === 0) {
      return total;
    }

    let lines = 1;
    let current = 0;
    for (const word of words) {
      const next = current === 0 ? word.length : current + word.length + 1;
      if (next > charsPerLine) {
        lines += 1;
        current = word.length;
      } else {
        current = next;
      }
    }
    return total + lines;
  }, 0);
}
