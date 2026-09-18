import { command } from "./client";
import type { AppSettings, SetupState } from "../../types";

export function getSettings(): Promise<AppSettings> {
  return command<AppSettings>("get_settings");
}

export function updateSettings(settings: AppSettings): Promise<AppSettings> {
  return command<AppSettings>("update_settings", { request: { settings } });
}

export function getSetupState(): Promise<SetupState> {
  return command<SetupState>("get_setup_state");
}

export function completeSetup(): Promise<void> {
  return command<void>("complete_setup");
}