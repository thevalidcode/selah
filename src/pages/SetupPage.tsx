import { useEffect, useState } from "react";
import { BookOpen, CheckCircle2, Loader2, Mic, Monitor } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { audioApi, bibleApi, presentationApi, settingsApi } from "@/lib/api";
import { useSettings } from "@/hooks/useSettings";
import type { AudioDeviceInfo, DisplayInfo, TranslationStatus } from "@/types";

/**
 * First-run experience.
 *
 * Deliberately short: pick a microphone, a Bible translation and a
 * presentation display, then continue. Everything is editable later in
 * Settings, and each step is individually skippable — the operator is never
 * blocked from reaching the Live screen.
 */
export default function SetupPage({ onDone }: { onDone: () => void }) {
  const { settings, save } = useSettings();
  const [devices, setDevices] = useState<AudioDeviceInfo[]>([]);
  const [translations, setTranslations] = useState<TranslationStatus[]>([]);
  const [displays, setDisplays] = useState<DisplayInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [finishing, setFinishing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([
      audioApi.listAudioDevices().catch(() => [] as AudioDeviceInfo[]),
      bibleApi.listBibleTranslations().catch(() => [] as TranslationStatus[]),
      presentationApi.listDisplays().catch(() => [] as DisplayInfo[]),
    ])
      .then(([d, t, s]) => {
        setDevices(d);
        setTranslations(t);
        setDisplays(s);
      })
      .finally(() => setLoading(false));
  }, []);

  const defaultDeviceId =
    settings?.audio.inputDeviceId ??
    devices.find((d) => d.isDefault)?.id ??
    undefined;

  const defaultTranslationId =
    settings?.general.defaultTranslationId ??
    translations.find((t) => t.translation.isDefault)?.translation.id ??
    translations[0]?.translation.id ??
    undefined;

  async function finish() {
    setFinishing(true);
    try {
      await settingsApi.completeSetup();
      onDone();
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setFinishing(false);
    }
  }

  return (
    <div className="grid h-full place-items-center overflow-y-auto bg-background p-6">
      <div className="w-full max-w-2xl space-y-5 py-8">
        <header className="space-y-2 text-center">
          <div className="text-3xl font-semibold tracking-[0.35em]">SELAH</div>
          <h1 className="text-lg font-medium">Welcome</h1>
          <p className="text-sm text-muted-foreground">
            Set up your presentation environment. You can change any of this
            later in Settings.
          </p>
        </header>
        <Card>
          <CardHeader>
            <div className="flex items-center gap-2">
              <Mic className="size-4 text-muted-foreground" />
              <CardTitle>Microphone</CardTitle>
            </div>
            <CardDescription>
              Selah listens locally — audio never leaves this machine.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-2">
            <Label htmlFor="setup-mic">Input device</Label>
            {loading ? (
              <LoadingRow />
            ) : devices.length === 0 ? (
              <p className="text-sm text-muted-foreground">
                No input devices were detected. Capture can be configured later.
              </p>
            ) : (
              <Select
                value={defaultDeviceId}
                onValueChange={(value) =>
                  void save({ audio: { inputDeviceId: value } })
                }
              >
                <SelectTrigger id="setup-mic">
                  <SelectValue placeholder="Select a microphone" />
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
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <div className="flex items-center gap-2">
              <BookOpen className="size-4 text-muted-foreground" />
              <CardTitle>Bible translation</CardTitle>
            </div>
            <CardDescription>
              Import the translation you are licensed to use — Selah ships no
              Bible text.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-2">
            <Label htmlFor="setup-translation">Default translation</Label>
            {loading ? (
              <LoadingRow />
            ) : translations.length === 0 ? (
              <p className="text-sm text-muted-foreground">
                No translations installed yet. Add one from Settings → Database.
              </p>
            ) : (
              <Select
                value={defaultTranslationId}
                onValueChange={async (value) => {
                  try {
                    await bibleApi.setDefaultTranslation(value);
                    await save({ general: { defaultTranslationId: value } });
                  } catch (e: unknown) {
                    setError(e instanceof Error ? e.message : String(e));
                  }
                }}
              >
                <SelectTrigger id="setup-translation">
                  <SelectValue placeholder="Select a translation" />
                </SelectTrigger>
                <SelectContent>
                  {translations.map(({ translation, verseCount }) => (
                    <SelectItem key={translation.id} value={translation.id}>
                      {translation.name}
                      {translation.abbreviation
                        ? ` (${translation.abbreviation})`
                        : ""}
                      {verseCount === 0 ? " — metadata only" : ""}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <div className="flex items-center gap-2">
              <Monitor className="size-4 text-muted-foreground" />
              <CardTitle>Presentation display</CardTitle>
            </div>
            <CardDescription>
              Where the congregation sees content. Leave blank for the primary
              display.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-2">
            <Label htmlFor="setup-display">Display</Label>
            {loading ? (
              <LoadingRow />
            ) : displays.length === 0 ? (
              <p className="text-sm text-muted-foreground">
                No displays reported by the operating system.
              </p>
            ) : (
              <Select
                value={
                  settings?.presentation.displayIndex !== undefined
                    ? String(settings.presentation.displayIndex)
                    : undefined
                }
                onValueChange={(value) =>
                  void save({ presentation: { displayIndex: Number(value) } })
                }
              >
                <SelectTrigger id="setup-display">
                  <SelectValue placeholder="Select a display" />
                </SelectTrigger>
                <SelectContent>
                  {displays.map((display) => (
                    <SelectItem
                      key={display.index}
                      value={String(display.index)}
                    >
                      {display.name ?? `Display ${display.index + 1}`} ·{" "}
                      {display.size[0]}×{display.size[1]}
                      {display.isPrimary ? " (primary)" : ""}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            )}
          </CardContent>
        </Card>

        {error ? (
          <p className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
            {error}
          </p>
        ) : null}

        <div className="flex items-center justify-between gap-3 pb-2">
          <p className="flex items-center gap-1.5 text-xs text-muted-foreground">
            <CheckCircle2 className="size-3.5" />
            Offline · no account · no API keys
          </p>
          <Button onClick={() => void finish()} disabled={finishing}>
            {finishing ? <Loader2 className="size-4 animate-spin" /> : null}
            Continue
          </Button>
        </div>
      </div>
    </div>
  );
}

/** Inline placeholder shown while device/translation/display discovery runs. */
function LoadingRow() {
  return (
    <div className="flex h-9 items-center gap-2 text-sm text-muted-foreground">
      <Loader2 className="size-3.5 animate-spin" />
      Detecting…
    </div>
  );
}
