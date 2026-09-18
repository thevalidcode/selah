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
        subtitle="Saved on this computer. Nothing is sent anywhere."
        actions={<Badge variant="muted">no internet needed</Badge>}
      />

      {error ? (
        <p className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
          {error}
        </p>
      ) : null}

      <Tabs defaultValue="general">
        <TabsList>
          <TabsTrigger value="general">
            <Palette className="size-3.5" /> Basics
          </TabsTrigger>
          <TabsTrigger value="audio">
            <Mic className="size-3.5" /> Microphone
          </TabsTrigger>
          <TabsTrigger value="speech">
            <Sparkles className="size-3.5" /> Listening
          </TabsTrigger>
          <TabsTrigger value="presentation">
            <Monitor className="size-3.5" /> Screen
          </TabsTrigger>
          <TabsTrigger value="database">
            <HardDrive className="size-3.5" /> Bible text
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
    return <Panel title="Basics">Loading…</Panel>;
  }

  return (
    <Panel title="Basics">
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
          <p className="text-xs text-muted-foreground">
            A label for this installation. The window title stays “Selah”.
          </p>
        </div>

        <div className="space-y-1.5">
          <Label htmlFor="theme">Look</Label>
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
              <SelectItem value="dark">Dark (easier in a dim room)</SelectItem>
              <SelectItem value="light">Light</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div className="space-y-1.5 sm:col-span-2">
          <Label htmlFor="default-translation">Bible to use by default</Label>
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
              <SelectValue placeholder="No Bible added yet" />
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
    <Panel title="Microphone">
      <div className="grid gap-4 sm:grid-cols-2">
        <div className="space-y-1.5 sm:col-span-2">
          <Label htmlFor="input-device">Which microphone</Label>
          <Select
            value={settings?.audio.inputDeviceId}
            onValueChange={(value) =>
              void save({ audio: { inputDeviceId: value } })
            }
          >
            <SelectTrigger id="input-device">
              <SelectValue placeholder="Whatever the computer uses by default" />
            </SelectTrigger>
            <SelectContent>
              {devices.map((device) => (
                <SelectItem key={device.id} value={device.id}>
                  {device.name}
                  {device.isDefault ? " (system default)" : ""}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <p className="text-xs text-muted-foreground">
            Pick the one that hears the person speaking.
          </p>
        </div>

        <div className="space-y-1.5">
          <Label htmlFor="sample-rate">Sound quality</Label>
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
            Leave this at 0 unless you have been told otherwise — Selah adjusts
            the sound itself before listening.
          </p>
        </div>

        <div className="space-y-1.5">
          <Label>Is it working?</Label>
          <div className="flex items-center gap-2">
            <Badge variant={capture?.capturing ? "success" : "muted"}>
              {capture?.capturing ? "listening" : "off"}
            </Badge>
            <Button
              variant="outline"
              size="sm"
              disabled={busy}
              onClick={() => void toggleCapture()}
            >
              {capture?.capturing ? "Stop" : "Check microphone"}
            </Button>
          </div>
        </div>
      </div>

      {capture?.capturing ? (
        <>
          <Separator className="my-4" />
          <KeyValueList
            items={[
              { label: "Using", value: capture.deviceId ?? "—" },
              { label: "Quality", value: capture.sampleRate ?? "—" },
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
    return <Panel title="Listening">Loading…</Panel>;
  }

  return (
    <Panel title="Listening">
      <div className="grid gap-4 sm:grid-cols-2">
        <div className="space-y-1.5">
          <Label htmlFor="recognizer">Can Selah understand words?</Label>
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
                Off — hears the room but writes nothing
              </SelectItem>
              <SelectItem value="whisper">
                On — words are worked out on this computer
              </SelectItem>
            </SelectContent>
          </Select>
          <p className="text-xs text-muted-foreground">
            {state?.recognizerId === "whisper"
              ? "Words from the microphone are turned into text on this machine."
              : "Turned off. Selah will not invent words it did not hear."}
          </p>
        </div>

        <div className="space-y-1.5">
          <Label htmlFor="model-path">Voice model file</Label>
          <Input
            id="model-path"
            value={settings.speech.modelPath ?? ""}
            onChange={(e) =>
              void save({ speech: { modelPath: e.target.value || undefined } })
            }
            placeholder="…/models/whisper/ggml-base.en.bin"
          />
          <p className="text-xs text-muted-foreground">
            The file that teaches Selah English. Selah never downloads it for
            you — point at one you already have.
          </p>
        </div>

        <div className="space-y-1.5">
          <Label htmlFor="language">Language spoken</Label>
          <Input
            id="language"
            value={settings.speech.language ?? ""}
            onChange={(e) =>
              void save({ speech: { language: e.target.value || undefined } })
            }
            placeholder="en — leave blank to work it out"
          />
        </div>

        <div className="space-y-1.5">
          <Label htmlFor="speech-rate">Listening quality</Label>
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
            Leave this at 16000 unless you have been told otherwise.
          </p>
        </div>

        <div className="space-y-1.5">
          <Label htmlFor="threads">How hard it works</Label>
          <Input
            id="threads"
            type="number"
            min={1}
            value={settings.speech.threads}
            onChange={(e) =>
              void save({ speech: { threads: Number(e.target.value) || 1 } })
            }
          />
          <p className="text-xs text-muted-foreground">
            Higher is faster but uses more of the computer.
          </p>
        </div>

        <div className="flex items-center justify-between rounded-lg border border-border/60 px-3 py-2">
          <div>
            <p className="text-sm font-medium">Skip silence and noise</p>
            <p className="text-xs text-muted-foreground">
              Selah only pays attention when someone is actually talking.
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
    return <Panel title="Screen">Loading…</Panel>;
  }

  return (
    <Panel title="Screen for the congregation">
      <div className="grid gap-4 sm:grid-cols-2">
        <div className="space-y-1.5 sm:col-span-2">
          <Label htmlFor="presentation-display">Which screen</Label>
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
              <SelectValue placeholder="The main screen" />
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
          <Label htmlFor="background">Background colour</Label>
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
          <Label htmlFor="font-size">Text size</Label>
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
            <p className="text-sm font-medium">Fill the whole screen</p>
            <p className="text-xs text-muted-foreground">
              No window edges or buttons. Best for a projector.
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
            <p className="text-sm font-medium">Follow the Live screen</p>
            <p className="text-xs text-muted-foreground">
              Show whatever you send from Live, without pressing anything else.
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
            {open ? "screen is showing" : "screen is off"}
          </Badge>
          <Button
            variant="outline"
            size="sm"
            disabled={busy}
            onClick={() => void toggleWindow()}
          >
            {open ? "Hide the screen" : "Show the screen"}
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
    <Panel title="Bible text">
      <p className="text-sm text-muted-foreground">
        Everything Selah knows is kept in one file on this computer. Nothing is
        uploaded, and nothing is shared with anyone else.
      </p>

      <Separator className="my-4" />

      <h3 className="mb-2 text-xs font-semibold tracking-[0.14em] text-muted-foreground uppercase">
        Add Bible text
      </h3>
      <p className="mb-3 text-sm text-muted-foreground">
        Selah does not include any Bible text. Choose a file you are allowed to
        use. The expected format is described in{" "}
        <code className="font-mono">data/bible/README.md</code>.
      </p>
      <div className="flex flex-col gap-3 sm:flex-row sm:items-end">
        <div className="flex-1 space-y-1.5">
          <Label htmlFor="import-path">File on this computer</Label>
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
          Add
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
        Bible text you have added
      </h3>
      {translations.length === 0 ? (
        <p className="text-sm text-muted-foreground">
          Nothing added yet. Selah includes no Bible text — choose a file you
          are allowed to use.
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
          {
            label: "Where it is kept",
            value: "This computer's app data folder",
          },
          { label: "Internet needed", value: "Never" },
        ]}
      />
    </Panel>
  );
}

