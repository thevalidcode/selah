/**
 * Bundled typefaces.
 *
 * Every family here ships with Selah (see `src/fonts/README.md`), because the
 * application has to work on a machine with no internet connection. Adding a
 * family means dropping the file in `src/fonts`, adding an `@font-face` rule in
 * `index.css`, and listing it here.
 */
export interface FontOption {
  /** Value stored in settings and matched against `@font-face`. */
  family: string;
  /** Name shown in the picker. */
  label: string;
  /** What the font is good for, shown under the name. */
  description: string;
  /** Bundled and ready to use (as opposed to a generic system family). */
  bundled: boolean;
}

export const FONT_OPTIONS: FontOption[] = [
  {
    family: "Creato Display",
    label: "Creato Display",
    description: "Selah's own typeface — clean and modern",
    bundled: true,
  },
  {
    family: "Inter",
    label: "Inter",
    description: "Plain and very easy to read from a distance",
    bundled: true,
  },
  {
    family: "Lora",
    label: "Lora",
    description: "Warm serif — suits Bible readings",
    bundled: true,
  },
  {
    family: "Oswald",
    label: "Oswald",
    description: "Tall and condensed — fits long song lines",
    bundled: true,
  },
  {
    family: "JetBrains Mono",
    label: "JetBrains Mono",
    description: "Evenly spaced — notices and references",
    bundled: true,
  },
];

/** The family Selah uses when a setting names one that is not bundled. */
export const DEFAULT_FONT_FAMILY = "Creato Display";

/** Resolves a stored font family to one that is actually available. */
export function resolveFontFamily(family: string | undefined): string {
  if (!family) {
    return DEFAULT_FONT_FAMILY;
  }
  return FONT_OPTIONS.some((option) => option.family === family)
    ? family
    : DEFAULT_FONT_FAMILY;
}

/**
 * Reads a `#RRGGBB` colour and reports whether it is light enough that dark
 * text will be more readable than white text. The projector picks its text
 * colour from the background this way, so a pale background setting is still
 * usable.
 */
export function isLightColor(color: string): boolean {
  const hex = color.trim().replace("#", "");
  if (hex.length !== 6 || !/^[0-9a-f]{6}$/i.test(hex)) {
    return false;
  }
  const [r, g, b] = [0, 2, 4].map((offset) =>
    parseInt(hex.slice(offset, offset + 2), 16) / 255,
  );
  // Relative luminance (WCAG), which matches how bright a colour looks.
  const linear = [r, g, b].map((channel) =>
    channel <= 0.03928 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4,
  );
  const luminance = 0.2126 * linear[0] + 0.7152 * linear[1] + 0.0722 * linear[2];
  return luminance > 0.5;
}
