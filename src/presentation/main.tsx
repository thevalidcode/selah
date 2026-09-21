/**
 * Presentation window entry point.
 *
 * This webview is what the congregation sees. It subscribes to the
 * `presentation://changed` Tauri event emitted by the Rust presentation engine
 * and renders nothing but content — no operator chrome, no controls.
 *
 * It also listens to `presentation://settings`, which is how a Save in
 * Settings changes the background colour, text size and typeface of what is on
 * screen. Media files are loaded through Tauri's `asset:` protocol, because a
 * webview cannot read a plain filesystem path.
 */

import { useCallback, useEffect, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { listen } from "@tauri-apps/api/event";

import { SelahIcon } from "@/components/ui/selah-icon";
import { mediaApi, settingsApi } from "@/lib/api";
import { EVENTS, subscribeToPresentation } from "@/lib/events";
import { isLightColor, resolveFontFamily } from "@/lib/fonts";
import { clipEndAction, mediaKindFromPath, mediaUrl } from "@/lib/media";
import type {
  AppSettings,
  BrandingSettings,
  ContentPayload,
  MediaClip,
  MediaPlaybackCommand,
  PresentationItem,
} from "@/types";

import "../index.css";

/**
 * The parts of the saved settings that change how projected content looks.
 */
type ProjectorSettings = Pick<
  AppSettings["presentation"],
  "background" | "fontSize" | "fontFamily" | "branding" | "repeatVideos"
>;

const FALLBACK_SETTINGS: ProjectorSettings = {
  background: "#000000",
  fontSize: 64,
  fontFamily: "Creato Display",
  branding: { position: "bottom", sizePercent: 30 },
  repeatVideos: true,
};

/**
 * Writes the projector settings onto the document as CSS variables.
 *
 * The text colour is derived from the background: a pale background gets dark
 * words, so the "Background colour" setting stays usable whatever is chosen.
 */
function applyProjectorSettings(settings: ProjectorSettings) {
  const light = isLightColor(settings.background);
  const root = document.documentElement;
  const branding = settings.branding ?? FALLBACK_SETTINGS.branding;

  root.style.setProperty("--pres-bg", settings.background);
  root.style.setProperty("--pres-fg", light ? "#141B2E" : "#FFFFFF");
  root.style.setProperty(
    "--pres-fg-muted",
    light ? "rgba(20, 27, 46, 0.55)" : "rgba(255, 255, 255, 0.5)",
  );
  root.style.setProperty(
    "--pres-font",
    `"${resolveFontFamily(settings.fontFamily)}"`,
  );
  root.style.setProperty("--pres-size", `${settings.fontSize}px`);

  // Branding is sized relative to the projected words, so a church with a big
  // screen and a church with a small one both get a sensible overlay.
  const percent = branding.sizePercent || FALLBACK_SETTINGS.branding.sizePercent;
  root.style.setProperty(
    "--pres-brand-size",
    `calc(var(--pres-size, 64px) * ${percent / 100})`,
  );

  // The accent used by announcements. It has to be legible on both a dark and a
  // pale background, so it darkens when the background is light.
  root.style.setProperty("--pres-accent", light ? "#8A6A16" : "#E0B64A");

  // Paint the webview itself too, so nothing flashes between items.
  document.body.style.background = settings.background;
}


/** Renders the currently projected item, or the idle slate. */
function PresentationScreen() {
  const [item, setItem] = useState<PresentationItem | null>(null);
  const [connected, setConnected] = useState(false);
  const [branding, setBranding] = useState<BrandingSettings>(
    FALLBACK_SETTINGS.branding,
  );
  // The saved answer to "when a video reaches the end: stop or start again".
  // It is the default for every video, and a per-file choice on the Media
  // screen overrides it for that file.
  const [repeatVideos, setRepeatVideos] = useState(
    FALLBACK_SETTINGS.repeatVideos,
  );

  const applySettings = useCallback((settings: ProjectorSettings) => {
    applyProjectorSettings(settings);
    setBranding(settings.branding ?? FALLBACK_SETTINGS.branding);
    setRepeatVideos(settings.repeatVideos ?? FALLBACK_SETTINGS.repeatVideos);
  }, []);

  useEffect(() => {
    const unlisteners: Array<() => void> = [];
    let disposed = false;

    /** Keeps a subscription, or closes it straight away if we already left. */
    const track = (unlisten: () => void) => {
      if (disposed) {
        unlisten();
        return;
      }
      unlisteners.push(unlisten);
    };

    subscribeToPresentation((next) => {
      setItem(next);
      setConnected(true);
    }).then(track);

    listen(EVENTS.presentationDisplayOpened, () => setConnected(true)).then(track);

    // How projected content should look. The event covers changes made while
    // Selah is running; the initial read covers a window opened later.
    listen<ProjectorSettings>(EVENTS.presentationSettings, (event) => {
      applySettings(event.payload);
    }).then(track);

    settingsApi
      .getSettings()
      .then((settings) => applySettings(settings.presentation))
      .catch(() => applySettings(FALLBACK_SETTINGS));

    return () => {
      disposed = true;
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, [applySettings]);

  if (!item) {
    return (
      <div className="pres-stage">
        <div className="pres-shell pres-shell--idle">
          <div className="pres-idle">
            {/*
              Gold mark on black, no tile — the tile is navy and would vanish
              against the projector background.
            */}
            <SelahIcon size={120} title="Selah" />
            <div className="pres-idle__mark">SELAH</div>
            <p className="pres-idle__hint">
              {connected
                ? "Ready — nothing is on screen right now"
                : "Waiting for the operator screen…"}
            </p>
          </div>
        </div>
        <BrandingOverlay branding={branding} />
      </div>
    );
  }

  return (
    <div className="pres-stage">
      <ProjectedItem item={item} repeatVideos={repeatVideos} />
      <BrandingOverlay branding={branding} />
    </div>
  );
}

/**
 * The operator's logo and/or line of text, drawn over whatever is on screen.
 *
 * It is a sibling of the content rather than part of it, so branding appears
 * over verses, words, songs and media alike — and never pushes content around,
 * which would break a carefully laid-out verse.
 */
function BrandingOverlay({ branding }: { branding: BrandingSettings }) {
  const text = branding.text?.trim();
  const logo = branding.logo?.trim();
  if (!text && !logo) {
    return null;
  }

  return (
    <div
      className={`pres-brand pres-brand--${branding.position}`}
      aria-hidden="true"
    >
      {logo ? (
        <img className="pres-brand__logo" src={mediaUrl(logo)} alt="" />
      ) : null}
      {text ? <span className="pres-brand__text">{text}</span> : null}
    </div>
  );
}

function ProjectedItem({
  item,
  repeatVideos,
}: {
  item: PresentationItem;
  /** Saved "stop or repeat" answer, used when the item carries no choice. */
  repeatVideos: boolean;
}) {
  const payload = item.payload;

  switch (payload.kind) {
    case "media":
      return (
        <ProjectedMedia
          payload={payload}
          title={item.title}
          itemId={item.id}
          repeatVideos={repeatVideos}
        />
      );
    case "song":
      return (
        <ProjectedText
          heading={payload.title}
          label={payload.label}
          text={payload.text}
          counter={`${payload.index} / ${payload.total}`}
        />
      );
    case "scripture":
      return (
        <ProjectedText
          // A heading typed by the operator wins; otherwise the reference.
          heading={payload.heading ?? payload.reference}
          label={payload.translation}
          text={payload.text}
          headingSize={payload.headingSize}
          textSize={payload.textSize}
        />
      );
    case "text":
      return (
        <ProjectedText
          heading={payload.heading ?? item.title}
          text={payload.text}
          // Notices get their own look so the congregation can tell an
          // announcement from Scripture at a glance.
          variant={item.contentType === "announcement" ? "notice" : "words"}
        />
      );
  }
}

/** Heading + optional label + words, shared by text, Scripture and songs. */
function ProjectedText({
  heading,
  label,
  text,
  counter,
  headingSize,
  textSize,
  variant = "words",
}: {
  heading?: string;
  label?: string;
  text: string;
  counter?: string;
  /** Exact sizes in CSS pixels, chosen for this one item. */
  headingSize?: number;
  textSize?: number;
  variant?: "words" | "notice";
}) {
  return (
    <div className={`pres-shell pres-shell--${variant}`}>
      <article className={`pres-scripture pres-scripture--${variant}`}>
        <header className="pres-content__head">
          {heading ? (
            <h1
              className="pres-content__heading"
              style={headingSize ? { fontSize: `${headingSize}px` } : undefined}
            >
              {heading}
            </h1>
          ) : null}
          {label ? <span className="pres-content__label">{label}</span> : null}
        </header>
        <p
          className="pres-content__text"
          style={textSize ? { fontSize: `${textSize}px` } : undefined}
        >
          {text}
        </p>
        {counter ? (
          <span className="pres-content__counter">{counter}</span>
        ) : null}
      </article>
    </div>
  );
}

/**
 * An image, video or audio file, taking over the whole screen.
 *
 * The file is fitted **inside** the window (`object-contain`, never cropped) and
 * centred in the space it does not fill, which is painted the projector's own
 * background colour — so a portrait photo on a wide screen never spills past the
 * edges, and nothing is ever cut off.
 *
 * A video can be a *part* of a file: the Media screen lets the operator choose
 * where it starts and where it stops, and whether it repeats, so a long clip can
 * be trimmed to the seconds that matter. Music gets a card instead of a blank
 * screen and plays once.
 *
 * Video and sound also answer the operator's play/pause/restart/stop buttons
 * and report back what they are actually doing: the operator screen cannot see
 * this one, so it is told rather than left guessing.
 */
function ProjectedMedia({
  payload,
  title,
  itemId,
  repeatVideos,
}: {
  payload: Extract<ContentPayload, { kind: "media" }>;
  title: string;
  itemId: string;
  /** Saved "stop or repeat" answer; a per-file choice in the item wins. */
  repeatVideos: boolean;
}) {
  const source = mediaUrl(payload.path);
  const kind = payload.mediaKind ?? mediaKindFromPath(payload.path);
  const heading = payload.heading ?? title;
  const player = useRef<HTMLVideoElement | HTMLAudioElement | null>(null);

  // The part of the video to play. A picture and a piece of music never carry a
  // range, and only video repeats — sound has always played once and stopped.
  const clip: MediaClip | undefined =
    kind === "video"
      ? {
          startMs: Math.max(0, payload.startMs ?? 0),
          endMs: payload.endMs,
          repeat: payload.repeat ?? repeatVideos,
        }
      : undefined;
  const startSeconds = (clip?.startMs ?? 0) / 1000;
  const endMs = clip?.endMs;

  /**
   * Puts the player on the first frame of the chosen part.
   *
   * A seek made before the file knows its own length is ignored by the player,
   * which would silently start the video from the very beginning.
   */
  function seekToStart(element: HTMLVideoElement | HTMLAudioElement) {
    if (startSeconds > 0 && Number.isFinite(element.duration)) {
      element.currentTime = Math.min(startSeconds, element.duration);
    }
  }

  /** Where a video should stop, whether the range says so or the file ends. */
  function rangeEnd(element: HTMLVideoElement | HTMLAudioElement): number | undefined {
    if (endMs !== undefined) {
      return endMs;
    }
    return Number.isFinite(element.duration) ? element.duration * 1000 : undefined;
  }

  /**
   * Holds the player inside the chosen range.
   *
   * Run on every `timeupdate`: a range that should repeat goes back to its
   * start, and one that should stop is paused on its last frame, so the
   * congregation sees the end of what the operator chose instead of the file
   * carrying on.
   */
  function holdWithinRange(element: HTMLVideoElement | HTMLAudioElement) {
    if (!clip) {
      return;
    }
    const action = clipEndAction(element.currentTime * 1000, clip, rangeEnd(element) ?? 0);
    if (action === "restart") {
      element.currentTime = startSeconds;
      void element.play().catch(() => undefined);
      return;
    }
    if (action === "end") {
      element.pause();
      if (endMs !== undefined) {
        // Land exactly on the chosen end so the last frame stays visible.
        element.currentTime = endMs / 1000;
      }
      reportElement(element, true);
    }
  }

  // Commands from the operator screen, and reports back to it.
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let disposed = false;

    if (kind === "image") {
      // A picture has nothing to play; say so once so the Media screen shows
      // "showing" instead of "paused".
      void mediaApi
        .reportMediaPlayback({ playing: false, ended: false })
        .catch(() => undefined);
      return undefined;
    }

    const report = () => {
      if (player.current) {
        reportElement(player.current);
      }
    };

    listen<MediaPlaybackCommand>(EVENTS.mediaPlayback, (event) => {
      // A command for a different item is stale: it was aimed at whatever was
      // on the screen when the button was pressed.
      if (event.payload.itemId && event.payload.itemId !== itemId) {
        return;
      }
      const element = player.current;
      if (!element) {
        return;
      }

      switch (event.payload.action) {
        case "play": {
          // Held at the end of the range: playing again starts the range over
          // rather than instantly finishing again.
          const end =
            endMs ??
            (Number.isFinite(element.duration)
              ? element.duration * 1000
              : undefined);
          if (end !== undefined && element.currentTime * 1000 >= end - 50) {
            element.currentTime = startSeconds;
          }
          void element.play().catch(() => undefined);
          break;
        }
        case "pause":
          element.pause();
          break;
        case "restart":
          // "Start again" means the beginning of what is being shown, which is
          // the chosen start for a trimmed video — not the start of the file the
          // operator deliberately skipped.
          element.currentTime = startSeconds;
          void element.play().catch(() => undefined);
          break;
        case "stop":
          element.pause();
          element.currentTime = startSeconds;
          break;
      }
      // Report straight away so the button feels connected, then again when the
      // element has caught up.
      report();
      window.setTimeout(report, 250);
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
    // `startSeconds`/`endMs` change with the projected item, which is also what
    // re-subscribes the command listener, so they belong in the deps.
  }, [itemId, kind, startSeconds, endMs]);

  /** Reports on the events the player itself raises. */
  const elementRef = useCallback(
    (element: HTMLVideoElement | HTMLAudioElement | null) => {
      player.current = element;
    },
    [],
  );

  return (
    <div className="pres-shell pres-shell--media">
      <div className="pres-media__frame">
        {kind === "video" ? (
          <video
            ref={elementRef}
            className="pres-media"
            src={source}
            autoPlay
            playsInline
            /*
              `loop` is deliberately not used. A repeating video is sent back to
              the start of the *chosen range*, not to the start of the file, and
              a video that should stop has to be able to reach its end.
            */
            onLoadedMetadata={(event) => {
              seekToStart(event.currentTarget);
              reportElement(event.currentTarget);
            }}
            onPlay={(event) => reportElement(event.currentTarget)}
            onPause={(event) => reportElement(event.currentTarget)}
            onEnded={(event) => {
              const element = event.currentTarget;
              if (clip?.repeat) {
                element.currentTime = startSeconds;
                void element.play().catch(() => undefined);
                return;
              }
              reportElement(element, true);
            }}
            onTimeUpdate={(event) => {
              holdWithinRange(event.currentTarget);
              reportElement(event.currentTarget, false, true);
            }}
          />
        ) : kind === "audio" ? (
          <div className="pres-audio">
            <span className="pres-audio__icon">♪</span>
            {heading ? (
              <h1 className="pres-content__heading">{heading}</h1>
            ) : null}
            <audio
              ref={elementRef}
              src={source}
              autoPlay
              onPlay={(event) => reportElement(event.currentTarget)}
              onPause={(event) => reportElement(event.currentTarget)}
              onEnded={(event) => reportElement(event.currentTarget, true)}
              onLoadedMetadata={(event) => reportElement(event.currentTarget)}
            />
          </div>
        ) : (
          <img className="pres-media" src={source} alt={heading} />
        )}

        {kind !== "audio" && heading ? (
          <div className="pres-media__caption">
            <h1 className="pres-content__heading">{heading}</h1>
          </div>
        ) : null}
      </div>
    </div>
  );
}

/** Last time a progress report was sent, so `timeupdate` cannot flood the IPC. */
let lastProgressReport = 0;

/**
 * Tells the operator screen what a media element is doing.
 *
 * `playing` is read from the element rather than assumed: a browser can refuse
 * autoplay, and a video that never started must not show as "playing".
 */
function reportElement(
  element: HTMLVideoElement | HTMLAudioElement,
  ended = false,
  progressOnly = false,
) {
  if (progressOnly) {
    const now = Date.now();
    if (now - lastProgressReport < 1000) {
      return;
    }
    lastProgressReport = now;
  }

  void mediaApi
    .reportMediaPlayback({
      playing: !element.paused && !element.ended,
      positionMs: element.currentTime * 1000,
      durationMs: Number.isFinite(element.duration) ? element.duration * 1000 : 0,
      ended: ended || element.ended,
    })
    .catch(() => undefined);
}

const container = document.getElementById("presentation-root");
if (!container) {
  throw new Error("#presentation-root missing from presentation.html");
}

createRoot(container).render(<PresentationScreen />);
