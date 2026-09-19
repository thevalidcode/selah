/**
 * TypeScript mirrors of the Rust domain types.
 *
 * Rust serializes with `#[serde(rename_all = "camelCase")]`, so the shapes
 * below map 1:1 to `src-tauri/src/models/*` and
 * `src-tauri/src/scripture/reference.rs`. Keep them in sync when the Rust
 * types change.
 */

// ------------------------------------------------------------------- content

export type ContentType =
  | "scripture"
  | "lyrics"
  | "text"
  | "image"
  | "video"
  | "announcement"
  | "slide"
  | "song";

/** A structurally valid Scripture reference (never stored as a string). */
export interface ScriptureReference {
  bookId: number;
  chapter: number;
  startVerse?: number;
  endVerse?: number;
}

export interface DetectedContent {
  type: "scripture" | "text" | "unknown";
  reference?: ScriptureReference;
  text?: string;
}

export interface DetectionResult {
  content: DetectedContent;
  /** 0.0 – 1.0; absent when the detection is not scored. */
  confidence?: number;
}

// ------------------------------------------------------------------- speech

export interface TranscriptSegment {
  text: string;
  startMs: number;
  endMs: number;
  confidence?: number;
}

export interface Transcript {
  text: string;
  segments: TranscriptSegment[];
  /** "mock" transcripts are development placeholders, never real speech. */
  source: "mock" | "moonshine";
  sampleCount: number;
}

export interface SpeechManagerState {
  listening: boolean;
  recognizerId: string;
  modelLoaded: boolean;
  vadEnabled: boolean;
  segmentsSeen: number;
  transcriptsGenerated: number;
  lastError?: string;
}

// -------------------------------------------------------------------- audio

export interface AudioDeviceInfo {
  id: string;
  name: string;
  isDefault: boolean;
  defaultSampleRate: number;
  channels: number;
}

export interface AudioCaptureState {
  capturing: boolean;
  deviceId?: string;
  sampleRate?: number;
  channels?: number;
  bufferedSamples: number;
}

// -------------------------------------------------------------------- bible

export interface Translation {
  id: string;
  name: string;
  language: string;
  abbreviation?: string;
  isDefault: boolean;
  createdAt: string;
}

export interface TranslationStatus {
  translation: Translation;
  verseCount: number;
}

export interface BibleBook {
  id: number;
  name: string;
  testament: "old" | "new" | string;
  abbreviation?: string;
  /** Canonical chapter count, resolved from the Rust Scripture registry. */
  chapters: number;
}

export interface Verse {
  translationId: string;
  bookId: number;
  chapter: number;
  verse: number;
  text: string;
}

export interface Passage {
  translationId: string;
  reference: string;
  verses: Verse[];
  text: string;
}

// ------------------------------------------------------------- presentation

export type MediaKind = "image" | "video" | "audio";

export type ContentPayload =
  | {
      kind: "scripture";
      reference: string;
      translation: string;
      text: string;
      /** Operator-supplied heading; wins over the reference on screen. */
      heading?: string;
    }
  | { kind: "text"; heading?: string; text: string }
  | {
      kind: "media";
      path: string;
      /** Absent for files registered by an earlier build. */
      mediaKind?: MediaKind;
      /** Optional caption drawn over the media. */
      heading?: string;
    }
  | {
      kind: "song";
      title: string;
      label?: string;
      text: string;
      /** 1-based position of this section within the song. */
      index: number;
      total: number;
    };

export interface PresentationItem {
  id: string;
  contentType: ContentType;
  title: string;
  payload: ContentPayload;
}

export interface PresentationState {
  current?: PresentationItem;
  queue: PresentationItem[];
  history: PresentationItem[];
}

/** A persisted presentation row (with its ordered item records). */
export interface Presentation {
  id: string;
  name: string;
  createdAt: string;
  updatedAt: string;
  items?: PresentationItemRecord[];
}

export interface PresentationItemRecord {
  id: string;
  presentationId: string;
  typeName: string;
  position: number;
  payload: string;
}

export interface DisplayInfo {
  index: number;
  name?: string;
  size: [number, number];
  position: [number, number];
  scaleFactor: number;
  isPrimary: boolean;
}

// -------------------------------------------------------------------- media

export interface MediaItem {
  id: string;
  kind: "image" | "video" | "audio" | string;
  name: string;
  path: string;
  metadata?: Record<string, unknown>;
  createdAt: string;
}

/** One row in the media folder picker. */
export interface DirectoryEntry {
  name: string;
  path: string;
  isDir: boolean;
  /** `image` / `video` / `audio` for files Selah can present. */
  mediaKind?: MediaKind;
}

/** The contents of one folder, as listed by the media folder picker. */
export interface DirectoryListing {
  path: string;
  parent?: string;
  entries: DirectoryEntry[];
}

/** The result of loading a folder into the media library. */
export interface MediaScan {
  directory: string;
  /** Files registered for the first time by this scan. */
  added: number;
  /** Media files now known inside the folder. */
  total: number;
  items: MediaItem[];
}

// -------------------------------------------------------------------- songs

/** One section of a song (verse, chorus, bridge…). */
export interface SongSection {
  id: string;
  label?: string;
  text: string;
  /** 1-based order within the song. */
  position: number;
}

export interface Song {
  id: string;
  title: string;
  author?: string;
  createdAt: string;
  updatedAt: string;
  /** Absent in the library list; present when one song is loaded. */
  sections?: SongSection[];
}

/** A section as sent to the backend (the store assigns ids). */
export interface SongSectionInput {
  label?: string;
  text: string;
}

export interface SongInput {
  title: string;
  author?: string;
  sections: SongSectionInput[];
}

// ----------------------------------------------------------------- settings

export interface AppSettings {
  general: {
    appName: string;
    defaultTranslationId?: string;
    theme: string;
  };
  audio: {
    inputDeviceId?: string;
    sampleRate: number;
  };
  speech: {
    recognizer: "mock" | "moonshine";
    modelPath?: string;
    language?: string;
    threads: number;
    vadEnabled: boolean;
    speechSampleRate: number;
  };
  presentation: {
    displayIndex?: number;
    fullscreen: boolean;
    background: string;
    fontSize: number;
    /** Projected typeface; one of the bundled families (see `lib/fonts`). */
    fontFamily: string;
    followLive: boolean;
  };
  /** Where the media library reads files from (chosen at runtime). */
  media: {
    directory?: string;
    recursive: boolean;
  };
}

export interface SetupState {
  completed: boolean;
  hasTranslations: boolean;
  hasModel: boolean;
}

// ------------------------------------------------------------------- events

export interface VadSegmentEvent {
  startMs: number;
  endMs: number;
  durationMs: number;
}

export interface DetectionEvent {
  source: string;
  results: DetectionResult[];
}

export interface PresentationChangedEvent {
  item?: PresentationItem;
  projected: boolean;
}

export interface PresentationDisplayEvent {
  open: boolean;
  display?: string;
}

/** Payload of `presentation://settings` — how projected content should look. */
export interface PresentationSettingsEvent {
  background: string;
  fontSize: number;
  fontFamily: string;
  fullscreen: boolean;
  followLive: boolean;
}

/** Serialized error payload returned by failing commands. */
export interface ErrorPayload {
  kind: string;
  message: string;
}
