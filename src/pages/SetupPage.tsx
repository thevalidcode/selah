import { useEffect, useState } from "react";
import { BookOpen, CheckCircle2, Loader2, Mic, Monitor } from "lucide-react";

import { Button } from "@/components/ui/button";
import { SelahIcon } from "@/components/ui/selah-icon";
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
        <header className="space-y-3 text-center">
          <SelahIcon
            size={64}
            tile
            title="Selah"
            className="mx-auto rounded-2xl"
          />
          <div className="text-2xl font-black tracking-[0.3em]">SELAH</div>
          <h1 className="text-lg font-bold">Welcome</h1>
          <p className="mx-auto max-w-md text-sm text-muted-foreground">
            Three quick choices and you are ready to go. You can change any of
            them later in Settings.
          </p>
        </header>
        <Card>
          <CardHeader>
            <div className="flex items-center gap-2">
              <Mic className="size-4 text-muted-foreground" />
              <CardTitle>Microphone</CardTitle>
            </div>
            <CardDescription>
              Selah listens on this computer. Your sound never leaves it.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-2">
            <Label htmlFor="setup-mic">Which microphone</Label>
            {loading ? (
              <LoadingRow />
            ) : devices.length === 0 ? (
              <p className="text-sm text-muted-foreground">
                No microphones found. You can choose one later in Settings.
              </p>
            ) : (
              <Select
                value={defaultDeviceId}
                onValueChange={(value) =>
                  void save({ audio: { inputDeviceId: value } })
                }
              >
                <SelectTrigger id="setup-mic">
                  <SelectValue placeholder="Pick a microphone" />
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
              <CardTitle>Bible text</CardTitle>
            </div>
            <CardDescription>
              Add the Bible text you are allowed to use. Selah does not include
              any of its own.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-2">
            <Label htmlFor="setup-translation">Bible to use</Label>
            {loading ? (
              <LoadingRow />
            ) : translations.length === 0 ? (
              <p className="text-sm text-muted-foreground">
                Nothing added yet. You can add it later in Settings.
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
                  <SelectValue placeholder="Pick a Bible" />
                </SelectTrigger>
                <SelectContent>
                  {translations.map(({ translation, verseCount }) => (
                    <SelectItem key={translation.id} value={translation.id}>
                      {translation.name}
                      {translation.abbreviation
                        ? ` (${translation.abbreviation})`
                        : ""}
                      {verseCount === 0 ? " — no text added yet" : ""}
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
              <CardTitle>Screen for the congregation</CardTitle>
            </div>
            <CardDescription>
              The screen everyone else sees. Leave this blank to use the main
              screen.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-2">
            <Label htmlFor="setup-display">Which screen</Label>
            {loading ? (
              <LoadingRow />
            ) : displays.length === 0 ? (
              <p className="text-sm text-muted-foreground">
                No other screens found. The main screen will be used.
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
                  <SelectValue placeholder="Pick a screen" />
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
            No account needed, nothing to sign up for
          </p>
          <Button onClick={() => void finish()} disabled={finishing}>
            {finishing ? <Loader2 className="size-4 animate-spin" /> : null}
            Get started
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
      Looking…
    </div>
  );
}
