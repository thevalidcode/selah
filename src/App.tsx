import { HashRouter, Navigate, Route, Routes } from "react-router-dom";
import { useEffect } from "react";

import Sidebar from "@/components/Sidebar";
import { SelahIcon } from "@/components/ui/selah-icon";
import { useSettings } from "@/hooks/useSettings";
import { useSetupGate } from "@/hooks/useSetupGate";
import BiblePage from "@/pages/BiblePage";
import HomePage from "@/pages/HomePage";
import LivePage from "@/pages/LivePage";
import MediaPage from "@/pages/MediaPage";
import PresentationsPage from "@/pages/PresentationsPage";
import SettingsPage from "@/pages/SettingsPage";
import SetupPage from "@/pages/SetupPage";

/**
 * Operator application shell: sidebar navigation + routed pages.
 *
 * The presentation screen is a separate webview (`presentation.html`) — it
 * never shares this routing or these components.
 */
export default function App() {
  const { checking, needsSetup, refresh } = useSetupGate();
  const { settings } = useSettings();

  // Apply the persisted theme once settings arrive. Selah is dark-first, so
  // anything other than an explicit "light" stays on the dark palette.
  useEffect(() => {
    const theme = settings?.general.theme;
    if (!theme) {
      return;
    }
    const light = theme === "light";
    document.documentElement.classList.toggle("light", light);
    document.documentElement.classList.toggle("dark", !light);
  }, [settings?.general.theme]);

  if (checking) {
    return (
      <div className="grid h-full place-items-center bg-background">
        <div className="flex flex-col items-center gap-3">
          <SelahIcon size={56} tile title="Selah" className="rounded-2xl" />
          <div className="text-xl font-black tracking-[0.3em] text-foreground/90">
            SELAH
          </div>
          <p className="text-sm text-muted-foreground">Getting things ready…</p>
        </div>
      </div>
    );
  }

  return (
    <HashRouter>
      <div className="flex h-full overflow-hidden bg-background">
        {needsSetup ? (
          <Routes>
            <Route path="*" element={<SetupPage onDone={refresh} />} />
          </Routes>
        ) : (
          <>
            <Sidebar />
            <main className="flex-1 overflow-y-auto">
              <div className="mx-auto flex w-full max-w-6xl flex-col gap-3.5 p-4 lg:p-5">
                <Routes>
                  <Route path="/" element={<HomePage />} />
                  <Route path="/live" element={<LivePage />} />
                  <Route path="/bible" element={<BiblePage />} />
                  <Route path="/media" element={<MediaPage />} />
                  <Route path="/presentations" element={<PresentationsPage />} />
                  <Route path="/settings" element={<SettingsPage />} />
                  <Route path="*" element={<Navigate to="/" replace />} />
                </Routes>
              </div>
            </main>
          </>
        )}
      </div>
    </HashRouter>
  );
}
