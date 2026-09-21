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
  /**
   * True for the Bibles that come with Selah (WEB, KJV, ASV). They cannot be
   * renamed or removed.
   */
  builtin: boolean;
  /** `bundled`, `catalogue` or `operator` — where the translation came from. */
  origin: string;
}

export interface TranslationStatus {
  translation: Translation;
  verseCount: number;
}

/**
 * One published translation Selah knows how to hold.
 *
 * Metadata only — nothing is downloaded, and the words still come from a file
 * the operator owns or from verses they supply.
 */
export interface CatalogueEntry {
  id: string;
  name: string;
  abbreviation: string;
  language: string;
  /** `Classic`, `Modern`, `Everyday`, or `Added by you`. */
  group: string;
  /** One of Selah's own three, which cannot be removed. */
  builtin: boolean;
  installed: boolean;
  verseCount: number;
  /** Whether the translation text itself is public domain. */
  publicDomain: boolean;
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

/**
 * A theme to offer when a search finds nothing.
 *
 * Selah ships no Bible text, so a suggestion is only a reference (plus the
 * theme it answers) — the words still come from the installed translation.
 */
export interface TopicSuggestion {
  topic: string;
  why: string;
  bookId: number;
  chapter: number;
  verse: number;
  /** Human-readable reference, e.g. `Philippians 4:6`. */
  reference: string;
}

/** What a search found, and what to try when it found nothing. */
export interface BibleSearchResult {
  verses: Verse[];
  /** True when the exact words did not match and these are the closest. */
  related: boolean;
  /** Themes to offer when no verse matched at all. */
  suggestions: TopicSuggestion[];
}

/** One verse typed or pasted into the Custom translation tab. */
export interface CustomVerseInput {
  /** Book name, abbreviation or number, e.g. `John`, `Ps`, `43`. */
  book: string;
  chapter: number;
  verse: number;
  text: string;
}

export interface SaveCustomVersesRequest {
  /** Short id the verses are filed under, e.g. `my-notes`. */
  translationId?: string;
  /** Name shown in the Bible picker. */
  name?: string;
  abbreviation?: string;
  verses: CustomVerseInput[];
}

/** What was saved, and anything the operator should know about it. */
export interface CustomVerseImportResult {
  translationId: string;
  reference: string;
  bookId: number;
  chapter: number;
  versesSaved: number;
  /** Verse numbers inside the saved range that were not supplied. */
  missingVerses: number[];
}

// ------------------------------------------------------------- presentation

export type MediaKind = "image" | "video" | "audio";

/** What the operator asked the projector's player to do. */
export type MediaPlaybackAction = "play" | "pause" | "restart" | "stop";

/**
 * What the projector window says it is playing.
 *
 * A video keeps moving after it is on the screen, and only the projector knows
 * whether it really started, so the operator screen is told rather than
 * guessing.
 */
export interface MediaPlaybackState {
  /** The item on screen, when there is one. */
  itemId?: string;
  /** `image`, `video` or `audio`, when the item is media. */
  mediaKind?: MediaKind;
  playing: boolean;
  positionMs: number;
  durationMs: number;
  /** A video that has run to its end. */
  ended: boolean;
}

export type ContentPayload =
  | {
      kind: "scripture";
      reference: string;
      translation: string;
      text: string;
      /** Operator-supplied heading; wins over the reference on screen. */
      heading?: string;
      /**
       * Exact heading size in CSS pixels, chosen on the Bible screen. Absent
       * means "use the saved projector size".
       */
      headingSize?: number;
      /** Exact verse size in CSS pixels. Absent uses the saved projector size. */
      textSize?: number;
    }
  | { kind: "text"; heading?: string; text: string }
  | {
      kind: "media";
      path: string;
      /** Absent for files registered by an earlier build. */
      mediaKind?: MediaKind;
      /** Optional caption drawn over the media. */
      heading?: string;
      /** Where playback begins, in milliseconds. Absent means the beginning. */
      startMs?: number;
      /** Where playback ends, in milliseconds. Absent runs to the end. */
      endMs?: number;
      /**
       * Whether a video starts again at the end (or at the end of the chosen
       * range) instead of stopping. Absent for pictures and sound; the projector
       * then follows the saved Screen setting.
       */
      repeat?: boolean;
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
  /**
   * Folders inside the chosen folder that could not be read. Reported so files
   * never appear to be missing without explanation.
   */
  skipped: number;
  /**
   * True when Selah stopped before the end of the folder — the file limit was
   * reached, or the folder tree was deeper than Selah walks.
   */
  truncated: boolean;
  items: MediaItem[];
}

/**
 * How a video should be played: where to begin, where to stop, and whether it
 * starts again instead of stopping.
 *
 * A clip with no range (`startMs: 0`, no `endMs`) is still meaningful: it is the
 * "stop or repeat" choice for the whole file, which is what the Screen setting
 * holds by default.
 */
export interface MediaClip {
  /** Where playback begins, in milliseconds from the start of the file. */
  startMs: number;
  /** Where playback ends, in milliseconds. Absent runs to the end. */
  endMs?: number;
  /** Whether playback starts again at `startMs` instead of stopping. */
  repeat: boolean;
}

/** The clip as sent to the backend; `repeat` may be left to the saved setting. */
export interface MediaClipRequest {
  startMs?: number;
  endMs?: number;
  repeat?: boolean;
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

// ---------------------------------------------------------------- settings

/**
 * Where the branding overlay sits on the projector.
 *
 * The overlay hugs one edge and is centred along it, so a logo always looks
 * deliberate rather than floating.
 */
export type BrandingPosition = "top" | "bottom" | "left" | "right";

/**
 * The operator's branding: a line of text and/or a logo image, drawn over every
 * projected item so the screen looks like their church's.
 *
 * Both parts are optional — an empty text and no logo simply means no branding,
 * so there is no separate on/off switch to fall out of sync.
 */
export interface BrandingSettings {
  position: BrandingPosition;
  /** Church name, service title, slogan… */
  text?: string;
  /** Absolute path to a logo image; drawn through the `asset:` protocol. */
  logo?: string;
  /** Overlay size as a percentage of the projected text size. */
  sizePercent: number;
}

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
    /** The operator's own logo and line of text, drawn on every item. */
    branding: BrandingSettings;
    /**
     * Whether a video starts again when it reaches the end (or the end of the
     * time range chosen for it on the Media screen). The per-file choice there
     * starts from this.
     */
    repeatVideos: boolean;
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

/** Payload of `media://playback` — what the operator asked the player to do. */
export interface MediaPlaybackCommand {
  action: MediaPlaybackAction;
  /** The item the action is for, so a stale command cannot affect a new file. */
  itemId?: string;
}

/** Payload of `presentation://settings` — how projected content should look. */
export interface PresentationSettingsEvent {
  background: string;
  fontSize: number;
  fontFamily: string;
  fullscreen: boolean;
  followLive: boolean;
  /** The operator's logo and line of text, drawn on every projected item. */
  branding: BrandingSettings;
  /**
   * Whether a video starts again when it reaches the end. Carried to the
   * projector so a Save changes a video already on the screen.
   */
  repeatVideos: boolean;
}

/** Serialized error payload returned by failing commands. */
export interface ErrorPayload {
  kind: string;
  message: string;
}
