/**
 * Selah frontend API layer.
 *
 * React components never call `invoke` directly — every Tauri command is
 * wrapped here so command names and payload shapes live in exactly one place.
 */

export * from "./client";
export * as audioApi from "./audio";
export * as bibleApi from "./bible";
export * as mediaApi from "./media";
export * as presentationApi from "./presentation";
export * as scriptureApi from "./scripture";
export * as settingsApi from "./settings";
export * as speechApi from "./speech";
