import { useEffect, useRef, useState } from "react";
import { MonitorPlay, Play } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import { clipEndAction, formatClipTime, mediaUrl } from "@/lib/media";
import type { MediaClip, MediaItem } from "@/types";

/**
 * Preview a video and choose the part of it that goes on the screen.
 *
 * A service rarely wants a whole clip, and nothing about a file says which
 * seconds matter — so the operator watches it here, sets a start and an end, and
 * decides whether the chosen part starts again at its end or stops there. The
 * choice can be saved with the file, which is what makes "Show" on the Media
 * screen behave the same way next Sunday without setting it up again.
 *
 * The preview plays the *chosen part* using the same rule as the projector
 * (`clipEndAction`), so what the operator watches here really is what the
 * congregation will see.
 */
export default function VideoClipDialog({
  item,
  savedClip,
  repeatByDefault,
  open,
  busy,
  onOpenChange,
  onSave,
  onShow,
}: {
  /** The video being previewed; `null` while no dialog is open. */
  item: MediaItem | null;
  /** The range already saved for this file, if the operator chose one. */
  savedClip?: MediaClip;
  /** Settings → Screen: stop, or start again, when a video reaches the end. */
  repeatByDefault: boolean;
  open: boolean;
  busy: boolean;
  onOpenChange: (open: boolean) => void;
  /** Remembers the range for this file; `null` forgets it. */
  onSave: (clip: MediaClip | null) => void;
  /** Shows the video now, using what is on the sliders. */
  onShow: (clip: MediaClip) => void;
}) {
  const preview = useRef<HTMLVideoElement | null>(null);
  /** Which file the sliders were last loaded for, so saving does not blank them. */
  const loadedFor = useRef<string | null>(null);

  // The draft, in milliseconds. An undefined `endMs` means "run to the end".
  const [startMs, setStartMs] = useState(0);
  const [endMs, setEndMs] = useState<number | undefined>(undefined);
  const [repeat, setRepeat] = useState(repeatByDefault);
  const [durationMs, setDurationMs] = useState(0);

  const savedStartMs = savedClip?.startMs ?? 0;
  const savedEndMs = savedClip?.endMs;
  const savedRepeat = savedClip?.repeat ?? repeatByDefault;

  // Opening on a file starts from what is saved for it, so the dialog always
  // shows the operator the truth about what Show will do.
  useEffect(() => {
    if (!open || !item) {
      loadedFor.current = null;
      return;
    }
    setStartMs(savedStartMs);
    setEndMs(savedEndMs);
    setRepeat(savedRepeat);

    // A different file starts with an unknown length until its metadata arrives.
    // Saving a range for the *same* file must not blank the sliders back to
    // "working out how long it is".
    if (loadedFor.current !== item.id) {
      loadedFor.current = item.id;
      setDurationMs(0);
    }
  }, [open, item, savedStartMs, savedEndMs, savedRepeat]);

  const wholeFile = startMs <= 0 && endMs === undefined;
  const endOfRange = Math.min(endMs ?? durationMs, durationMs || endMs || 0);
  const rangeLengthMs = Math.max(0, (endMs ?? durationMs) - startMs);

  /** The draft as it will be sent, including a whole-file stop/repeat choice. */
  function draftClip(): MediaClip {
    return { startMs, endMs, repeat };
  }

  /** Where the preview has got to, so a range can be set by ear. */
  function takePreviewPosition(setter: (ms: number) => void) {
    const element = preview.current;
    if (element) {
      setter(Math.round(element.currentTime * 1000));
    }
  }

  /** Keeps the range in order: an end before the start is not a range. */
  function changeStart(ms: number) {
    setStartMs(ms);
    if (endMs !== undefined && endMs <= ms) {
      setEndMs(undefined);
    }
  }

  /** Plays only the chosen part, exactly as the projector will. */
  function playSelection() {
    const element = preview.current;
    if (!element) {
      return;
    }
    element.currentTime = startMs / 1000;
    void element.play().catch(() => undefined);
  }

  /** Holds the preview inside the range, the same rule the projector applies. */
  function holdWithinRange(element: HTMLVideoElement) {
    const clip = draftClip();
    const lengthMs = Number.isFinite(element.duration)
      ? element.duration * 1000
      : 0;
    const action = clipEndAction(element.currentTime * 1000, clip, lengthMs);
    if (action === "restart") {
      element.currentTime = clip.startMs / 1000;
      void element.play().catch(() => undefined);
    } else if (action === "end") {
      // Stopped at the end of the range: the operator is left looking at the
      // exact frame the congregation will be left with.
      element.pause();
      element.currentTime = (clip.endMs ?? element.currentTime * 1000) / 1000;
    }
  }

  const endLabel =
    endMs === undefined || durationMs === 0 ? "the end" : formatClipTime(endMs);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-3xl overflow-y-auto max-h-[90vh] pr-4">
        <DialogHeader>
          <DialogTitle>Preview and choose the part to show</DialogTitle>
          <DialogDescription>
            Watch the video, set where it should start and stop, then put it on
            the screen. Save the range and Show uses it from then on.
          </DialogDescription>
        </DialogHeader>

        {item ? (
          <div className="space-y-3">
            <video
              ref={preview}
              // Operator-only preview. The congregation's screen is a separate
              // window that plays the file itself.
              className="max-h-[40vh] w-full rounded-lg bg-black"
              src={mediaUrl(item.path)}
              controls
              preload="metadata"
              onLoadedMetadata={(event) => {
                const element = event.currentTarget;
                setDurationMs(
                  Number.isFinite(element.duration) ? element.duration * 1000 : 0,
                );
                if (startMs > 0) {
                  element.currentTime = startMs / 1000;
                }
              }}
              onTimeUpdate={(event) => holdWithinRange(event.currentTarget)}
            />

            <p className="text-xs text-muted-foreground">
              {item.name}
              {durationMs > 0
                ? ` · ${formatClipTime(durationMs)} long`
                : " · working out how long it is…"}
            </p>

            <Separator />

            <div className="space-y-1.5">
              <Label htmlFor="clip-start">
                Starts at — {formatClipTime(startMs)}
              </Label>
              <input
                id="clip-start"
                type="range"
                min={0}
                max={Math.max(durationMs, startMs)}
                step={100}
                value={startMs}
                disabled={durationMs === 0}
                onChange={(event) => changeStart(Number(event.target.value))}
                className="h-9 w-full cursor-pointer accent-[var(--brand)]"
              />
              <div className="flex items-center gap-2">
                <Input
                  type="number"
                  min={0}
                  aria-label="Start in seconds"
                  value={Math.round(startMs / 1000)}
                  onChange={(event) =>
                    changeStart(
                      Math.max(0, Number(event.target.value) || 0) * 1000,
                    )
                  }
                />
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => takePreviewPosition(changeStart)}
                >
                  <Play className="size-3.5" />
                  Use where it is now
                </Button>
              </div>
            </div>

            <div className="space-y-1.5">
              <Label htmlFor="clip-end">Stops at — {endLabel}</Label>
              <input
                id="clip-end"
                type="range"
                min={0}
                max={durationMs || 0}
                step={100}
                value={endOfRange}
                disabled={durationMs === 0}
                onChange={(event) => {
                  const next = Number(event.target.value);
                  // Sliding right to the very end means "let it run to the end",
                  // which is the same as no end at all.
                  setEndMs(
                    durationMs > 0 && next >= durationMs ? undefined : next,
                  );
                }}
                className="h-9 w-full cursor-pointer accent-[var(--brand)]"
              />
              <div className="flex items-center gap-2">
                <Input
                  type="number"
                  min={0}
                  aria-label="End in seconds"
                  placeholder="end of the video"
                  value={endMs === undefined ? "" : Math.round(endMs / 1000)}
                  onChange={(event) => {
                    const raw = event.target.value;
                    if (raw.trim() === "") {
                      setEndMs(undefined);
                      return;
                    }
                    setEndMs(Math.max(0, Number(raw) || 0) * 1000);
                  }}
                />
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => takePreviewPosition((ms) => setEndMs(ms))}
                >
                  <Play className="size-3.5" />
                  Use where it is now
                </Button>
              </div>
              <p className="text-xs text-muted-foreground">
                {wholeFile
                  ? "The whole video plays."
                  : `${formatClipTime(rangeLengthMs)} is shown${
                      repeat
                        ? ", and it starts again at the end."
                        : ", and it stops there."
                    }`}
              </p>
            </div>

            <div className="flex items-center justify-between rounded-lg border border-border/60 px-3 py-2">
              <div>
                <p className="text-sm font-medium">Start again at the end</p>
                <p className="text-xs text-muted-foreground">
                  Play the chosen part over and over instead of stopping at{" "}
                  {endLabel}.
                </p>
              </div>
              <Switch checked={repeat} onCheckedChange={setRepeat} />
            </div>
          </div>
        ) : null}

        <DialogFooter className="sm:justify-between">
          <div className="flex flex-wrap items-center gap-2">
            <Button
              variant="secondary"
              size="sm"
              disabled={!item || durationMs === 0}
              onClick={playSelection}
            >
              <Play className="size-3.5" />
              Play the chosen part
            </Button>
            <Button
              variant="ghost"
              size="sm"
              disabled={busy || !item}
              onClick={() => {
                setStartMs(0);
                setEndMs(undefined);
                // Forgetting the range is a change like any other, so it is
                // saved straight away rather than leaving the file half-trimmed.
                onSave(null);
              }}
            >
              Whole video
            </Button>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <Button
              variant="outline"
              disabled={busy || !item}
              onClick={() => onSave(draftClip())}
            >
              Save the range
            </Button>
            <Button
              variant="success"
              disabled={busy || !item}
              onClick={() => onShow(draftClip())}
            >
              <MonitorPlay className="size-4" />
              Show on screen
            </Button>
          </div>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
