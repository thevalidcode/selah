import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import {
  ArrowUpRight,
  BookOpen,
  CheckCircle2,
  Mic,
  Monitor,
  Radio,
} from "lucide-react";

import PageHeader, {
  EmptyHint,
  KeyValueList,
  Panel,
  StatusPill,
} from "@/components/PageHeader";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { useSettings } from "@/hooks/useSettings";
import { audioApi, bibleApi, presentationApi, speechApi } from "@/lib/api";
import { friendlyContentType } from "@/lib/content";
import type {
  AudioCaptureState,
  DisplayInfo,
  PresentationItem,
  SpeechManagerState,
  TranslationStatus,
} from "@/types";

/** Home: a one-glance readiness check before a service starts. */
export default function HomePage() {
  const { settings } = useSettings();
  const [speech, setSpeech] = useState<SpeechManagerState | null>(null);
  const [audio, setAudio] = useState<AudioCaptureState | null>(null);
  const [displays, setDisplays] = useState<DisplayInfo[]>([]);
  const [translations, setTranslations] = useState<TranslationStatus[]>([]);
  const [current, setCurrent] = useState<PresentationItem | null>(null);

  useEffect(() => {
    speechApi.getSpeechState().then(setSpeech).catch(() => setSpeech(null));
    audioApi
      .getAudioCaptureState()
      .then(setAudio)
      .catch(() => setAudio(null));
    presentationApi
      .listDisplays()
      .then(setDisplays)
      .catch(() => setDisplays([]));
    bibleApi
      .listBibleTranslations()
      .then(setTranslations)
      .catch(() => setTranslations([]));
    presentationApi
      .getPresentationState()
      .then((state) => setCurrent(state.current ?? null))
      .catch(() => setCurrent(null));
  }, []);

  const hasTranslation = translations.some((t) => t.verseCount > 0);
  const ready = hasTranslation && displays.length > 0;

  return (
    <>
      <PageHeader
        title="Home"
        subtitle="Check everything works before the service starts"
        actions={
          <>
            <StatusPill
              active={speech?.listening ?? false}
              label={speech?.listening ? "Listening" : "Idle"}
            />
            <Button asChild size="sm" variant="success">
              <Link to="/live">
                <Radio className="size-3.5" />
                Go live
              </Link>
            </Button>
          </>
        }
      />

      <div className="grid gap-4 lg:grid-cols-2">
        <Panel
          title="Are we ready?"
          actions={
            <Badge variant={ready ? "success" : "warning"}>
              {ready ? "ready" : "finish setup"}
            </Badge>
          }
        >
          <ul className="space-y-2">
            <ChecklistRow
              ok={hasTranslation}
              label="Bible text loaded"
              detail={
                hasTranslation
                  ? `${translations.filter((t) => t.verseCount > 0).length} ready to use`
                  : "add one you are allowed to use"
              }
            />
            <ChecklistRow
              ok={displays.length > 0}
              label="Screen for the congregation"
              detail={`${displays.length} screen${displays.length === 1 ? "" : "s"} found`}
            />
            <ChecklistRow
              ok={Boolean(settings?.audio.inputDeviceId)}
              label="Microphone chosen"
              detail={
                settings?.audio.inputDeviceId
                  ? "chosen"
                  : "using the computer default"
              }
            />
            <ChecklistRow
              ok={settings?.speech.recognizer === "moonshine"}
              label="Listening for words"
              detail={
                settings?.speech.recognizer === "moonshine"
                  ? "turned on"
                  : "off — Selah cannot understand speech yet"
              }
            />
          </ul>
        </Panel>

        <Panel title="Listening right now">
          {!speech ? (
            <EmptyHint>Starting up…</EmptyHint>
          ) : (
            <KeyValueList
              items={[
                {
                  label: "Listening",
                  value: speech.listening ? "yes" : "no",
                },
                {
                  label: "Times it heard speech",
                  value: speech.segmentsSeen,
                },
                {
                  label: "Times it wrote words",
                  value: speech.transcriptsGenerated,
                },
              ]}
            />
          )}
        </Panel>

        <Panel title="Microphone">
          {!audio ? (
            <EmptyHint>No microphone information yet.</EmptyHint>
          ) : (
            <KeyValueList
              items={[
                {
                  label: "Recording",
                  value: audio.capturing ? "yes" : "no",
                },
                { label: "Using", value: audio.deviceId ?? "—" },
                {
                  label: "Sound kept in memory",
                  value: `${Math.round(audio.bufferedSamples / 1000)}k samples`,
                },
              ]}
            />
          )}
        </Panel>

        <Panel
          title="On screen now"
          actions={
            current ? (
              <Badge variant="success">{friendlyContentType(current.contentType)}</Badge>
            ) : (
              <Badge variant="muted">blank</Badge>
            )
          }
        >
          {!current ? (
            <EmptyHint>The screen is blank right now.</EmptyHint>
          ) : (
            <div className="space-y-1.5">
              <p className="text-sm font-medium">{current.title}</p>
              <p className="selectable line-clamp-3 text-sm text-muted-foreground">
                {current.payload.kind === "media"
                  ? current.payload.path
                  : current.payload.text}
              </p>
            </div>
          )}
        </Panel>
      </div>

      <Panel title="Start here">
        <ol className="space-y-2 text-sm text-muted-foreground">
          <Step
            icon={Mic}
            index={1}
            text="Choose the microphone that hears the speaker."
          />
          <Step
            icon={BookOpen}
            index={2}
            text="Add the Bible text you are allowed to use."
          />
          <Step
            icon={Monitor}
            index={3}
            text="Pick the screen the congregation sees, then press Listen on Live."
          />
        </ol>
        <div className="mt-4 flex flex-wrap gap-2">
          <Button asChild variant="outline" size="sm">
            <Link to="/live">
              Live <ArrowUpRight className="size-3.5" />
            </Link>
          </Button>
          <Button asChild variant="outline" size="sm">
            <Link to="/bible">
              Bible <ArrowUpRight className="size-3.5" />
            </Link>
          </Button>
          <Button asChild variant="outline" size="sm">
            <Link to="/settings">
              Settings <ArrowUpRight className="size-3.5" />
            </Link>
          </Button>
        </div>
      </Panel>
    </>
  );
}

function ChecklistRow({
  ok,
  label,
  detail,
}: {
  ok: boolean;
  label: string;
  detail: string;
}) {
  return (
    <li className="flex items-start gap-2.5">
      <CheckCircle2
        className={
          ok
            ? "text-success mt-0.5 size-4 shrink-0"
            : "text-muted-foreground/40 mt-0.5 size-4 shrink-0"
        }
      />
      <div className="min-w-0">
        <p className="text-sm">{label}</p>
        <p className="text-xs text-muted-foreground">{detail}</p>
      </div>
    </li>
  );
}

function Step({
  icon: Icon,
  index,
  text,
}: {
  icon: typeof Mic;
  index: number;
  text: string;
}) {
  return (
    <li className="flex items-start gap-3">
      <span className="grid size-6 shrink-0 place-items-center rounded-md bg-muted text-xs font-semibold text-muted-foreground">
        {index}
      </span>
      <span className="flex items-center gap-1.5">
        <Icon className="size-3.5 shrink-0" />
        {text}
      </span>
    </li>
  );
}
