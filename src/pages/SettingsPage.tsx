import { useCallback, useEffect, useState } from "react";
import {
  CheckCircle2,
  FolderOpen,
  HardDrive,
  Mic,
  Monitor,
  Palette,
  RotateCcw,
  Save,
  Sparkles,
  Upload,
} from "lucide-react";

import PageHeader, { EmptyHint, KeyValueList, Panel } from "@/components/PageHeader";
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
import { useSettings, type SettingsPatch } from "@/hooks/useSettings";
import { FONT_OPTIONS, isLightColor, resolveFontFamily } from "@/lib/fonts";
import { mediaUrl } from "@/lib/media";
import { audioApi, bibleApi, mediaApi, presentationApi, speechApi } from "@/lib/api";
import { EVENTS, useTauriEvent } from "@/lib/events";
import type {
  AppSettings,
  AudioDeviceInfo,
  BrandingPosition,
  BrandingSettings,
  DisplayInfo,
  MediaItem,
  SpeechManagerState,
  TranslationStatus,
} from "@/types";

/** Longest branding line, mirroring `MAX_BRANDING_TEXT` on the Rust side. */
const MAX_BRANDING_TEXT = 120;

/**
 * Settings.
 *
 * Everything is edited as a *draft* and written to SQLite when Save is pressed.
 * Saving is not just a database write: the projector window is told about the
 * new background, text size and typeface, and an open projector follows the
 * fullscreen and screen choices — so a Save is visibly real.
 */
export default function SettingsPage() {
  const { settings, error, save } = useSettings();
  const [draft, setDraft] = useState<AppSettings | null>(null);
  const [dirty, setDirty] = useState(false);
  const [saving, setSaving] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);

  // Follow the stored document until the operator starts editing; never
  // clobber half-finished edits with a background reload.
  useEffect(() => {
    if (settings && !dirty) {
      setDraft(settings);
    }
  }, [settings, dirty]);

  const update = useCallback((patch: SettingsPatch) => {
    setDraft((current) =>
      current
        ? {
            general: { ...current.general, ...patch.general },
            audio: { ...current.audio, ...patch.audio },
            speech: { ...current.speech, ...patch.speech },
            presentation: { ...current.presentation, ...patch.presentation },
            media: { ...current.media, ...patch.media },
          }
        : current,
    );
    setDirty(true);
    setNotice(null);
  }, []);

  async function saveDraft() {
    if (!draft) {
      return;
    }
    setSaving(true);
    try {
      if (await save(draft)) {
        setDirty(false);
        setNotice("Saved — the screen follows these settings straight away.");
      }
    } finally {
      setSaving(false);
    }
  }

  function discard() {
    setDraft(settings);
    setDirty(false);
    setNotice(null);
  }

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
          <TabsTrigger value="media">
            <FolderOpen className="size-3.5" /> Media
          </TabsTrigger>
          <TabsTrigger value="database">
            <HardDrive className="size-3.5" /> Bible text
          </TabsTrigger>
        </TabsList>

        <TabsContent value="general">
          <GeneralSection settings={draft} update={update} />
        </TabsContent>
        <TabsContent value="audio">
          <AudioSection settings={draft} update={update} />
        </TabsContent>
        <TabsContent value="speech">
          <SpeechSection settings={draft} update={update} />
        </TabsContent>
        <TabsContent value="presentation">
          <PresentationSection settings={draft} update={update} />
        </TabsContent>
        <TabsContent value="media">
          <MediaSection settings={draft} update={update} />
        </TabsContent>
        <TabsContent value="database">
          <DatabaseSection />
        </TabsContent>
      </Tabs>

      <SaveBar
        dirty={dirty}
        saving={saving}
        notice={notice}
        onSave={() => void saveDraft()}
        onDiscard={discard}
      />
    </>
  );
}

/** The always-visible Save row, so nothing is ever silently lost. */
function SaveBar({
  dirty,
  saving,
  notice,
  onSave,
  onDiscard,
}: {
  dirty: boolean;
  saving: boolean;
  notice: string | null;
  onSave: () => void;
  onDiscard: () => void;
}) {
  return (
    <div className="sticky bottom-0 z-10 -mx-4 flex flex-wrap items-center justify-between gap-3 border-t border-border/60 bg-background/95 px-4 py-3 backdrop-blur lg:-mx-5 lg:px-5">
      <p className="text-xs text-muted-foreground">
        {notice ? (
          <span className="flex items-center gap-1.5 text-success">
            <CheckCircle2 className="size-3.5" /> {notice}
          </span>
        ) : dirty ? (
          "You have changes that are not saved yet."
        ) : (
          "Everything here is saved."
        )}
      </p>
      <div className="flex items-center gap-2">
        <Button
          variant="ghost"
          size="sm"
          disabled={!dirty || saving}
          onClick={onDiscard}
        >
          <RotateCcw className="size-3.5" />
          Undo changes
        </Button>
        <Button
          variant="success"
          size="sm"
          disabled={!dirty || saving}
          onClick={onSave}
        >
          <Save className="size-3.5" />
          {saving ? "Saving…" : "Save settings"}
        </Button>
      </div>
    </div>
  );
}

type Settings = NonNullable<ReturnType<typeof useSettings>["settings"]>;

/** Applies a change to the draft document (nothing is stored until Save). */
type Update = (patch: SettingsPatch) => void;

// ---------------------------------------------------------------- general

function GeneralSection({ settings, update }: { settings: Settings | null; update: Update }) {
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
              update({ general: { appName: e.target.value } })
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
              update({ general: { theme } });
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
              update({ general: { defaultTranslationId: value } });
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

function AudioSection({ settings, update }: { settings: Settings | null; update: Update }) {
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
              update({ audio: { inputDeviceId: value } })
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
              update({ audio: { sampleRate: Number(e.target.value) || 0 } })
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

function SpeechSection({ settings, update }: { settings: Settings | null; update: Update }) {
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
              update({
                speech: { recognizer: recognizer as "mock" | "moonshine" },
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
              <SelectItem value="moonshine">
                On — words are worked out on this computer
              </SelectItem>
            </SelectContent>
          </Select>
          <p className="text-xs text-muted-foreground">
            {state?.recognizerId === "moonshine"
              ? "Words from the microphone are turned into text on this machine."
              : "Turned off. Selah will not invent words it did not hear."}
          </p>
        </div>

        <div className="space-y-1.5">
          <Label htmlFor="model-path">Voice model folder</Label>
          <Input
            id="model-path"
            value={settings.speech.modelPath ?? ""}
            onChange={(e) =>
              update({ speech: { modelPath: e.target.value || undefined } })
            }
            placeholder="…/models/moonshine"
          />
          <p className="text-xs text-muted-foreground">
            The folder holding the files that teach Selah English. Selah never
            downloads them for you — point at a folder you already have.
          </p>
          {settings.speech.recognizer === "moonshine" &&
          state &&
          !state.modelLoaded ? (
            <p className="rounded-md border border-warning/30 bg-warning/10 px-2.5 py-1.5 text-xs text-warning">
              This folder could not be opened, so words are not being written
              down yet. Check the whole path is written out and the files are
              really there.
            </p>
          ) : null}
        </div>

        <div className="space-y-1.5">
          <Label htmlFor="language">Language spoken</Label>
          <Input
            id="language"
            value={settings.speech.language ?? ""}
            onChange={(e) =>
              update({ speech: { language: e.target.value || undefined } })
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
              update({
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
              update({ speech: { threads: Number(e.target.value) || 1 } })
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
              update({ speech: { vadEnabled } })
            }
          />
        </div>
      </div>
    </Panel>
  );
}

// ------------------------------------------------------------ presentation

/** Smallest / largest text size the projector offers (readable at the back). */
const MIN_FONT_SIZE = 16;
const MAX_FONT_SIZE = 200;

function PresentationSection({
  settings,
  update,
}: {
  settings: Settings | null;
  update: Update;
}) {
  const [displays, setDisplays] = useState<DisplayInfo[]>([]);
  const [open, setOpen] = useState(false);
  const [busy, setBusy] = useState(false);
  // Pictures the operator already has, offered as logo choices — Selah never
  // guesses at a path, and there is no separate file dialog to learn.
  const [logoChoices, setLogoChoices] = useState<MediaItem[]>([]);

  useEffect(() => {
    presentationApi
      .listDisplays()
      .then(setDisplays)
      .catch(() => setDisplays([]));
  }, []);

  useEffect(() => {
    mediaApi
      .listMedia()
      .then((files) =>
        setLogoChoices(
          files.filter(
            (file) =>
              file.kind === "image" || /\.(png|jpe?g|gif|webp|svg)$/i.test(file.path),
          ),
        ),
      )
      .catch(() => setLogoChoices([]));
  }, []);

  // Track the projector window from the events the Rust display manager emits,
  // so the badge stays correct even if the screen was opened from elsewhere.
  useTauriEvent(EVENTS.presentationDisplayOpened, () => setOpen(true));
  useTauriEvent(EVENTS.presentationDisplayClosed, () => setOpen(false));

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

  const { presentation } = settings;
  const fontFamily = resolveFontFamily(presentation.fontFamily);
  const light = isLightColor(presentation.background);
  const textColor = light ? "#141B2E" : "#FFFFFF";
  // A settings document saved before branding existed has no `branding` key,
  // so the fields are read defensively rather than trusted.
  const branding: BrandingSettings = presentation.branding ?? {
    position: "bottom",
    sizePercent: 30,
  };
  const brandText = branding.text?.trim();
  const brandLogo = branding.logo?.trim();
  const hasBranding = Boolean(brandText || brandLogo);

  /** Size the branding is drawn at on the projector, in CSS pixels. */
  const brandTextPx = Math.max(
    8,
    Math.round((presentation.fontSize * branding.sizePercent) / 100),
  );
  /**
   * Size used inside the settings panel. A 60%-of-200px brand name is 120px on
   * the wall, which would not fit a settings card, so it is shown smaller — and
   * the line underneath says so.
   */
  const brandPreviewPx = Math.min(brandTextPx, 40);

  return (
    <>
      <Panel title="Screen for the congregation">
        <div className="grid gap-4 sm:grid-cols-2">
          <div className="space-y-1.5 sm:col-span-2">
            <Label htmlFor="presentation-display">Which screen</Label>
            <Select
              value={
                presentation.displayIndex === undefined
                  ? undefined
                  : String(presentation.displayIndex)
              }
              onValueChange={(value) =>
                update({ presentation: { displayIndex: Number(value) } })
              }
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
            <p className="text-xs text-muted-foreground">
              Applied when you press Save settings.
            </p>
          </div>

          <div className="space-y-1.5">
            <Label htmlFor="background">Background colour</Label>
            <div className="flex items-center gap-2">
              <input
                id="background"
                type="color"
                value={
                  /^#[0-9a-f]{6}$/i.test(presentation.background)
                    ? presentation.background
                    : "#000000"
                }
                onChange={(e) =>
                  update({ presentation: { background: e.target.value } })
                }
                className="h-9 w-12 shrink-0 cursor-pointer rounded-md border border-border/60 bg-transparent p-1"
              />
              <Input
                aria-label="Background colour code"
                value={presentation.background}
                onChange={(e) =>
                  update({ presentation: { background: e.target.value } })
                }
                placeholder="#000000"
              />
            </div>
            <p className="text-xs text-muted-foreground">
              Letters turn dark by themselves on a pale colour.
            </p>
          </div>

          <div className="space-y-1.5">
            <Label htmlFor="font-size">
              Text size — {presentation.fontSize}px
            </Label>
            <input
              id="font-size"
              type="range"
              min={MIN_FONT_SIZE}
              max={MAX_FONT_SIZE}
              step={2}
              value={presentation.fontSize}
              onChange={(e) =>
                update({ presentation: { fontSize: Number(e.target.value) } })
              }
              className="h-9 w-full cursor-pointer accent-[var(--brand)]"
            />
            <Input
              type="number"
              min={MIN_FONT_SIZE}
              max={MAX_FONT_SIZE}
              value={presentation.fontSize}
              aria-label="Text size in pixels"
              onChange={(e) => {
                const next = Number(e.target.value) || MIN_FONT_SIZE;
                update({
                  presentation: {
                    fontSize: Math.min(
                      Math.max(next, MIN_FONT_SIZE),
                      MAX_FONT_SIZE,
                    ),
                  },
                });
              }}
            />
          </div>

          <div className="space-y-1.5 sm:col-span-2">
            <Label htmlFor="font-family">Typeface</Label>
            {/*
              Every option is drawn in its own typeface, so the choice can be
              judged by eye rather than by name.
            */}
            <Select
              value={fontFamily}
              onValueChange={(value) =>
                update({ presentation: { fontFamily: value } })
              }
            >
              <SelectTrigger id="font-family">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {FONT_OPTIONS.map((font) => (
                  <SelectItem
                    key={font.family}
                    value={font.family}
                    style={{ fontFamily: `"${font.family}"` }}
                  >
                    {font.label} · {font.description}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="flex items-center justify-between rounded-lg border border-border/60 px-3 py-2">
            <div>
              <p className="text-sm font-medium">Fill the whole screen</p>
              <p className="text-xs text-muted-foreground">
                No window edges or buttons. Best for a projector.
              </p>
            </div>
            <Switch
              checked={presentation.fullscreen}
              onCheckedChange={(fullscreen) =>
                update({ presentation: { fullscreen } })
              }
            />
          </div>

          <div className="flex items-center justify-between rounded-lg border border-border/60 px-3 py-2">
            <div>
              <p className="text-sm font-medium">Follow the Live screen</p>
              <p className="text-xs text-muted-foreground">
                Show whatever you send from Live, without pressing anything
                else.
              </p>
            </div>
            <Switch
              checked={presentation.followLive}
              onCheckedChange={(followLive) =>
                update({ presentation: { followLive } })
              }
            />
          </div>

          <div className="flex items-center justify-between rounded-lg border border-border/60 px-3 py-2">
            <div>
              <p className="text-sm font-medium">
                Start videos again at the end
              </p>
              <p className="text-xs text-muted-foreground">
                When a video reaches its end — or the end of the time range
                chosen for it on the Media screen — it plays again. Turn this
                off and it stops on its last frame. A file can be set the other
                way from its Preview button on Media.
              </p>
            </div>
            <Switch
              checked={presentation.repeatVideos}
              onCheckedChange={(repeatVideos) =>
                update({ presentation: { repeatVideos } })
              }
            />
          </div>

          <div className="space-y-3 rounded-lg border border-border/60 p-3 sm:col-span-2">
            <div className="flex flex-wrap items-center justify-between gap-2">
              <div>
                <p className="text-sm font-medium">Your logo and name</p>
                <p className="text-xs text-muted-foreground">
                  Drawn over everything you show, so the screen looks like your
                  church&apos;s. Leave both blank for no branding at all.
                </p>
              </div>
              <Badge variant={hasBranding ? "success" : "muted"}>
                {hasBranding ? "branding on" : "no branding"}
              </Badge>
            </div>

            <div className="grid gap-3 sm:grid-cols-2">
              <div className="space-y-1.5">
                <Label htmlFor="brand-text">Words to show</Label>
                <Input
                  id="brand-text"
                  value={branding.text ?? ""}
                  onChange={(e) =>
                    update({
                      presentation: {
                        branding: { ...branding, text: e.target.value },
                      },
                    })
                  }
                  placeholder="Grace Chapel"
                  maxLength={MAX_BRANDING_TEXT}
                />
                <p className="text-xs text-muted-foreground">
                  Usually the church name. Kept short on purpose — it is not
                  words to be read aloud.
                </p>
              </div>

              <div className="space-y-1.5">
                <Label htmlFor="brand-logo">Logo</Label>
                <Select
                  value={branding.logo ?? ""}
                  onValueChange={(logo) =>
                    update({
                      presentation: {
                        branding: { ...branding, logo: logo || undefined },
                      },
                    })
                  }
                >
                  <SelectTrigger id="brand-logo">
                    <SelectValue placeholder="No logo" />
                  </SelectTrigger>
                  <SelectContent>
                    {logoChoices.map((file) => (
                      <SelectItem key={file.path} value={file.path}>
                        {file.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <Input
                  aria-label="Logo file path"
                  value={branding.logo ?? ""}
                  onChange={(e) =>
                    update({
                      presentation: {
                        branding: {
                          ...branding,
                          logo: e.target.value || undefined,
                        },
                      },
                    })
                  }
                  placeholder="/Users/you/logo.png"
                />
                <p className="text-xs text-muted-foreground">
                  Pick a picture from your media library, or type the whole path
                  to one. PNG with a transparent background looks best.
                </p>
              </div>
            </div>

            <div className="grid gap-3 sm:grid-cols-2">
              <div className="space-y-1.5">
                <Label htmlFor="brand-position">Where it sits</Label>
                <Select
                  value={branding.position}
                  onValueChange={(position) =>
                    update({
                      presentation: {
                        branding: {
                          ...branding,
                          position: position as BrandingPosition,
                        },
                      },
                    })
                  }
                >
                  <SelectTrigger id="brand-position">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="bottom">Along the bottom</SelectItem>
                    <SelectItem value="top">Along the top</SelectItem>
                    <SelectItem value="left">Down the left side</SelectItem>
                    <SelectItem value="right">Down the right side</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              <div className="space-y-1.5">
                <Label htmlFor="brand-size">
                  How big — {branding.sizePercent}% of the words
                </Label>
                <input
                  id="brand-size"
                  type="range"
                  min={10}
                  max={60}
                  step={5}
                  value={branding.sizePercent}
                  onChange={(e) =>
                    update({
                      presentation: {
                        branding: {
                          ...branding,
                          sizePercent: Number(e.target.value),
                        },
                      },
                    })
                  }
                  className="h-9 w-full cursor-pointer accent-[var(--brand)]"
                />
              </div>
            </div>

            {/*
              What the branding will actually look like: the chosen image, and
              the church name drawn in the typeface and at the relative size the
              projector will use. Seeing the logo here also proves the file can
              be read — a mistyped path shows as a broken image.
            */}
            <div className="space-y-2">
              <p className="text-sm font-medium">What the branding looks like</p>
              <div
                className="flex flex-wrap items-center gap-3 rounded-lg border border-border/60 px-3 py-3"
                style={{
                  background: presentation.background,
                  color: textColor,
                  fontFamily: `"${fontFamily}"`,
                }}
              >
                {brandLogo ? (
                  <img
                    src={mediaUrl(brandLogo)}
                    alt="The logo that will be shown on the screen"
                    className="max-h-16 max-w-40 object-contain"
                  />
                ) : (
                  <span
                    className="flex h-16 w-40 items-center justify-center rounded-md border border-dashed text-xs"
                    style={{ borderColor: "currentColor", opacity: 0.55 }}
                  >
                    no logo chosen
                  </span>
                )}

                {brandText ? (
                  <span
                    className="font-medium tracking-[0.22em] uppercase"
                    style={{ fontSize: `${brandPreviewPx}px` }}
                  >
                    {brandText}
                  </span>
                ) : (
                  <span className="text-xs" style={{ opacity: 0.55 }}>
                    no words typed
                  </span>
                )}
              </div>
              <p className="text-xs text-muted-foreground">
                {brandText
                  ? `${fontFamily} · ${brandTextPx}px (${branding.sizePercent}% of the ${presentation.fontSize}px words)`
                  : `${fontFamily} · nothing to draw yet`}
                {brandPreviewPx < brandTextPx
                  ? " · shown smaller here so it fits the panel"
                  : ""}
              </p>
            </div>
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

      <Panel title="How it will look">
        <div
          aria-label="Preview of the projected screen"
          className="relative grid place-items-center overflow-hidden rounded-lg border border-border/60 px-[6%] py-[6%]"
          style={{
            background: presentation.background,
            color: textColor,
            fontFamily: `"${fontFamily}"`,
            minHeight: "9rem",
          }}
        >
          <div className="text-center">
            {/* Scaled down so a whole sample fits the settings panel. */}
            <p
              className="font-semibold uppercase tracking-[0.12em]"
              style={{ fontSize: `${presentation.fontSize * 0.045}rem` }}
            >
              John 3:16
            </p>
            <p
              className="font-light"
              style={{ fontSize: `${presentation.fontSize * 0.035}rem` }}
            >
              For God so loved the world…
            </p>
          </div>

          {/*
            The overlay is shown in the same place it will appear on the real
            screen, at the same relative size.
          */}
          {hasBranding ? (
            <div
              className={`absolute flex items-center gap-1.5 ${
                branding.position === "top" || branding.position === "bottom"
                  ? "inset-x-0 justify-center"
                  : "inset-y-0 flex-col justify-center"
              } ${branding.position === "top" ? "top-0" : ""} ${
                branding.position === "bottom" ? "bottom-0" : ""
              } ${branding.position === "left" ? "left-0" : ""} ${
                branding.position === "right" ? "right-0" : ""
              }`}
              style={{
                opacity: 0.9,
                padding: "0.5rem 0.75rem",
                fontSize: `${presentation.fontSize * 0.02 * (branding.sizePercent / 30)}rem`,
              }}
            >
              {brandLogo ? (
                <img
                  src={mediaUrl(brandLogo)}
                  alt=""
                  className="max-h-[1.6em] max-w-[6em] object-contain"
                />
              ) : null}
              {brandText ? (
                <span className="font-medium tracking-[0.22em] uppercase">
                  {brandText}
                </span>
              ) : null}
            </div>
          ) : null}
        </div>
        <p className="mt-2 text-xs text-muted-foreground">
          {fontFamily} · {presentation.fontSize}px · {presentation.background}
          {light ? " · dark letters" : " · light letters"}
          {hasBranding
            ? ` · branding ${branding.position}, ${branding.sizePercent}%`
            : " · no branding"}
        </p>
      </Panel>
    </>
  );
}

// ------------------------------------------------------------------- media

/**
 * Media files.
 *
 * The folder itself is chosen on the Media screen, which is where files are
 * browsed and loaded; this panel shows what was loaded and re-reads the folder
 * after new files have been added to it.
 */
function MediaSection({
  settings,
  update,
}: {
  settings: Settings | null;
  update: Update;
}) {
  const [items, setItems] = useState<MediaItem[]>([]);
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const directory = settings?.media.directory;
  const recursive = settings?.media.recursive ?? false;

  useEffect(() => {
    mediaApi
      .listMedia()
      .then(setItems)
      .catch(() => setItems([]));
  }, []);

  async function reloadFolder() {
    if (!directory) {
      return;
    }
    setBusy(true);
    try {
      const scan = await mediaApi.loadMediaDirectory(directory, recursive);
      setItems(scan.items);
      setError(null);
      setNotice(
        `${scan.total} file${scan.total === 1 ? "" : "s"} ready${
          scan.added > 0 ? ` (${scan.added} new)` : ""
        }.`,
      );
    } catch (e: unknown) {
      setNotice(null);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  if (!settings) {
    return <Panel title="Media files">Loading…</Panel>;
  }

  return (
    <Panel title="Media files">
      <div className="space-y-4">
        <div className="flex flex-col gap-3 sm:flex-row sm:items-end">
          <div className="flex-1 space-y-1.5">
            <Label htmlFor="media-folder">
              Folder Selah reads pictures and videos from
            </Label>
            <Input
              id="media-folder"
              value={directory ?? ""}
              readOnly
              placeholder="No folder chosen yet — pick one on the Media screen"
            />
          </div>
          <Button
            variant="outline"
            disabled={busy || !directory}
            onClick={() => void reloadFolder()}
          >
            <FolderOpen className="size-4" />
            Read it again
          </Button>
        </div>

        <div className="flex items-center justify-between rounded-lg border border-border/60 px-3 py-2">
          <div>
            <p className="text-sm font-medium">Include sub-folders</p>
            <p className="text-xs text-muted-foreground">
              Also look inside folders within the one you picked.
            </p>
          </div>
          <Switch
            checked={recursive}
            onCheckedChange={(value) => update({ media: { recursive: value } })}
          />
        </div>
        <p className="text-xs text-muted-foreground">
          Press Save settings after changing this, then use “Read it again”.
        </p>

        {notice ? (
          <p className="rounded-md border border-success/30 bg-success/10 px-3 py-2 text-sm text-success">
            {notice}
          </p>
        ) : null}
        {error ? (
          <p className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
            {error}
          </p>
        ) : null}

        <Separator />

        {items.length === 0 ? (
          <EmptyHint>
            Nothing loaded yet. Open the Media screen, choose a folder, and Selah
            will read the pictures and videos inside it.
          </EmptyHint>
        ) : (
          <ul className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
            {items.slice(0, 9).map((item) => (
              <li
                key={item.id}
                className="overflow-hidden rounded-lg border border-border/60"
              >
                {item.kind === "image" ? (
                  <img
                    src={mediaUrl(item.path)}
                    alt={item.name}
                    className="h-24 w-full bg-black object-contain"
                  />
                ) : (
                  <div className="grid h-24 w-full place-items-center bg-muted text-xs tracking-[0.12em] text-muted-foreground uppercase">
                    {friendlyMediaKind(item.kind)}
                  </div>
                )}
                <p className="truncate px-2 py-1.5 text-xs">{item.name}</p>
              </li>
            ))}
          </ul>
        )}
        {items.length > 9 ? (
          <p className="text-xs text-muted-foreground">
            Showing 9 of {items.length} files — the rest are on the Media screen.
          </p>
        ) : null}
      </div>
    </Panel>
  );
}

/** Plain-language name for a media kind. */
function friendlyMediaKind(kind: string): string {
  if (kind === "video") {
    return "video";
  }
  if (kind === "audio") {
    return "sound";
  }
  return "picture";
}

// ---------------------------------------------------------------- database

/** Full names for the translations people most often have on disk. */
const KNOWN_TRANSLATIONS: Record<string, string> = {
  web: "World English Bible",
  kjv: "King James Version",
  asv: "American Standard Version",
};

/**
 * Derives a short id and a readable name from a file path.
 *
 * Saves the operator from typing metadata: `…/kjv.sqlite` becomes
 * `{ id: "kjv", name: "King James Version" }`.
 */
function describeFile(path: string): { id: string; name: string } {
  const stem = (path.split(/[\\/]/).pop() ?? "")
    .replace(/\.(sqlite3?|db|json)$/i, "")
    .toLowerCase();
  const id =
    stem.replace(/[^a-z0-9]+/g, "_").replace(/^_+|_+$/g, "") || "imported";
  return {
    id,
    name:
      KNOWN_TRANSLATIONS[id] ??
      stem.replace(/[-_]+/g, " ").replace(/\b\w/g, (c) => c.toUpperCase()),
  };
}

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
    const path = importPath.trim();
    if (path.length === 0) {
      return;
    }
    setImporting(true);
    try {
      // The file extension decides which reader to use.
      const isSqlite = /\.(sqlite3?|db)$/i.test(path);
      const result = isSqlite
        ? await bibleApi.importSqliteBibleTranslation({
            path,
            translationId: describeFile(path).id,
            name: describeFile(path).name,
          })
        : await bibleApi.importBibleTranslation(path);

      setNotice(
        `Added ${result.versesImported.toLocaleString()} verses as “${result.translationId}”.`,
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
        Selah already includes three public-domain translations — WEB, KJV and
        ASV. To add another, point at a file you are allowed to use: either a{" "}
        <code className="font-mono">.sqlite</code> Bible or a{" "}
        <code className="font-mono">.json</code> document. Both shapes are
        described in <code className="font-mono">data/bible/README.md</code>.
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
            placeholder="…/Downloads/kjv.sqlite"
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
        Bible text available
      </h3>
      {translations.length === 0 ? (
        <p className="text-sm text-muted-foreground">
          None yet. Restart Selah to install the built-in translations, or add a
          file above.
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

