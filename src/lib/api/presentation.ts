import { command } from "./client";
import type {
  DisplayInfo,
  Presentation,
  PresentationState,
} from "../../types";
import type { Passage } from "../../types";

// --------------------------------------------------------- live projection

export function getPresentationState(): Promise<PresentationState> {
  return command<PresentationState>("get_presentation_state");
}

/** Projects plain text (announcements, custom text, notes). */
export function projectText(
  title: string,
  text: string,
  display?: number,
  fullscreen?: boolean,
): Promise<PresentationState> {
  return command<PresentationState>("project_text", {
    request: {
      title,
      text,
      display: display ?? null,
      fullscreen: fullscreen ?? null,
    },
  });
}

/** Projects an already-resolved passage from the Bible repository. */
export function projectPassage(passage: Passage): Promise<PresentationState> {
  return command<PresentationState>("project_passage", { passage });
}

export function clearPresentation(): Promise<PresentationState> {
  return command<PresentationState>("clear_presentation");
}

// ------------------------------------------------------------- presentation window

export function openPresentationWindow(
  display?: number,
  fullscreen?: boolean,
): Promise<DisplayInfo> {
  return command<DisplayInfo>("open_presentation_window", {
    display: display ?? null,
    fullscreen: fullscreen ?? null,
  });
}

export function closePresentationWindow(): Promise<void> {
  return command<void>("close_presentation_window");
}

// ------------------------------------------------------------------ displays

export function listDisplays(): Promise<DisplayInfo[]> {
  return command<DisplayInfo[]>("list_displays");
}

export function setPresentationDisplay(display: number): Promise<DisplayInfo> {
  return command<DisplayInfo>("set_presentation_display", { display });
}

export function setFullscreen(fullscreen: boolean): Promise<void> {
  return command<void>("set_fullscreen", { fullscreen });
}

// -------------------------------------------------------- saved presentations

export function listPresentations(): Promise<Presentation[]> {
  return command<Presentation[]>("list_presentations");
}

export function getPresentation(id: string): Promise<Presentation | null> {
  return command<Presentation | null>("get_presentation", { id });
}

export function createPresentation(name: string): Promise<Presentation> {
  return command<Presentation>("create_presentation", { name });
}

export function deletePresentation(id: string): Promise<void> {
  return command<void>("delete_presentation", { id });
}

export function addPresentationItem(
  presentationId: string,
  type: string,
  payload: string,
): Promise<void> {
  return command<void>("add_presentation_item", {
    request: { presentationId, type, payload },
  });
}

export function removePresentationItem(
  presentationId: string,
  itemId: string,
): Promise<void> {
  return command<void>("remove_presentation_item", {
    presentationId,
    itemId,
  });
}
