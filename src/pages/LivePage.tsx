import { useCallback, useEffect, useMemo, useState } from "react";
import { Circle, Square } from "lucide-react";

import PageHeader, { StatusPill } from "@/components/PageHeader";
import DetectedPanel from "@/components/live/DetectedPanel";
import LiveTranscriptPanel from "@/components/live/LiveTranscriptPanel";
import NowProjectingPanel from "@/components/live/NowProjectingPanel";
import PipelineDiagnosticsPanel from "@/components/live/PipelineDiagnosticsPanel";
import { Button } from "@/components/ui/button";
import { useBooks } from "@/hooks/useBooks";
import { useLiveSession } from "@/hooks/useLiveSession";
import { useSettings } from "@/hooks/useSettings";
import { bibleApi, presentationApi } from "@/lib/api";
import { EVENTS, useTauriEvent } from "@/lib/events";
import { formatReference } from "@/lib/reference";
import type {
  DetectedContent,
  Passage,
  PresentationItem,
  TranslationStatus,
} from "@/types";

/**
 * Live service screen.
 *
 * Pipeline: microphone → VAD → recognizer → deterministic detection. Detected
 * Scripture is only ever a *suggestion* here — the operator must press Display
 * before anything reaches the projector.
 */
export default function LivePage() {
  const { settings } = useSettings();
  const { bookNames } = useBooks();
  const {
    speechState,
    feed,
    pending,
    listening,
    busy,
    error,
    start,
    stop,
    dismissPending,
    clearFeed,
  } = useLiveSession();

  const [translations, setTranslations] = useState<TranslationStatus[]>([]);
  const [current, setCurrent] = useState<PresentationItem | null>(null);
  const [projected, setProjected] = useState(false);
  const [working, setWorking] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);
  const [editing, setEditing] = useState(false);
  const [editTitle, setEditTitle] = useState("");
  const [editText, setEditText] = useState("");

  const translationId = useMemo(() => {
    const preferred = settings?.general.defaultTranslationId;
    if (preferred) {
      return preferred;
    }
    return translations[0]?.translation.id;
  }, [settings, translations]);

  const translationLabel = useMemo(() => {
    const found = translations.find((t) => t.translation.id === translationId);
    return found?.translation.abbreviation ?? found?.translation.name;
  }, [translations, translationId]);

  useEffect(() => {
    bibleApi
      .listBibleTranslations()
      .then(setTranslations)
      .catch(() => setTranslations([]));
    presentationApi
      .getPresentationState()
      .then((state) => setCurrent(state.current ?? null))
      .catch(() => setCurrent(null));
  }, []);

  useTauriEvent(EVENTS.presentationChanged, (payload) => {
    setCurrent(payload.item ?? null);
    setProjected(payload.projected);
  });
  useTauriEvent(EVENTS.presentationDisplayOpened, () => setProjected(true));
  useTauriEvent(EVENTS.presentationDisplayClosed, () => setProjected(false));

  const scripture = pending?.results.find(
    (result) => result.content.type === "scripture",
  );

  const startEdit = useCallback(() => {
    const reference = scripture?.content.reference;
    setEditTitle(
      reference ? formatReference(reference, bookNames) : "Message",
    );
    setEditText("");
    setEditing(true);
  }, [scripture, bookNames]);

  /** Resolves a detection against the installed translation, then projects. */
  const displayDetection = useCallback(
    async (content: DetectedContent, overrideText?: string) => {
      setWorking(true);
      setActionError(null);
      try {
        if (content.type === "scripture" && content.reference) {
          if (!translationId) {
            setActionError(
              "Install a Bible translation before displaying Scripture.",
            );
            return;
          }
          const passage: Passage = await bibleApi.getPassage({
            translationId,
            bookId: content.reference.bookId,
            chapter: content.reference.chapter,
            startVerse: content.reference.startVerse,
            endVerse: content.reference.endVerse,
          });
          const state = await presentationApi.projectPassage(
            overrideText === undefined
              ? passage
              : { ...passage, text: overrideText },
          );
          setCurrent(state.current ?? null);
        } else {
          const body = overrideText ?? content.text ?? "";
          if (body.trim().length === 0) {
            setActionError("Nothing to display.");
            return;
          }
          const state = await presentationApi.projectText(
            editTitle.trim() || "Message",
            body,
          );
          setCurrent(state.current ?? null);
        }
        dismissPending();
        setEditing(false);
      } catch (e: unknown) {
        setActionError(e instanceof Error ? e.message : String(e));
      } finally {
        setWorking(false);
      }
    },
    [translationId, dismissPending, editTitle],
  );

  const clearScreen = useCallback(async () => {
    setWorking(true);
    try {
      const state = await presentationApi.clearPresentation();
      setCurrent(state.current ?? null);
      setActionError(null);
    } catch (e: unknown) {
      setActionError(e instanceof Error ? e.message : String(e));
    } finally {
      setWorking(false);
    }
  }, []);

  return (
    <>
      <PageHeader
        title="Live"
        subtitle="Selah listens and suggests verses. You decide what goes on screen."
        actions={
          <>
            <StatusPill
              active={listening}
              label={listening ? "Listening" : "Idle"}
            />
            <Button
              variant={listening ? "outline" : "success"}
              size="sm"
              disabled={busy}
              onClick={() => void (listening ? stop() : start())}
            >
              {listening ? (
                <>
                  <Square className="size-3.5" /> Stop
                </>
              ) : (
                <>
                  <Circle className="size-3.5" /> Listen
                </>
              )}
            </Button>
          </>
        }
      />

      {error ?? actionError ? (
        <p className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
          {error ?? actionError}
        </p>
      ) : null}

      <div className="grid gap-4 xl:grid-cols-[1.15fr_1fr]">
        <LiveTranscriptPanel
          feed={feed}
          listening={listening}
          onClear={clearFeed}
        />
        <div className="flex flex-col gap-4">
          <DetectedPanel
            detection={scripture}
            bookNames={bookNames}
            translationLabel={translationLabel}
            working={working}
            editing={editing}
            editTitle={editTitle}
            editText={editText}
            onEditTitle={setEditTitle}
            onEditText={setEditText}
            onStartEdit={startEdit}
            onCancelEdit={() => setEditing(false)}
            onDisplay={(override) => {
              if (scripture) {
                void displayDetection(scripture.content, override);
              }
            }}
            onIgnore={() => {
              dismissPending();
              setEditing(false);
            }}
          />
          <NowProjectingPanel
            current={current}
            projected={projected}
            working={working}
            onClear={() => void clearScreen()}
          />
        </div>
      </div>

      <PipelineDiagnosticsPanel
        speechState={speechState}
        pendingCount={pending?.results.length ?? 0}
      />
    </>
  );
}
