import { useCallback, useEffect, useRef, useState } from "react";

import { speechApi } from "@/lib/api";
import { EVENTS, useTauriEvent } from "@/lib/events";
import type {
  DetectionResult,
  SpeechManagerState,
  Transcript,
  VadSegmentEvent,
} from "@/types";

/** One entry in the live transcript feed. */
export interface LiveTranscriptEntry {
  id: string;
  text: string;
  source: Transcript["source"];
  receivedAt: number;
  /** True when the recognizer produced no text (mock build / silence). */
  empty: boolean;
}

/** A pending Scripture suggestion awaiting operator confirmation. */
export interface PendingDetection {
  id: string;
  results: DetectionResult[];
  receivedAt: number;
}

/**
 * Subscribes to the Rust speech pipeline and keeps the operator-visible live
 * state: transcript feed, VAD activity, and detected content awaiting review.
 *
 * Nothing is auto-projected — detected Scripture only becomes a suggestion
 * until the operator presses Display.
 */
export function useLiveSession(): {
  speechState: SpeechManagerState | null;
  feed: LiveTranscriptEntry[];
  pending: PendingDetection | null;
  segments: VadSegmentEvent[];
  listening: boolean;
  busy: boolean;
  error: string | null;
  start: () => Promise<void>;
  stop: () => Promise<void>;
  dismissPending: () => void;
  clearFeed: () => void;
} {
  const [speechState, setSpeechState] = useState<SpeechManagerState | null>(
    null,
  );
  const [feed, setFeed] = useState<LiveTranscriptEntry[]>([]);
  const [pending, setPending] = useState<PendingDetection | null>(null);
  const [segments, setSegments] = useState<VadSegmentEvent[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const counter = useRef(0);

  const nextId = () => {
    counter.current += 1;
    return `${Date.now()}-${counter.current}`;
  };

  // Initial snapshot (the pipeline may already be running).
  useEffect(() => {
    speechApi
      .getSpeechState()
      .then(setSpeechState)
      .catch(() => setSpeechState(null));
  }, []);

  useTauriEvent(EVENTS.speechStarted, () => {
    setSpeechState((prev) => (prev ? { ...prev, listening: true } : prev));
  });

  useTauriEvent(EVENTS.speechStopped, () => {
    setSpeechState((prev) => (prev ? { ...prev, listening: false } : prev));
  });

  useTauriEvent(EVENTS.speechVadSegment, (payload) => {
    setSegments((prev) => [...prev.slice(-19), payload]);
  });

  useTauriEvent(EVENTS.speechTranscript, (transcript) => {
    setSpeechState((prev) =>
      prev
        ? {
            ...prev,
            transcriptsGenerated: prev.transcriptsGenerated + 1,
            lastError: undefined,
          }
        : prev,
    );

    const empty = transcript.text.trim().length === 0;
    setFeed((prev) =>
      [
        ...prev.slice(-49),
        {
          id: nextId(),
          text: transcript.text.trim(),
          source: transcript.source,
          receivedAt: Date.now(),
          empty,
        },
      ].filter((entry) => entry.text.length > 0 || entry.empty),
    );
  });

  useTauriEvent(EVENTS.contentDetected, (event) => {
    const meaningful = event.results.filter(
      (r) => r.content.type === "scripture",
    );
    if (meaningful.length === 0) {
      return;
    }
    setPending({ id: nextId(), results: meaningful, receivedAt: Date.now() });
  });

  const start = useCallback(async () => {
    setBusy(true);
    try {
      const state = await speechApi.startListening();
      setSpeechState(state);
      setError(null);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }, []);

  const stop = useCallback(async () => {
    setBusy(true);
    try {
      const state = await speechApi.stopListening();
      setSpeechState(state);
      setError(null);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }, []);

  const dismissPending = useCallback(() => setPending(null), []);
  const clearFeed = useCallback(() => setFeed([]), []);

  return {
    speechState,
    feed,
    pending,
    segments,
    listening: speechState?.listening ?? false,
    busy,
    error,
    start,
    stop,
    dismissPending,
    clearFeed,
  };
}
