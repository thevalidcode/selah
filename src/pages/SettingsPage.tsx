import { useCallback, useEffect, useState } from "react";
import { HardDrive, Mic, Monitor, Palette, Sparkles, Upload } from "lucide-react";

import PageHeader, { KeyValueList, Panel } from "@/components/PageHeader";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@/components/ui/tabs";
import { useSettings } from "@/hooks/useSettings";
import { audioApi, bibleApi, presentationApi, speechApi } from "@/lib/api";
import type {
  AudioDeviceInfo,
  DisplayInfo,
  SpeechManagerState,
  TranslationStatus,
} from "@/types";

/**
 * Settings.
 *
 * Every control writes through to SQLite immediately (one JSON settings
 * document), so changing a value can never be lost by forgetting to save.
 */
export default function SettingsPage() {
  const { settings, save, error } = useSettings();

  return (
    <>
      <PageHeader
        title="Settings"
        subtitle="Stored locally in SQLite — no cloud, no accounts"
        actions={<Badge variant="muted">offline</Badge>}
      />

      {error ? (
        <p className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
          {error}
        </p>
      ) : null}

      <Tabs defaultValue="general">
        <TabsList>
          <TabsTrigger value="general">
            <Palette className="size-3.5" /> General
          </TabsTrigger>
          <TabsTrigger value="audio">
            <Mic className="size-3.5" /> Audio
          </TabsTrigger>
          <TabsTrigger value="speech">
            <Sparkles className="size-3.5" /> Speech
          </TabsTrigger>
          <TabsTrigger value="presentation">
            <Monitor className="size-3.5" /> Presentation
          </TabsTrigger>
          <TabsTrigger value="database">
            <HardDrive className="size-3.5" /> Database
          </TabsTrigger>
        </TabsList>

        <TabsContent value="general">
          <GeneralSection settings={settings} save={save} />
        </TabsContent>
        <TabsContent value="audio">
          <AudioSection settings={settings} save={save} />
        </TabsContent>
        <TabsContent value="speech">
          <SpeechSection settings={settings} save={save} />
        </TabsContent>
        <TabsContent value="presentation">
          <PresentationSection settings={settings} save={save} />
        </TabsContent>
        <TabsContent value="database">
          <DatabaseSection />
        </TabsContent>
      </Tabs>
    </>
  );
}

type Settings = NonNullable<ReturnType<typeof useSettings>["settings"]>;
type Save = ReturnType<typeof useSettings>["save"];

// ---------------------------------------------------------------- general

function GeneralSection({ settings, save }: { settings: Settings | null; save: Save }) {
  const [translations, setTranslations] = useState<TranslationStatus[]>([]);

  useEffect(() => {
    bibleApi
      .listBibleTranslations()
      .then(setTranslations)
      .catch(() => setTranslations([]));
  }, []);

  if (!settings) {
    return <Panel title="General">Loading…</Panel>;
  }

  return (
    <Panel title="General">
      <div className="grid gap-4 sm:grid-cols-2">
        <div className="space-y-1.5">
          <Label htmlFor="app-name">Application name</Label>
          <Input
            id="app-name"
            value={settings.general.appName}
            onChange={(e) =>
              void save({ general: { appName: e.target.value } })
            }
          />
        </div>

        <div className="space-y-1.5">
          <Label htmlFor="theme">Theme</Label>
          <Select
            value={settings.general.theme}
            onValueChange={(theme) => {
              void save({ general: { theme } });
              document.documentElement.classList.toggle(
                "light",
                theme === "light",
              );
              document.documentElement.classList.toggle("dark", theme !== "light");
            }}
          >
            <SelectTrigger id="theme">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="dark">Dark (recommended)</SelectItem>
              <SelectItem value="light">Light</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div className="space-y-1.5 sm:col-span-2">
          <Label htmlFor="default-translation">Default Bible translation</Label>
          <Select
            value={settings.general.defaultTranslationId}
            onValueChange={async (value) => {
              try {
                await bibleApi.setDefaultTranslation(value);
              } catch {
                // The Rust side reports a missing translation; ignore here so
                // the settings write still lands.
              }
              void save({ general: { defaultTranslationId: value } });
            }}
          >
            <SelectTrigger id="default-translation">
              <SelectValue placeholder="No translation installed" />
            </SelectTrigger>
            <SelectContent>
              {translations.map(({ translation }) => (
                <SelectItem key={translation.id} value={translation.id}>
                  {translation.name}
                  {translation.abbreviation
                    ? ` (${translation.abbreviation})`
                    : ""}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>
    </Panel>
  );
}

// ------------------------------------------------------------------- audio

function AudioSection({ settings, save }: { settings: Settings | null; save: Save }) {
  const [devices, setDevices] = useState<AudioDeviceInfo[]>([]);
  const [capture, setCapture] = useState<{
    capturing: boolean;
    deviceId?: string;
    sampleRate?: number;
    channels?: number;
  } | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    audioApi
      .listAudioDevices()
      .then(setDevices)
      .catch(() => setDevices([]));
    audioApi
      .getAudioCaptureState()
      .then(setCapture)
      .catch(() => setCapture(null));
  }, []);

  async function toggleCapture() {
    setBusy(true);
    try {
      if (capture?.capturing) {
        await audioApi.stopAudioCapture();
      } else {
        await audioApi.startAudioCapture();
      }
      setCapture(await audioApi.getAudioCaptureState());
    } finally {
      setBusy(false);
    }
  }

  return (
    <Panel title="Audio input">
      <div className="grid gap-4 sm:grid-cols-2">
        <div className="space-y-1.5 sm:col-span-2">
          <Label htmlFor="input-device">Input device</Label>
          <Select
            value={settings?.audio.inputDeviceId}
            onValueChange={(value) =>
              void save({ audio: { inputDeviceId: value } })
            }
          >
            <SelectTrigger id="input-device">
              <SelectValue placeholder="System default microphone" />
            </SelectTrigger>
            <SelectContent>
              {devices.map((device) => (
                <SelectItem key={device.id} value={device.id}>
                  {device.name}
                  {device.isDefault ? " (system default)" : ""} ·{" "}
                  {device.defaultSampleRate} Hz · {device.channels} ch
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="space-y-1.5">
          <Label htmlFor="sample-rate">Sample rate</Label>
          <Input
            id="sample-rate"
            type="number"
            min={0}
            value={settings?.audio.sampleRate ?? 0}
            onChange={(e) =>
              void save({ audio: { sampleRate: Number(e.target.value) || 0 } })
            }
          />
          <p className="text-xs text-muted-foreground">
            0 = use the device default. Audio is resampled to{" "}
            {settings?.speech.speechSampleRate ?? 16000} Hz for recognition.
          </p>
        </div>

        <div className="space-y-1.5">
          <Label>Capture status</Label>
          <div className="flex items-center gap-2">
            <Badge variant={capture?.capturing ? "success" : "muted"}>
              {capture?.capturing ? "capturing" : "stopped"}
            </Badge>
            <Button
              variant="outline"
              size="sm"
              disabled={busy}
              onClick={() => void toggleCapture()}
            >
              {capture?.capturing ? "Stop" : "Test capture"}
            </Button>
          </div>
        </div>
      </div>

      {capture?.capturing ? (
        <>
          <Separator className="my-4" />
          <KeyValueList
            items={[
              { label: "Device", value: capture.deviceId ?? "—" },
              { label: "Sample rate", value: capture.sampleRate ?? "—" },
              { label: "Channels", value: capture.channels ?? "—" },
            ]}
          />
        </>
      ) : null}
    </Panel>
  );
}

// ------------------------------------------------------------------ speech

function SpeechSection({ settings, save }: { settings: Settings | null; save: Save }) {
  const [state, setState] = useState<SpeechManagerState | null>(null);

  useEffect(() => {
    speechApi
      .getSpeechState()
      .then(setState)
      .catch(() => setState(null));
  }, []);

  if (!settings) {
    return <Panel title="Speech">Loading…</Panel>;
  }

  return (
    <Panel title="Speech recognition">
      <div className="grid gap-4 sm:grid-cols-2">
        <div className="space-y-1.5">
          <Label htmlFor="recognizer">Recognizer</Label>
          <Select
            value={settings.speech.recognizer}
            onValueChange={(recognizer) =>
              void save({
                speech: { recognizer: recognizer as "mock" | "whisper" },
              })
            }
          >
            <SelectTrigger id="recognizer">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="mock">
                Development recognizer (no transcription)
              </SelectItem>
              <SelectItem value="whisper">whisper.cpp (local, CPU)</SelectItem>
            </SelectContent>
          </Select>
          <p className="text-xs text-muted-foreground">
            Active: {state?.recognizerId ?? "—"}
            {state?.recognizerId === "mock"
              ? " — the development build never fabricates text."
              : ""}
          </p>
        </div>

        <div className="space-y-1.5">
          <Label htmlFor="model-path">Whisper model path</Label>
          <Input
            id="model-path"
            value={settings.speech.modelPath ?? ""}
            onChange={(e) =>
              void save({ speech: { modelPath: e.target.value || undefined } })
            }
            placeholder="…/models/whisper/ggml-base.en.bin"
          />
          <p className="text-xs text-muted-foreground">
            Models are never downloaded automatically.
          </p>
        </div>

        <div className="space-y-1.5">
          <Label htmlFor="language">Language</Label>
          <Input
            id="language"
            value={settings.speech.language ?? ""}
            onChange={(e) =>
              void save({ speech: { language: e.target.value || undefined } })
            }
            placeholder="en (blank = auto)"
          />
        </div>

        <div className="space-y-1.5">
          <Label htmlFor="speech-rate">Recognition sample rate</Label>
          <Input
            id="speech-rate"
            type="number"
            value={settings.speech.speechSampleRate}
            onChange={(e) =>
              void save({
                speech: { speechSampleRate: Number(e.target.value) || 16000 },
              })
            }
          />
          <p className="text-xs text-muted-foreground">
            whisper.cpp expects 16000 Hz mono.
          </p>
        </div>

        <div className="space-y-1.5">
          <Label htmlFor="threads">Threads</Label>
          <Input
            id="threads"
            type="number"
            min={1}
            value={settings.speech.threads}
            onChange={(e) =>
              void save({ speech: { threads: Number(e.target.value) || 1 } })
            }
          />
        </div>

        <div className="flex items-center justify-between rounded-lg border border-border/60 px-3 py-2">
          <div>
            <p className="text-sm font-medium">Voice activity detection</p>
            <p className="text-xs text-muted-foreground">
              Only detected speech reaches the recognizer.
            </p>
          </div>
          <Switch
            checked={settings.speech.vadEnabled}
            onCheckedChange={(vadEnabled) =>
              void save({ speech: { vadEnabled } })
            }
          />
        </div>
      </div>
    </Panel>
  );
}

// ------------------------------------------------------------ presentation

function PresentationSection({
  settings,
  save,
}: {
  settings: Settings | null;
  save: Save;
}) {
  const [displays, setDisplays] = useState<DisplayInfo[]>([]);
  const [open, setOpen] = useState(false);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    presentationApi
      .listDisplays()
      .then(setDisplays)
      .catch(() => setDisplays([]));
  }, []);

  async function toggleWindow() {
    if (!settings) {
      return;
    }
    setBusy(true);
    try {
      if (open) {
        await presentationApi.closePresentationWindow();
        setOpen(false);
      } else {
        await presentationApi.openPresentationWindow(
          settings.presentation.displayIndex,
          settings.presentation.fullscreen,
        );
        setOpen(true);
      }
    } finally {
      setBusy(false);
    }
  }

  if (!settings) {
    return <Panel title="Presentation">Loading…</Panel>;
  }

  return (
    <Panel title="Presentation output">
      <div className="grid gap-4 sm:grid-cols-2">
        <div className="space-y-1.5 sm:col-span-2">
          <Label htmlFor="presentation-display">Presentation display</Label>
          <Select
            value={
              settings.presentation.displayIndex === undefined
                ? undefined
                : String(settings.presentation.displayIndex)
            }
            onValueChange={(value) => {
              const index = Number(value);
              void save({ presentation: { displayIndex: index } });
              // The Rust side validates the index; a stale display list simply
              // fails silently here.
              void presentationApi
                .setPresentationDisplay(index)
                .catch(() => undefined);
            }}
          >
            <SelectTrigger id="presentation-display">
              <SelectValue placeholder="Primary display" />
            </SelectTrigger>
            <SelectContent>
              {displays.map((display) => (
                <SelectItem key={display.index} value={String(display.index)}>
                  {display.name ?? `Display ${display.index + 1}`} ·{" "}
                  {display.size[0]}×{display.size[1]}
                  {display.isPrimary ? " (primary)" : ""}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="space-y-1.5">
          <Label htmlFor="background">Background</Label>
          <Input
            id="background"
            value={settings.presentation.background}
            onChange={(e) =>
              void save({ presentation: { background: e.target.value } })
            }
            placeholder="#000000"
          />
        </div>

        <div className="space-y-1.5">
          <Label htmlFor="font-size">Font size</Label>
          <Input
            id="font-size"
            type="number"
            min={12}
            value={settings.presentation.fontSize}
            onChange={(e) =>
              void save({
                presentation: { fontSize: Number(e.target.value) || 12 },
              })
            }
          />
        </div>

        <div className="flex items-center justify-between rounded-lg border border-border/60 px-3 py-2">
          <div>
            <p className="text-sm font-medium">Fullscreen</p>
            <p className="text-xs text-muted-foreground">
              Project without window chrome.
            </p>
          </div>
          <Switch
            checked={settings.presentation.fullscreen}
            onCheckedChange={(fullscreen) =>
              void save({ presentation: { fullscreen } })
            }
          />
        </div>

        <div className="flex items-center justify-between rounded-lg border border-border/60 px-3 py-2">
          <div>
            <p className="text-sm font-medium">Follow live</p>
            <p className="text-xs text-muted-foreground">
              Keep the projector in sync with the Live screen.
            </p>
          </div>
          <Switch
            checked={settings.presentation.followLive}
            onCheckedChange={(followLive) =>
              void save({ presentation: { followLive } })
            }
          />
        </div>

        <div className="flex items-center gap-2 sm:col-span-2">
          <Badge variant={open ? "success" : "muted"}>
            {open ? "display open" : "display closed"}
          </Badge>
          <Button
            variant="outline"
            size="sm"
            disabled={busy}
            onClick={() => void toggleWindow()}
          >
            {open ? "Close presentation window" : "Open presentation window"}
          </Button>
        </div>
      </div>
    </Panel>
  );
}

// ---------------------------------------------------------------- database

function DatabaseSection() {
  const [translations, setTranslations] = useState<TranslationStatus[]>([]);
  const [importPath, setImportPath] = useState("");
  const [importing, setImporting] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [importError, setImportError] = useState<string | null>(null);

  const reload = useCallback(() => {
    bibleApi
      .listBibleTranslations()
      .then(setTranslations)
      .catch(() => setTranslations([]));
  }, []);

  useEffect(reload, [reload]);

  async function runImport() {
    if (importPath.trim().length === 0) {
      return;
    }
    setImporting(true);
    try {
      const result = await bibleApi.importBibleTranslation(importPath.trim());
      setNotice(
        `Imported ${result.versesImported} verses for “${result.translationId}”.`,
      );
      setImportError(null);
      setImportPath("");
      reload();
    } catch (e: unknown) {
      setNotice(null);
      setImportError(e instanceof Error ? e.message : String(e));
    } finally {
      setImporting(false);
    }
  }

  return (
    <Panel title="Database">
      <p className="text-sm text-muted-foreground">
        Selah stores everything in a single SQLite file inside the application
        data directory. It is never written into the source repository, and no
        data leaves this machine.
      </p>

      <Separator className="my-4" />

      <h3 className="mb-2 text-xs font-semibold tracking-[0.14em] text-muted-foreground uppercase">
        Import a translation
      </h3>
      <p className="mb-3 text-sm text-muted-foreground">
        Selah ships no Bible text. Point at a JSON document you are licensed to
        use; the documented format lives in{" "}
        <code className="font-mono">data/bible/README.md</code>.
      </p>
      <div className="flex flex-col gap-3 sm:flex-row sm:items-end">
        <div className="flex-1 space-y-1.5">
          <Label htmlFor="import-path">Absolute file path</Label>
          <Input
            id="import-path"
            value={importPath}
            onChange={(e) => setImportPath(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                void runImport();
              }
            }}
            placeholder="…/Downloads/web-bible.json"
          />
        </div>
        <Button
          variant="success"
          disabled={importing || importPath.trim().length === 0}
          onClick={() => void runImport()}
        >
          <Upload className="size-4" />
          Import
        </Button>
      </div>
      {notice ? (
        <p className="mt-3 rounded-md border border-success/30 bg-success/10 px-3 py-2 text-sm text-success">
          {notice}
        </p>
      ) : null}
      {importError ? (
        <p className="mt-3 rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
          {importError}
        </p>
      ) : null}

      <Separator className="my-4" />

      <h3 className="mb-2 text-xs font-semibold tracking-[0.14em] text-muted-foreground uppercase">
        Installed translations
      </h3>
      {translations.length === 0 ? (
        <p className="text-sm text-muted-foreground">
          None installed. Selah ships no Bible text — import a translation you
          are licensed to redistribute.
        </p>
      ) : (
        <ul className="space-y-1.5">
          {translations.map(({ translation, verseCount }) => (
            <li
              key={translation.id}
              className="flex items-center justify-between gap-3 rounded-lg border border-border/60 px-3 py-2"
            >
              <div className="min-w-0">
                <p className="truncate text-sm font-medium">
                  {translation.name}
                </p>
                <p className="text-xs text-muted-foreground">
                  {translation.language} · {translation.id}
                  {translation.isDefault ? " · default" : ""}
                </p>
              </div>
              <Badge variant={verseCount > 0 ? "success" : "warning"}>
                {verseCount} verses
              </Badge>
            </li>
          ))}
        </ul>
      )}

      <Separator className="my-4" />

      <KeyValueList
        items={[
          { label: "Engine", value: "SQLite (bundled, WAL)" },
          { label: "Schema", value: "versioned migrations" },
          {
            label: "Tables",
            value:
              "translations, books, verses, verses_fts, settings, media, presentations, presentation_items",
          },
        ]}
      />
    </Panel>
  );
}

