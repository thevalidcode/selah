/**
 * Class-name helpers used by every UI primitive.
 *
 * `cn` merges conditional class names (clsx) and resolves Tailwind conflicts
 * (tailwind-merge) so composed components can be overridden by callers.
 */

import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}
