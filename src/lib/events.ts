/**
 * Typed helpers around Tauri's event system.
 *
 * Event names mirror `src-tauri/src/events.rs`. The presentation window uses
 * the same helpers as the operator UI.
 */

import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";

import type {
  AudioCaptureState,
  DetectionEvent,
  MediaPlaybackCommand,
  MediaPlaybackState,
  PresentationChangedEvent,
  PresentationDisplayEvent,
  PresentationItem,
  PresentationSettingsEvent,
  Transcript,
  VadSegmentEvent,
} from "../types";

export const EVENTS = {
  audioStarted: "audio://started",
  audioStopped: "audio://stopped",
  speechStarted: "speech://started",
  speechStopped: "speech://stopped",
  speechVadSegment: "speech://segment",
  speechTranscript: "speech://transcript",
  contentDetected: "content://detected",
  presentationChanged: "presentation://changed",
  presentationSettings: "presentation://settings",
  presentationDisplayOpened: "presentation://display-opened",
  presentationDisplayClosed: "presentation://display-closed",
  mediaPlayback: "media://playback",
  mediaPlaybackState: "media://playback-state",
} as const;

export type EventMap = {
  [EVENTS.audioStarted]: AudioCaptureState;
  [EVENTS.audioStopped]: undefined;
  [EVENTS.speechStarted]: undefined;
  [EVENTS.speechStopped]: undefined;
  [EVENTS.speechVadSegment]: VadSegmentEvent;
  [EVENTS.speechTranscript]: Transcript;
  [EVENTS.contentDetected]: DetectionEvent;
  [EVENTS.presentationChanged]: PresentationChangedEvent;
  [EVENTS.presentationSettings]: PresentationSettingsEvent;
  [EVENTS.presentationDisplayOpened]: PresentationDisplayEvent;
  [EVENTS.presentationDisplayClosed]: PresentationDisplayEvent;
  [EVENTS.mediaPlayback]: MediaPlaybackCommand;
  [EVENTS.mediaPlaybackState]: MediaPlaybackState;
};

export type EventName = (typeof EVENTS)[keyof typeof EVENTS];

/**
 * React hook: subscribe to a Tauri event for the lifetime of the component.
 * The handler is kept in a ref so it always sees fresh state without
 * re-subscribing on every render.
 */
export function useTauriEvent<K extends EventName>(
  event: K,
  handler: (payload: EventMap[K]) => void,
): void {
  const handlerRef = useRef(handler);
  handlerRef.current = handler;

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    let disposed = false;
    listen<unknown>(event, (e) => {
      handlerRef.current(e.payload as EventMap[K]);
    }).then((fn) => {
      if (disposed) {
        fn();
        return;
      }
      unlisten = fn;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [event]);
}

/**
 * Presentation-window helper: watch the projection stream and report the
 * currently displayed item (or `null` when the screen was cleared).
 */
export async function subscribeToPresentation(
  onItem: (item: PresentationItem | null) => void,
): Promise<UnlistenFn> {
  return listen<PresentationChangedEvent>(
    EVENTS.presentationChanged,
    (e) => {
      onItem(e.payload.item ?? null);
    },
  );
}
