import { useCallback, useEffect, useState } from "react";

import { settingsApi } from "@/lib/api";
import type { AppSettings } from "@/types";

/** Deep-partial patch accepted by {@link useSettings}'s `save`. */
export type SettingsPatch = {
  general?: Partial<AppSettings["general"]>;
  audio?: Partial<AppSettings["audio"]>;
  speech?: Partial<AppSettings["speech"]>;
  presentation?: Partial<AppSettings["presentation"]>;
};

/**
 * Loads persisted settings and exposes a shallow-merging save helper.
 *
 * Settings live in SQLite as one JSON document, so adding a field never needs
 * a migration — the Rust side fills in defaults for anything missing.
 */
export function useSettings(): {
  settings: AppSettings | null;
  loading: boolean;
  error: string | null;
  save: (patch: SettingsPatch) => Promise<void>;
  reload: () => void;
} {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(() => {
    setLoading(true);
    settingsApi
      .getSettings()
      .then((s) => {
        setSettings(s);
        setError(null);
      })
      .catch((e: unknown) => {
        setError(e instanceof Error ? e.message : String(e));
      })
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => {
    reload();
  }, [reload]);

  const save = useCallback(
    async (patch: SettingsPatch) => {
      const current =
        settings ??
        // First save before the initial load finished: fetch, then patch.
        (await settingsApi.getSettings());

      const next: AppSettings = {
        general: { ...current.general, ...patch.general },
        audio: { ...current.audio, ...patch.audio },
        speech: { ...current.speech, ...patch.speech },
        presentation: { ...current.presentation, ...patch.presentation },
      };

      // Optimistic update keeps controls responsive; the Rust side echoes the
      // authoritative document back.
      setSettings(next);
      try {
        const saved = await settingsApi.updateSettings(next);
        setSettings(saved);
        setError(null);
      } catch (e: unknown) {
        setError(e instanceof Error ? e.message : String(e));
        reload();
      }
    },
    [settings, reload],
  );

  return { settings, loading, error, save, reload };
}
