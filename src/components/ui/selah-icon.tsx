import * as React from "react";

import { cn } from "@/lib/utils";

/**
 * Selah's logo, as a component.
 *
 * The mark is a gold cross inside a square frame — an open window with the
 * cross dividing it into four panes. It is drawn on the same 48-unit grid as
 * the application icon in `src-tauri/icons/selah-icon.svg`, so the window, the
 * projector screen and the operating-system icon can never drift apart. Keep
 * the two files in sync when the artwork changes.
 *
 * Colours come from design tokens rather than being hard-coded, so the logo
 * keeps its true identity colours while everything around it follows the
 * active theme:
 *
 *   `--brand-mark`  logo gold (#C9A24B)
 *   `--brand-tile`  logo navy (#141B2E)
 *
 * Both are defined once per theme in `src/index.css`.
 */

/** The bare cross-and-frame glyph. Internal: always render via `SelahIcon`. */
function Mark() {
  return (
    <g
      // The vertical beam overshoots the frame top and bottom while the
      // horizontal beam meets the frame exactly. That asymmetry is the
      // logo's signature — do not "tidy" it into a symmetric cross.
      fill="none"
      stroke="currentColor"
      strokeWidth={3.1}
      strokeLinecap="butt"
    >
      <rect x="12.55" y="12.55" width="22.9" height="22.9" />
      <path d="M24 7.5V40.5" />
      <path d="M12.55 24H35.45" />
    </g>
  );
}

export interface SelahIconProps extends React.ComponentProps<"svg"> {
  /** Rendered size in pixels. The logo is always square. */
  size?: number;
  /** Draw the navy rounded tile behind the mark, as on the application icon. */
  tile?: boolean;
  /** Accessible name. Omit when the logo is decorative (next to the wordmark). */
  title?: string;
}

/**
 * The Selah logo.
 *
 * Use `<SelahIcon tile />` where the logo stands alone as the application
 * mark (sidebar header, splash screens) and `<SelahIcon />` where it sits on a
 * surface that already reads as brand — notably the projector, where the navy
 * tile would disappear against the black background.
 */
export function SelahIcon({
  className,
  size = 24,
  tile = false,
  title,
  ...props
}: SelahIconProps) {
  const decorative = !title;

  return (
    <svg
      viewBox="0 0 48 48"
      width={size}
      height={size}
      role={decorative ? "presentation" : "img"}
      aria-hidden={decorative || undefined}
      aria-label={title}
      className={cn("shrink-0", className)}
      {...props}
    >
      {title ? <title>{title}</title> : null}

      {tile ? (
        // Squircle tile. The corner radius is ~23% of the side, matching how
        // macOS and Windows round their own application tiles.
        <rect x="0" y="0" width="48" height="48" rx="11" fill="var(--brand-tile)" />
      ) : null}

      <g
        // Scale the mark down inside the tile so it keeps the padding of the
        // real icon instead of touching the edges. The stroke scales with it,
        // which is what preserves the logo's proportions.
        transform={
          tile ? "translate(24 24) scale(0.62) translate(-24 -24)" : undefined
        }
        className="text-brand-mark"
      >
        <Mark />
      </g>
    </svg>
  );
}
