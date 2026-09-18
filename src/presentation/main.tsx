/**
 * Presentation window entry point.
 *
 * This webview is what the congregation sees. It subscribes to the
 * `presentation://changed` Tauri event emitted by the Rust presentation engine
 * and renders nothing but content — no operator chrome, no controls.
 */

import { useEffect, useState } from "react";
import { createRoot } from "react-dom/client";

import { EVENTS, subscribeToPresentation } from "@/lib/events";
import { listen } from "@tauri-apps/api/event";
import type { PresentationItem } from "@/types";

import "../index.css";

/** Renders the currently projected item, or the idle slate. */
function PresentationScreen() {
  const [item, setItem] = useState<PresentationItem | null>(null);
  const [connected, setConnected] = useState(false);

  useEffect(() => {
    let dispose: (() => void) | undefined;
    let displayUnlisten: (() => void) | undefined;
    let disposed = false;

    subscribeToPresentation((next) => {
      setItem(next);
      setConnected(true);
    }).then((unlisten) => {
      if (disposed) {
        unlisten();
        return;
      }
      dispose = unlisten;
      setConnected(true);
    });

    listen(EVENTS.presentationDisplayOpened, () => setConnected(true)).then(
      (unlisten) => {
        if (disposed) {
          unlisten();
          return;
        }
        displayUnlisten = unlisten;
      },
    );

    return () => {
      disposed = true;
      dispose?.();
      displayUnlisten?.();
    };
  }, []);

  if (!item) {
    return (
      <div className="pres-shell pres-shell--idle">
        <div className="pres-idle">
          <div className="pres-idle__mark">SELAH</div>
          <p className="pres-idle__hint">
            {connected
              ? "Ready — nothing is being displayed"
              : "Waiting for the operator application…"}
          </p>
        </div>
      </div>
    );
  }

  const payload = item.payload;

  if (payload.kind === "media") {
    const isVideo = /\.(mp4|mov|m4v|mkv|webm)$/i.test(payload.path);
    return (
      <div className="pres-shell pres-shell--media">
        {isVideo ? (
          <video className="pres-media" src={toFileUrl(payload.path)} autoPlay />
        ) : (
          <img
            className="pres-media"
            src={toFileUrl(payload.path)}
            alt={item.title}
          />
        )}
      </div>
    );
  }

  if (payload.kind === "scripture") {
    return (
      <div className="pres-shell">
        <article className="pres-scripture">
          <header className="pres-scripture__head">
            <h1 className="pres-scripture__reference">{payload.reference}</h1>
            <span className="pres-scripture__translation">
              {payload.translation}
            </span>
          </header>
          <p className="pres-scripture__text">{payload.text}</p>
        </article>
      </div>
    );
  }

  return (
    <div className="pres-shell">
      <article className="pres-text">
        <h1 className="pres-text__title">{item.title}</h1>
        <p className="pres-text__body">{payload.text}</p>
      </article>
    </div>
  );
}

/** Converts a local filesystem path to a URL the webview can load. */
function toFileUrl(path: string): string {
  const normalized = path.replace(/\\/g, "/");
  return normalized.startsWith("/")
    ? `file://${normalized}`
    : `file:///${normalized}`;
}

const container = document.getElementById("presentation-root");
if (!container) {
  throw new Error("#presentation-root missing from presentation.html");
}

createRoot(container).render(<PresentationScreen />);
