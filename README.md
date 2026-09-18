# Selah

**Offline-first church/service presentation application.**

Selah listens to the microphone, understands what is being said, and turns
detected content into presentation content on a second screen — entirely on the
local machine. No cloud backend, no accounts, no API keys, no external AI
service.

```text
Microphone
    ↓
CPAL (native input stream)
    ↓
Audio pipeline (ring buffer + resampler)
    ↓
VAD (energy-based, replaceable)
    ↓
Speech recognition (whisper.cpp, local, behind a trait)
    ↓
Transcript
    ↓
Deterministic content detection (rule-based; no LLM)
    ↓
Content resolver
    ├── Scripture → SQLite Bible library
    ├── Lyrics    → SQLite            (later phase)
    ├── Media     → local library
    └── Text      → presentation
    ↓
Presentation engine
    ↓
Display window (second monitor / projector)
```

Detected Scripture is **never** projected automatically: it appears as a
suggestion the operator must confirm with `Display`, `Edit` or `Ignore`.

---

## Status: phase 1 (bootstrap) complete

| Area                          | State                                                                   |
| ----------------------------- | ----------------------------------------------------------------------- |
| Tauri 2 + React 19 + TS shell | ✅ sidebar, routing, dark-first design system (Tailwind v4 + shadcn/ui) |
| SQLite + migrations           | ✅ bundled `rusqlite`, versioned migrations, repositories               |
| Settings persistence          | ✅ one JSON document in `settings`, written through immediately         |
| Live screen                   | ✅ transcript feed, detected-content review, projector state            |
| Bible screen                  | ✅ translation/book/chapter/verse selection, preview, Display, search   |
| Media screen                  | ✅ metadata-only library with path validation                           |
| Presentations screen          | ✅ create/delete, JSON item payloads                                    |
| Presentation window           | ✅ separate webview, positioned on the chosen monitor, fullscreen       |
| Audio                         | ✅ device enumeration, selection, capture start/stop, downmix, resample |
| Speech boundary               | ✅ `SpeechRecognizer` trait + Whisper impl (feature-gated) + dev mock   |
| VAD                           | ✅ `VoiceActivityDetector` trait + energy VAD, speech segments          |
| Scripture domain              | ✅ typed reference model, 66-book registry with spoken aliases          |
| Scripture parser              | ✅ deterministic; chapter/verse, ranges, spoken number words            |
| Content detection             | ✅ transcript → typed detections with confidence                        |
| Presentation engine           | ✅ generic items (scripture/text/media), queue + history                |
| Multi-monitor                 | ✅ `DisplayManager` (enumeration, target, open/close/fullscreen)        |
| Events                        | ✅ `audio://`, `speech://`, `content://`, `presentation://`             |
| TypeScript API layer          | ✅ `src/lib/api/*` — no `invoke` calls in components                    |
| Tests                         | ✅ 66 Rust tests (unit + integration), `tsc` and eslint clean           |

Phase 1 deliberately stops before real transcription — see
[Next phase](#next-phase).

---

## Requirements

| Tool      | Version  | Notes                                               |
| --------- | -------- | --------------------------------------------------- |
| Rust      | ≥ 1.77   | `rustup` recommended.                               |
| Node.js   | ≥ 20     |                                                     |
| pnpm      | ≥ 9      | `corepack enable pnpm`.                             |
| Xcode CLT | macOS    | Provides `clang`/`cc` for native builds.            |
| `cmake`   | optional | **Only** needed to build with `--features whisper`. |

Tauri's platform prerequisites (WebKit/WebView2, `libsoup`, …) must be installed
— see <https://v2.tauri.app/start/prerequisites/>.

No `.env` file is used or required.

---

## Getting started

```bash
pnpm install          # frontend dependencies
pnpm tauri dev        # launch the desktop app (Vite + Rust, hot reload)
```

Production build:

```bash
pnpm tauri build
```

### Development commands

| Command                        | What it does                                            |
| ------------------------------ | ------------------------------------------------------- |
| `pnpm dev`                     | Vite only (browser UI work; Tauri commands unavailable) |
| `pnpm build`                   | `tsc --noEmit` + production Vite build                  |
| `pnpm typecheck`               | TypeScript, no emit                                     |
| `pnpm lint`                    | ESLint (flat config)                                    |
| `pnpm test`                    | Vitest                                                  |
| `pnpm tauri dev`               | Full desktop app in dev mode                            |
| `pnpm tauri build`             | Bundled desktop application                             |
| `pnpm rust:check`              | `cargo check` on the Tauri crate                        |
| `pnpm rust:test`               | Rust unit + integration tests                           |
| `pnpm rust:lint`               | `cargo clippy --all-targets` (currently zero warnings)   |
| `pnpm rust:fmt`                | `cargo fmt`                                             |
| `pnpm rust:fmt:check`          | `cargo fmt --check`                                     |

---

## First run

1. **Setup screen** — pick a microphone, a Bible translation and the
   presentation display. Every step is skippable; nothing blocks you from
   reaching Live.
2. **Settings → Database** — import a translation you are licensed to use (see
   [`data/bible/README.md`](data/bible/README.md)). Selah ships no Bible text.
3. **Live** — press **Listen** to start the microphone pipeline.

Everything is stored under the OS application-data directory, never inside the
repository:

```text
<AppData>/app.selah.desktop/
├── database/selah.db     # SQLite (WAL)
├── models/whisper/       # ggml-*.bin
├── models/vad/
├── media/{images,videos,audio}
└── logs/
```

---

## Architecture

### Rust backend (`src-tauri/src`)

```text
audio/          device.rs capture.rs buffer.rs resampler.rs  — CPAL, no UI coupling
speech/         recognizer.rs (trait) whisper.rs (impl) vad.rs mock.rs manager.rs
scripture/      books.rs (registry) normalizer.rs parser.rs reference.rs resolver.rs
bible/          database.rs models.rs repository.rs import.rs
presentation/   engine.rs state.rs display.rs
media/          library.rs
db/             connection.rs migrations.rs
models/         content.rs presentation.rs settings.rs
commands/       audio speech scripture bible presentation media settings
state.rs        AppState (one managed instance, explicit locking)
errors.rs       AppError (typed; serialized without stack traces)
events.rs       event names + payloads
storage.rs      AppPaths (Tauri app-data resolution)
```

Rules the code follows:

- **Commands are thin adapters.** Business logic lives in modules; commands only
  borrow `State<AppState>` and delegate.
- **All SQL lives in repositories.** Services never embed SQL.
- **Infrastructure sits behind traits.** `SpeechRecognizer` and
  `VoiceActivityDetector` expose no Whisper-specific types, so the recognizer is
  replaceable and testable.
- **No global mutable state.** `AppState` is managed once; thread ownership is
  explicit (`Mutex`, `Arc<AtomicBool>`).
- **No `println!` debugging.** Structured `tracing` only.

### Presentation of content

`ContentType` (`scripture`, `lyrics`, `text`, `image`, `video`, `announcement`,
`slide`) plus a tagged `ContentPayload` keep the presentation engine generic. The
engine knows nothing about the Bible; the Bible module produces content.

### Frontend (`src`)

```text
components/ui/                 shadcn/ui primitives (button, card, select, …)
components/live/               Live-screen panels
components/presentations/      saved-presentation panels
hooks/                         useSettings, useLiveSession, useBooks, useSetupGate
lib/api/                       one module per command group — the ONLY invoke callers
lib/events.ts                  typed Tauri event subscriptions
lib/reference.ts               reference formatting (book names come from SQLite)
pages/                         Home, Live, Bible, Media, Presentations, Settings, Setup
presentation/                  projector webview entry point (presentation.html)
types/index.ts                 camelCase mirrors of the Rust serde types
```

Styling is **Tailwind CSS v4** (CSS-first config in `src/index.css`) with
**shadcn/ui** components on Radix primitives — no ad-hoc stylesheets, only
utility classes and design-token-backed components.

The projection window is a **separate webview** (`presentation.html`), so
congregational output can never accidentally show operator chrome.

---

## Database schema

Applied by `db/migrations.rs`; migrations are embedded at compile time and
recorded in `schema_migrations`.

| Table                | Purpose                                                                  |
| -------------------- | ------------------------------------------------------------------------ |
| `translations`       | Installed translations (`id`, `name`, `language`, `is_default`).         |
| `books`              | Canonical 66-book registry seeded by migration 1.                        |
| `verses`             | Verse text, PK `(translation_id, book_id, chapter, verse)`.              |
| `verses_fts`         | FTS5 index kept in sync by triggers; powers Bible search.                |
| `settings`           | Key/value store; the whole settings document under `app_settings`.       |
| `media`              | Media **metadata** only — files stay on disk.                            |
| `presentations`      | Saved presentations.                                                     |
| `presentation_items` | Ordered items; `payload` is JSON so new content types need no migration. |

Adding a migration: never edit an applied one — append a new
`migrations/000N_*.sql` and register it in `db/connection.rs::MIGRATIONS`.

---

## Speech recognition

whisper.cpp is compiled through `whisper-rs` behind an **optional cargo
feature**, because it needs `cmake` and a C++ toolchain:

```bash
# default build: real microphone → VAD pipeline, development recognizer
pnpm tauri dev

# real local transcription (after installing cmake)
pnpm tauri dev -- --features whisper
```

Without the feature the app runs a `MockSpeechRecognizer` that **returns an
empty transcript**. It never fabricates text and the UI labels it explicitly, so
a development build can never be mistaken for working transcription. No model is
downloaded automatically — see [`models/whisper/README.md`](models/whisper/README.md).

---

## Testing

```bash
cd src-tauri && cargo test            # 50 unit + 16 integration tests
cd src-tauri && cargo clippy --all-targets
cd src-tauri && cargo fmt --check
pnpm typecheck
pnpm lint
```

Rust coverage includes the specification's acceptance cases: spoken references
inside flowing speech, number-word conversion (`twenty eight` → 28), verse
ranges, book-alias normalization (`first corinthians` → `1 Corinthians`), and
negative cases (`"we have twenty people"`, `"chapter three was difficult"`) that
must **not** be detected.

---

## Next phase

```text
REAL MICROPHONE → VAD → WHISPER → SCRIPTURE DETECTION → PROJECTOR
```

Concretely, phase 2 is: install `cmake`, build `--features whisper`, load a
`ggml-*` model, validate transcript quality against a real service microphone,
tune the VAD thresholds, and add the model-management UI (plus a microphone level
meter) that goes with it.

Future phases (explicitly **not** started): lyrics/playlist management, cloud
sync, accounts, mobile apps, livestreaming, LLM features, speaker recognition.

---

## Licensing note

Selah contains no Bible text. Import only translations you are licensed to use.
