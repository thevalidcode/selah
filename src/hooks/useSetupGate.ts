import { useCallback, useEffect, useState } from "react";
import { settingsApi } from "../lib/api";
import type { SetupState } from "../types";

/**
 * First-run gate: while setup is incomplete the UI routes to the Setup page.
 */
export function useSetupGate(): {
  checking: boolean;
  needsSetup: boolean;
  refresh: () => void;
} {
  const [state, setState] = useState<SetupState | null>(null);
  const [checking, setChecking] = useState(true);

  const refresh = useCallback(() => {
    settingsApi
      .getSetupState()
      .then(setState)
      .catch(() => setState(null))
      .finally(() => setChecking(false));
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return {
    checking,
    needsSetup: state ? !state.completed : false,
    refresh,
  };
}