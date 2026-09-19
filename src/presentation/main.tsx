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

import { useCallback, useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import { listen } from "@tauri-apps/api/event";

import { SelahIcon } from "@/components/ui/selah-icon";
import { settingsApi } from "@/lib/api";
import { EVENTS, subscribeToPresentation } from "@/lib/events";
import { isLightColor, resolveFontFamily } from "@/lib/fonts";
import { mediaKindFromPath, mediaUrl } from "@/lib/media";
import type { AppSettings, ContentPayload, PresentationItem } from "@/types";

import "../index.css";

/** The parts of the saved settings that change how projected content looks. */
type ProjectorSettings = Pick<
  AppSettings["presentation"],
  "background" | "fontSize" | "fontFamily"
>;

const FALLBACK_SETTINGS: ProjectorSettings = {
  background: "#000000",
  fontSize: 64,
  fontFamily: "Creato Display",
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

  // Paint the webview itself too, so nothing flashes between items.
  document.body.style.background = settings.background;
}


/** Renders the currently projected item, or the idle slate. */
function PresentationScreen() {
  const [item, setItem] = useState<PresentationItem | null>(null);
  const [connected, setConnected] = useState(false);

  const applySettings = useCallback((settings: ProjectorSettings) => {
    applyProjectorSettings(settings);
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
    );
  }

  return <ProjectedItem item={item} />;
}

function ProjectedItem({ item }: { item: PresentationItem }) {
  const payload = item.payload;

  switch (payload.kind) {
    case "media":
      return <ProjectedMedia payload={payload} title={item.title} />;
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
        />
      );
    case "text":
      return (
        <ProjectedText
          heading={payload.heading ?? item.title}
          text={payload.text}
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
}: {
  heading?: string;
  label?: string;
  text: string;
  counter?: string;
}) {
  return (
    <div className="pres-shell">
      <article className="pres-scripture">
        <header className="pres-content__head">
          {heading ? <h1 className="pres-content__heading">{heading}</h1> : null}
          {label ? <span className="pres-content__label">{label}</span> : null}
        </header>
        <p className="pres-content__text">{text}</p>
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
 * Pictures and video are drawn on the projector's background colour with
 * `object-contain`, so nothing is ever cropped; a video plays straight away and
 * loops until the operator moves on. Music gets a card instead of a black
 * screen, with the words still drawn on top.
 */
function ProjectedMedia({
  payload,
  title,
}: {
  payload: Extract<ContentPayload, { kind: "media" }>;
  title: string;
}) {
  const source = mediaUrl(payload.path);
  const kind = payload.mediaKind ?? mediaKindFromPath(payload.path);
  const heading = payload.heading ?? title;

  return (
    <div className="pres-shell pres-shell--media">
      <div className="relative grid h-full w-full place-items-center">
        {kind === "video" ? (
          <video className="pres-media" src={source} autoPlay loop playsInline />
        ) : kind === "audio" ? (
          <div className="pres-audio">
            <span className="pres-audio__icon">♪</span>
            {heading ? (
              <h1 className="pres-content__heading">{heading}</h1>
            ) : null}
            <audio src={source} autoPlay />
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

const container = document.getElementById("presentation-root");
if (!container) {
  throw new Error("#presentation-root missing from presentation.html");
}

createRoot(container).render(<PresentationScreen />);
