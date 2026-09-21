# Selah

## Project Overview

Selah helps operators manage church service presentations entirely offline. It captures local audio, detects spoken references, and prepares content for projection on a second screen. This allows teams to run reliable presentations without depending on internet connectivity or external cloud services.

## System Architecture

```mermaid
flowchart LR
  Mic["Microphone"]
  Audio["Audio Pipeline"]
  Recognizer["Speech Recognizer"]
  Detector["Content Detector"]
  SQLite[("Database")]
  Display["Presentation Display"]

  Mic --> Audio
  Audio --> Recognizer
  Recognizer --> Detector
  Detector --> SQLite
  SQLite --> Display

  style Mic fill:#1e1b4b,stroke:#6366f1,stroke-width:2px,color:#fff
  style Audio fill:#2e1065,stroke:#8b5cf6,stroke-width:2px,color:#fff
  style Recognizer fill:#451a03,stroke:#f59e0b,stroke-width:2px,color:#fff
  style Detector fill:#2e1065,stroke:#8b5cf6,stroke-width:2px,color:#fff
  style SQLite fill:#0f172a,stroke:#3b82f6,stroke-width:2px,color:#fff
  style Display fill:#1e1b4b,stroke:#6366f1,stroke-width:2px,color:#fff
```

## Features

- **Local Speech Recognition**: Captures audio locally, processes it through voice activity detection, and transcribes it without cloud services.

```mermaid
sequenceDiagram
  actor Speaker
  participant Mic
  participant VAD
  participant Recognizer
  participant UI

  Speaker->>Mic: Speak
  Mic->>VAD: Send audio buffer
  VAD->>Recognizer: Forward speech segment
  Recognizer->>UI: Return transcript
```

- **Deterministic Content Detection**: Analyzes transcripts for scripture references and matches them against a local database for review before projection.

```mermaid
sequenceDiagram
  participant UI
  participant Detector
  participant DB as "Database"
  participant Display

  UI->>Detector: Send transcript
  Detector->>Detector: Parse scripture reference
  Detector->>DB: Query passage text
  DB->>UI: Return passage
  UI->>Display: Operator approves projection
```

- **Multi-Monitor Presentation**: Manages separate windows for the operator interface and the congregation display, keeping controls hidden from the audience.
- **Songs Library**: Songs are stored as ordered sections (verse, chorus, bridge) and presented one section at a time — the same step-by-step flow as Scripture, so a long song never has to be squeezed onto one screen.
- **Media Folders**: Selah never hardcodes a media path. The operator picks a folder at runtime, Selah reads the pictures, videos and sound files inside it — and inside its sub-folders when the switch is on — and only that folder is readable by the projector window. The page lists what that folder holds, says so when a file appeared since last time, and reports any folder it could not read.
- **Video Time Ranges**: a video can be trimmed without editing the file: **Preview** on a video plays it, sets a start and an end (6s → 30s), and decides whether the chosen part stops there or starts again. The choice can be saved with the file, so **Show** uses it from then on, and Settings → Screen holds the default for every other video.
- **Projector Look**: Background colour, text size and typeface are set in Settings and applied to the congregation's screen when Save is pressed — an already-open projector window updates immediately. Media always fits the screen whole and is centred, rather than being cropped or zoomed to fill it.

## Design System

### Typeface

Selah uses **Creato Display** throughout the operator interface, and lets the
_projected_ text use any of five bundled typefaces — so the words on the wall can
be matched to the room without changing the interface.

- Self-hosted from `src/fonts/` and bundled by Vite. Selah never fetches a font
  over the network, because it has to run on a machine with no internet.
- Creato Display weights bundled: Light (300), Regular (400), Medium (500),
  Bold (700), Black (900). Italics are omitted — nothing in the interface uses
  them.
- Projector choices are listed in `FONT_OPTIONS` (`src/lib/fonts.ts`) and offered
  by the typeface picker in Settings → Screen: Creato Display, Inter, Lora,
  Oswald and JetBrains Mono. The four extras are variable fonts (28–48 KB each).
- All are licensed under the **SIL Open Font License 1.1**, which permits
  embedding in an application. Creato Display: copyright (c) 2021 Anugrah Pasau,
  with Reserved Font Name "Creato Display" — the files must not be renamed or
  modified. Full texts live in `src/fonts/OFL.txt` and `src/fonts/licenses/`;
  see `src/fonts/README.md` for provenance and how to add a family.

### Brand palette

Both colours are sampled directly from the logo artwork:

| Token     | Value     | Role                                   |
| --------- | --------- | -------------------------------------- |
| Logo gold | `#C9A24B` | Brand mark, active navigation, buttons |
| Logo navy | `#141B2E` | Interface base, logo tile, icon        |

The neutral greys are tinted toward the logo navy, so the interface reads as one
brand rather than a grey theme with a gold highlight added afterwards.

Because the logo gold only reaches about 2.4:1 contrast against white, the light
theme uses a darkened version of the same hue for anything interactive. The logo
itself keeps the true gold and navy in both themes via `--brand-mark` and
`--brand-tile`, so the branding never shifts.

### Logo component

`src/components/ui/selah-icon.tsx` renders the logo. Use it everywhere the mark
appears rather than repeating the artwork:

```tsx
<SelahIcon size={26} tile />            // application mark, e.g. sidebar header
<SelahIcon size={120} />                // on a dark surface, e.g. projector slate
```

### Application icon

`src-tauri/icons/selah-icon.svg` is the source of truth for every platform icon
and uses the same 48-unit geometry as the component above. It is a rounded
squircle with a ~23% corner radius, matching how macOS and Windows round their
own application tiles. After editing it, regenerate the whole set:

```bash
pnpm tauri icon src-tauri/icons/selah-icon.svg
```

## Installation

- Clone the Repository:

```bash
git clone https://github.com/thevalidcode/selah.git
```

- Ensure you have Node.js 20 or higher, pnpm 9 or higher, and Rust 1.77 or higher installed on your system.
- Install frontend dependencies:

```bash
pnpm install
```

- Run the application in development mode:

```bash
pnpm tauri dev
```

Two things must be true for Selah to turn speech into words: the `moonshine`
build feature (on by default) and a model folder on disk. Selah never
downloads a model itself.

1. Make ONNX Runtime available. Selah loads it at start-up, so it needs the
   shared library somewhere it looks — `ORT_DYLIB_PATH`, the app's resource
   folder, or `/usr/local/lib`. See
   [`models/moonshine/README.md`](models/moonshine/README.md) for the full
   search order.

2. Put a Moonshine model folder on disk and point Settings at it:

```bash
# The model can live anywhere; the app data directory is the documented default.
DEST="$HOME/Library/Application Support/app.selah.desktop/models/moonshine"
mkdir -p "$DEST"
cp ~/Downloads/encoder_model_quantized.onnx \
   ~/Downloads/decoder_model_quantized.onnx \
   ~/Downloads/tokenizer.json \
   "$DEST/"
```

Then open **Settings → Listening**, set **Can Selah understand words?** to
_On_, and point **Voice model folder** at that folder.

> **Careful with "int8" files.** A dynamically-quantised `int8` export contains
> `ConvInteger` nodes, which ONNX Runtime's CPU provider cannot execute (it
> supports only the uint8 form). Selah then reports
> `Could not find an implementation for ConvInteger(10) node`. Use the
> `_quantized` (QDQ) or unquantised files from the same model instead —
> `models/moonshine/README.md` explains which is which.

If the model or the runtime is missing, Selah still runs — it just reports
_"Voice model ready: no"_ on the Live screen and returns no words.

## Usage

When you first launch the application, you will be greeted by the setup screen. Follow the prompts to configure your environment.

1.  **Select a Microphone**: Choose the input device you want Selah to listen to.
2.  **Import a Bible Translation**: Navigate to Settings, select Database, and import a translation JSON file.
3.  **Choose a Display**: Select the monitor where the presentation window will appear.
4.  **Start Listening**: Go to the Live screen and click the Listen button to activate the audio pipeline.

Detected scripture will appear in the review panel. Click Display to send the content to the presentation window.

### A round trip you can demo

1. **Songs** → write a title and two sections (label + words) → **Save** → **Show on screen**. Each section is one screen; **Next** / **Back** step through the song verse by verse. In a hurry? **Paste words** takes one long block — numbered lines, blank lines, one verse per line, or a single paragraph of sentences — shows how it will be cut into verses, then fills the sections for you.
2. **Media** → **Choose folder…** → walk to the folder with your pictures and videos → **Read this folder** → **Show** on any file. Selah opens on the folder you used last time and reads it again, so a file added since — in the folder or in any of its sub-folders (**Include sub-folders**) — is simply there. The list shows that folder's files; **Show every file** widens it to everything Selah has ever read. On a video, **Preview** plays it and lets you choose the part to show: set **Starts at** 0:06 and **Stops at** 0:30, decide whether it **starts again at the end**, press **Play the chosen part** to check it and **Save the range** to keep it — or **Show on screen** straight away. Pictures and video are fitted inside the window rather than cropped, centred in the background colour that fills the space around them; the folder you picked is the only one Selah may read. **On the screen now** tells you whether a video is really playing, which seconds of it are showing, and gives you **Play**, **Pause**, **Start again** and **Stop** from the operator screen.
3. **Settings → Screen** → change the background colour, drag the text size, pick a typeface (each option is drawn in its own font) → add your **logo and name**, choose which edge it hugs and how big it is → the "What the branding looks like" panel shows the actual image and the name in the chosen typeface at the chosen size, and **Save settings** makes it real on an already-open projector window.
4. **Bible → Look up a verse** → the screen opens on chapter 1, verse 3. Change the **heading size** and **verse size** for this passage only and a warning appears if it will not fit the chosen screen. Search words across **every** installed Bible; if the words are not there you get the closest matches, or passages about the same subject to try instead.
5. **Bible → Add a Translation** → the published Bibles are listed (NKJV, NIV, ESV, NASB, MSG, AMP, NLT, CSB, HCSB, NET, RSV, NRSV, GNT, CEV, TLB and more): press **Add** on one and it is created, ready for verses. Then store the words — switch on **Enter one verse** and type only the message (`Jesus wept.`; Selah refuses anything with the reference still attached), or switch on **Paste a whole chapter**, press **Copy the prompt** (it names the chapter _and_ the translation), give it to an AI chat and paste the array back: every verse is checked before anything is saved. Anything you added can be renamed or removed on the same screen; the three Bibles Selah ships (WEB, KJV, ASV) are marked as its own and cannot be changed.
6. **Presentations** → add an item of kind **Notice** and one of kind **Bible verse** → **Show on screen**: a notice is drawn as a card with a gold accent bar and its own colour, so the congregation can tell an announcement from Scripture at a glance. Use **Edit** on any row to change its words without moving it in the list.

## Technologies Used

| Category     | Technology                                 |
| ------------ | ------------------------------------------ |
| Frontend     | React, TypeScript, Tailwind CSS, shadcn/ui |
| Backend      | Rust, Tauri                                |
| Database     | SQLite                                     |
| Audio/Speech | CPAL, Moonshine on ONNX Runtime            |

## API Documentation

The application backend uses Tauri IPC commands for communication with the frontend. Below are the registered endpoints.

#### [IPC] list_audio_devices

**Description**: Lists available audio input devices.

**Request**:

```json
{}
```

**Response**:

```json
{
  "status": "success",
  "data": [
    {
      "id": "device_1",
      "name": "Microphone",
      "isDefault": true,
      "defaultSampleRate": 48000,
      "channels": 2
    }
  ]
}
```

**Errors**:

- 500: Internal error fetching audio devices

#### [IPC] get_default_audio_device

**Description**: Returns the system default audio input device.

**Request**:

```json
{}
```

**Response**:

```json
{
  "status": "success",
  "data": {
    "id": "device_1",
    "name": "Microphone",
    "isDefault": true,
    "defaultSampleRate": 48000,
    "channels": 2
  }
}
```

**Errors**:

- 500: Audio query failed

#### [IPC] start_audio_capture

**Description**: Starts microphone capture using the configured device.

**Request**:

```json
{}
```

**Response**:

```json
{
  "status": "success",
  "data": {
    "capturing": true,
    "deviceId": "device_1",
    "sampleRate": 16000,
    "channels": 1,
    "bufferedSamples": 0
  }
}
```

**Errors**:

- 400: Audio device not found
- 500: Audio capture failed

#### [IPC] stop_audio_capture

**Description**: Stops active microphone capture.

**Request**:

```json
{}
```

**Response**:

```json
{
  "status": "success",
  "data": null
}
```

**Errors**:

- 500: Internal error stopping capture

#### [IPC] get_audio_capture_state

**Description**: Retrieves the current status of the audio pipeline.

**Request**:

```json
{}
```

**Response**:

```json
{
  "status": "success",
  "data": {
    "capturing": true,
    "deviceId": "device_1",
    "sampleRate": 16000,
    "channels": 1,
    "bufferedSamples": 512
  }
}
```

**Errors**:

- 500: Internal error getting state

#### [IPC] get_speech_state

**Description**: Retrieves the status of the speech recognition manager.

**Request**:

```json
{}
```

**Response**:

```json
{
  "status": "success",
  "data": {
    "listening": true,
    "recognizerId": "moonshine",
    "modelLoaded": true,
    "vadEnabled": true,
    "segmentsSeen": 12,
    "transcriptsGenerated": 10
  }
}
```

**Errors**:

- 500: Internal error getting speech state

#### [IPC] start_listening

**Description**: Starts both microphone capture and the speech recognition worker.

**Request**:

```json
{}
```

**Response**:

```json
{
  "status": "success",
  "data": {
    "listening": true,
    "recognizerId": "moonshine",
    "modelLoaded": true,
    "vadEnabled": true,
    "segmentsSeen": 0,
    "transcriptsGenerated": 0
  }
}
```

**Errors**:

- 500: Failed to spawn speech worker

#### [IPC] stop_listening

**Description**: Stops both the recognition worker and microphone capture.

**Request**:

```json
{}
```

**Response**:

```json
{
  "status": "success",
  "data": {
    "listening": false,
    "recognizerId": "moonshine",
    "modelLoaded": true,
    "vadEnabled": true,
    "segmentsSeen": 5,
    "transcriptsGenerated": 5
  }
}
```

**Errors**:

- 500: Internal error stopping speech pipeline

#### [IPC] start_audio_pipeline

**Description**: Starts microphone capture only, without transcribing.

**Request**:

```json
{}
```

**Response**:

```json
{
  "status": "success",
  "data": {
    "capturing": true,
    "deviceId": "device_1",
    "sampleRate": 16000,
    "channels": 1,
    "bufferedSamples": 0
  }
}
```

**Errors**:

- 400: Audio device not found

#### [IPC] stop_audio_pipeline

**Description**: Stops the standalone microphone capture.

**Request**:

```json
{}
```

**Response**:

```json
{
  "status": "success",
  "data": null
}
```

**Errors**:

- 500: Internal error stopping audio

#### [IPC] detect_scripture

**Description**: Runs deterministic rule-based content detection on a transcript.

**Request**:

```json
{
  "transcript": "John chapter three verse sixteen"
}
```

**Response**:

```json
{
  "status": "success",
  "data": [
    {
      "content": {
        "type": "scripture",
        "reference": {
          "bookId": 43,
          "chapter": 3,
          "startVerse": 16,
          "endVerse": 16
        }
      },
      "confidence": 0.95
    }
  ]
}
```

**Errors**:

- 500: Detection internal error

#### [IPC] list_bible_translations

**Description**: Lists all imported Bible translations and their verse counts.

**Request**:

```json
{}
```

**Response**:

```json
{
  "status": "success",
  "data": [
    {
      "translation": {
        "id": "web",
        "name": "World English Bible",
        "language": "en",
        "abbreviation": "WEB",
        "isDefault": true,
        "createdAt": "2023-10-01T12:00:00Z"
      },
      "verseCount": 31102
    }
  ]
}
```

**Errors**:

- 500: Database error listing translations

#### [IPC] list_books

**Description**: Lists all canonical Bible books in the registry.

**Request**:

```json
{}
```

**Response**:

```json
{
  "status": "success",
  "data": [
    {
      "id": 1,
      "name": "Genesis",
      "testament": "Old",
      "abbreviation": "Gen",
      "chapters": 50
    }
  ]
}
```

**Errors**:

- 500: Database error listing books

#### [IPC] get_passage

**Description**: Fetches a single verse or range of verses from a specific translation.

**Request**:

```json
{
  "request": {
    "translationId": "web",
    "bookId": 43,
    "chapter": 3,
    "startVerse": 16,
    "endVerse": 16
  }
}
```

**Response**:

```json
{
  "status": "success",
  "data": {
    "translationId": "web",
    "reference": "John 3:16",
    "verses": [
      {
        "translationId": "web",
        "bookId": 43,
        "chapter": 3,
        "verse": 16,
        "text": "For God so loved the world..."
      }
    ],
    "text": "For God so loved the world..."
  }
}
```

**Errors**:

- 400: Invalid scripture reference
- 404: Bible translation or scripture not found
- 500: Database error

#### [IPC] search_bible

**Description**: Searches verse text, optionally across **every** installed translation. When the exact words are not found the reply carries the closest matches (with `related: true`) or, if nothing is close at all, themes to try instead — so a search never dead-ends.

**Request**:

```json
{
  "translationId": null,
  "query": "shepherd",
  "limit": 25
}
```

`translationId` may be a translation id (`"web"`), or `null` / `""` / `"*"` / `"@all"` to search every installed translation.

**Response**:

```json
{
  "status": "success",
  "data": {
    "verses": [
      {
        "translationId": "kjv",
        "bookId": 19,
        "chapter": 23,
        "verse": 1,
        "text": "The LORD is my shepherd; I shall not want."
      }
    ],
    "related": false,
    "suggestions": []
  }
}
```

When nothing matches, `suggestions` carries references (never text — Selah ships no Bible text):

```json
{
  "verses": [],
  "related": false,
  "suggestions": [
    {
      "topic": "Comfort",
      "why": "comfort in trouble",
      "bookId": 19,
      "chapter": 23,
      "verse": 1,
      "reference": "Psalms 23:1"
    }
  ]
}
```

**Errors**:

- 500: Database error during search

#### [IPC] suggest_bible_topics

**Description**: Returns passages about the subject a query describes, for when an operator knows what they want to say but not where it is. References only.

**Request**:

```json
{
  "query": "worried about money",
  "limit": 6
}
```

**Response**:

```json
{
  "status": "success",
  "data": [
    {
      "topic": "Money",
      "why": "for giving, debt and worry about provision",
      "bookId": 40,
      "chapter": 6,
      "verse": 33,
      "reference": "Matthew 6:33"
    }
  ]
}
```

**Errors**: none — an unrecognised query returns the starter passages rather than an error.

#### [IPC] list_translation_catalogue

**Description**: Every published Bible translation Selah knows how to hold (KJV, NKJV, ASV, WEB, RSV, NRSV, KJ21, NIV, ESV, NASB, MSG, AMP, NLT, CSB, HCSB, NET, GNT, CEV, TLB, NCV, GNB, ERV…), merged with what is installed. Metadata only — Selah never downloads or ships Bible text. Anything installed that is not in the list (imported from a file, or added by hand) is included as well.

**Request**: none

**Response**:

```json
{
  "status": "success",
  "data": [
    {
      "id": "msg",
      "name": "The Message",
      "abbreviation": "MSG",
      "language": "en",
      "group": "Modern",
      "builtin": false,
      "installed": false,
      "verseCount": 0,
      "publicDomain": false
    },
    {
      "id": "kjv",
      "name": "King James Version",
      "abbreviation": "KJV",
      "language": "en",
      "group": "Classic",
      "builtin": true,
      "installed": true,
      "verseCount": 31102,
      "publicDomain": true
    }
  ]
}
```

**Errors**:

- 500: Database error reading the installed translations

#### [IPC] add_translation

**Description**: Adds a translation so verses can be kept under it. A published id (`msg`) brings its own name, abbreviation and language so the picker stays consistent; an id Selah does not list needs a `name`. The three Bibles Selah ships (`web`, `kjv`, `asv`) are refused — they are always present. A new translation never becomes the default.

**Request**:

```json
{
  "request": {
    "id": "msg",
    "name": null,
    "abbreviation": null,
    "language": null
  }
}
```

**Response**:

```json
{
  "status": "success",
  "data": {
    "translation": {
      "id": "msg",
      "name": "The Message",
      "language": "en",
      "abbreviation": "MSG",
      "isDefault": false,
      "createdAt": "2026-09-19T06:40:00Z",
      "builtin": false,
      "origin": "catalogue"
    },
    "verseCount": 0
  }
}
```

**Errors**:

- 400: the id is one of Selah's own, or a translation Selah does not list was added without a name
- 500: Database error writing the translation

#### [IPC] update_translation

**Description**: Renames a translation the operator added, or changes the short name shown in the picker. The three bundled Bibles are refused: the application relies on their names.

**Request**:

```json
{
  "request": {
    "id": "msg",
    "name": "The Message (paraphrase)",
    "abbreviation": "MSG",
    "language": "en"
  }
}
```

**Response**:

```json
{
  "status": "success",
  "data": {
    "id": "msg",
    "name": "The Message (paraphrase)",
    "abbreviation": "MSG",
    "builtin": false,
    "origin": "catalogue"
  }
}
```

**Errors**:

- 400: the name is empty, or the translation is one of Selah's own
- 500: translation not found, or a database error

#### [IPC] delete_translation

**Description**: Removes a translation the operator added, along with every verse stored under it (the schema cascades). The three bundled Bibles are refused. If the translation was the default, the flag moves to another installed translation and `settings.general.defaultTranslationId` is cleared when it pointed at the removed one.

**Request**:

```json
{
  "id": "msg"
}
```

**Response**:

```json
{
  "status": "success",
  "data": null
}
```

**Errors**:

- 400: the translation is one of Selah's own
- 500: translation not found, or a database error

**Description**: Stores verses the operator supplied under the translation they chose, which then behaves like any other Bible. Every reference is checked against the canonical book registry before anything is written: the book must exist, the chapter must exist in that book, all verses must belong to one book and one chapter, and verse numbers must be unique with real words in them. Writing into one of the three bundled Bibles is refused — add a translation of your own for verses you supply.

**Request**:

```json
{
  "request": {
    "translationId": "msg",
    "name": null,
    "abbreviation": null,
    "verses": [
      {
        "book": "John",
        "chapter": 3,
        "verse": 16,
        "text": "This is how much God loved the world"
      }
    ]
  }
}
```

`name` is only used when the translation does not exist yet; adding verses to an installed translation never renames it.

**Response**:

```json
{
  "status": "success",
  "data": {
    "translationId": "msg",
    "reference": "John 3 (1 verses)",
    "bookId": 43,
    "chapter": 3,
    "versesSaved": 1,
    "missingVerses": []
  }
}
```

**Errors**:

- 400: an unknown book, a chapter that does not exist, a paste mixing books or chapters, repeated verse numbers, empty words, code pasted where verse text belongs, or a bundled translation
- 500: Database error while writing the verses

#### [IPC] set_default_translation

**Description**: Sets the default translation used when one is not explicitly specified.

**Request**:

```json
{
  "translationId": "web"
}
```

**Response**:

```json
{
  "status": "success",
  "data": null
}
```

**Errors**:

- 404: Bible translation not found
- 500: Database error setting default

#### [IPC] import_bible_translation

**Description**: Imports a Bible translation from a local JSON file path.

**Request**:

```json
{
  "path": "/Users/operator/Downloads/web-bible.json"
}
```

**Response**:

```json
{
  "status": "success",
  "data": {
    "translationId": "web",
    "versesImported": 31102
  }
}
```

**Errors**:

- 400: Invalid configuration or invalid import path
- 500: Database error during import

#### [IPC] get_presentation_state

**Description**: Returns the current state of the presentation engine queue.

**Request**:

```json
{}
```

**Response**:

```json
{
  "status": "success",
  "data": {
    "current": null,
    "queue": [],
    "history": []
  }
}
```

**Errors**:

- 500: Presentation engine error

#### [IPC] project_text

**Description**: Sends plain text to the presentation display.

**Request**:

```json
{
  "request": {
    "title": "Welcome",
    "text": "Please silence your phones.",
    "display": 1,
    "fullscreen": true
  }
}
```

**Response**:

```json
{
  "status": "success",
  "data": {
    "current": {
      "id": "item_1",
      "contentType": "text",
      "title": "Welcome",
      "payload": {
        "kind": "text",
        "text": "Please silence your phones."
      }
    },
    "queue": [],
    "history": []
  }
}
```

**Errors**:

- 400: Display not found
- 500: Presentation error

#### [IPC] project_passage

**Description**: Sends a Bible passage to the presentation display. `headingSize` and `textSize` are optional per-passage overrides in CSS pixels, set from the Bible screen; omit them (or send `null`) to keep the sizes saved in Settings.

**Request**:

```json
{
  "request": {
    "passage": {
      "translationId": "web",
      "reference": "John 3:16",
      "verses": [],
      "text": "For God so loved the world..."
    },
    "headingSize": 120,
    "textSize": 72
  }
}
```

**Response**:

```json
{
  "status": "success",
  "data": {
    "current": {
      "id": "item_1",
      "contentType": "scripture",
      "title": "web John 3:16",
      "payload": {
        "kind": "scripture",
        "reference": "John 3:16",
        "translation": "web",
        "text": "For God so loved the world...",
        "headingSize": 120,
        "textSize": 72
      }
    },
    "queue": [],
    "history": []
  }
}
```

**Errors**:

- 500: Presentation error projecting passage

#### [IPC] clear_presentation

**Description**: Clears the current content from the presentation display.

**Request**:

```json
{}
```

**Response**:

```json
{
  "status": "success",
  "data": {
    "current": null,
    "queue": [],
    "history": []
  }
}
```

**Errors**:

- 500: Presentation error clearing display

#### [IPC] open_presentation_window

**Description**: Opens the presentation window on a selected display monitor.

**Request**:

```json
{
  "display": 1,
  "fullscreen": true
}
```

**Response**:

```json
{
  "status": "success",
  "data": {
    "index": 1,
    "name": "Display 2",
    "size": [1920, 1080],
    "position": [1920, 0],
    "scaleFactor": 1.0,
    "isPrimary": false
  }
}
```

**Errors**:

- 400: Display not found
- 500: Presentation error opening window

#### [IPC] close_presentation_window

**Description**: Closes the presentation window.

**Request**:

```json
{}
```

**Response**:

```json
{
  "status": "success",
  "data": null
}
```

**Errors**:

- 500: Presentation error closing window

#### [IPC] set_presentation_display

**Description**: Sets the target display index for the presentation window.

**Request**:

```json
{
  "display": 1
}
```

**Response**:

```json
{
  "status": "success",
  "data": {
    "index": 1,
    "size": [1920, 1080],
    "position": [1920, 0],
    "scaleFactor": 1.0,
    "isPrimary": false
  }
}
```

**Errors**:

- 400: Display not found
- 500: Presentation error setting target

#### [IPC] set_fullscreen

**Description**: Toggles fullscreen mode for the presentation window.

**Request**:

```json
{
  "fullscreen": true
}
```

**Response**:

```json
{
  "status": "success",
  "data": null
}
```

**Errors**:

- 400: Display not found
- 500: Presentation error setting fullscreen

#### [IPC] list_displays

**Description**: Enumerates all monitors detected by the operating system.

**Request**:

```json
{}
```

**Response**:

```json
{
  "status": "success",
  "data": [
    {
      "index": 0,
      "name": "Built-in Display",
      "size": [1440, 900],
      "position": [0, 0],
      "scaleFactor": 2.0,
      "isPrimary": true
    }
  ]
}
```

**Errors**:

- 500: Display not found or error listing monitors

#### [IPC] list_presentations

**Description**: Lists all saved presentations.

**Request**:

```json
{}
```

**Response**:

```json
{
  "status": "success",
  "data": [
    {
      "id": "pres_1",
      "name": "Sunday Morning",
      "createdAt": "2023-10-01T12:00:00Z",
      "updatedAt": "2023-10-01T12:00:00Z"
    }
  ]
}
```

**Errors**:

- 500: Database error listing presentations

#### [IPC] get_presentation

**Description**: Retrieves a specific presentation and its ordered items.

**Request**:

```json
{
  "id": "pres_1"
}
```

**Response**:

```json
{
  "status": "success",
  "data": {
    "id": "pres_1",
    "name": "Sunday Morning",
    "createdAt": "2023-10-01T12:00:00Z",
    "updatedAt": "2023-10-01T12:00:00Z",
    "items": []
  }
}
```

**Errors**:

- 404: Presentation not found
- 500: Database error fetching presentation

#### [IPC] create_presentation

**Description**: Creates a new empty presentation.

**Request**:

```json
{
  "name": "Sunday Evening"
}
```

**Response**:

```json
{
  "status": "success",
  "data": {
    "id": "pres_2",
    "name": "Sunday Evening",
    "createdAt": "2023-10-01T18:00:00Z",
    "updatedAt": "2023-10-01T18:00:00Z"
  }
}
```

**Errors**:

- 500: Database error creating presentation

#### [IPC] delete_presentation

**Description**: Deletes a saved presentation.

**Request**:

```json
{
  "id": "pres_1"
}
```

**Response**:

```json
{
  "status": "success",
  "data": null
}
```

**Errors**:

- 500: Database error deleting presentation

#### [IPC] add_presentation_item

**Description**: Adds a new item payload to a saved presentation.

**Request**:

```json
{
  "request": {
    "presentationId": "pres_1",
    "type": "text",
    "payload": "{\"kind\":\"text\",\"text\":\"Welcome\"}"
  }
}
```

**Response**:

```json
{
  "status": "success",
  "data": null
}
```

**Errors**:

- 500: Database error adding item

#### [IPC] update_presentation_item

**Description**: Replaces the content of an item already saved in a presentation, keeping its place in the list. This is what the Edit button in the items table uses. The payload is read back as a `ContentPayload` before it is stored, so a broken edit is refused rather than discovered later on the wall.

**Request**:

```json
{
  "request": {
    "presentationId": "pres_1",
    "itemId": "item_1",
    "type": "announcement",
    "payload": "{\"kind\":\"text\",\"heading\":\"Notices\",\"text\":\"Tea after the service\"}"
  }
}
```

**Response**:

```json
{
  "status": "success",
  "data": null
}
```

**Errors**:

- 400: the payload could not be read back, or an item with no words
- 500: item not part of that presentation, or a database error

#### [IPC] remove_presentation_item

**Description**: Removes a specific item from a saved presentation.

**Request**:

```json
{
  "presentationId": "pres_1",
  "itemId": "item_1"
}
```

**Response**:

```json
{
  "status": "success",
  "data": null
}
```

**Errors**:

- 400: Item not found in presentation
- 500: Database error removing item

#### [IPC] list_media

**Description**: Lists the metadata of every media file Selah has read. Files themselves are never copied — SQLite stores metadata only. `metadata` is a free-form record: the file's size and kind, plus anything the operator chose about it — a video's `clip` (`startMs`, `endMs`, `repeat`) written by `set_media_clip`, which the Media screen reads so a trimmed video keeps its range.

**Request**:

```json
{}
```

**Response**:

```json
{
  "status": "success",
  "data": [
    {
      "id": "media_1",
      "kind": "video",
      "name": "worship.mp4",
      "path": "/Users/operator/Pictures/worship.mp4",
      "metadata": {
        "size": 5242880,
        "kind": "video",
        "clip": { "startMs": 6000, "endMs": 30000, "repeat": true }
      },
      "createdAt": "2023-10-01T12:00:00Z"
    }
  ]
}
```

**Errors**:

- 500: Database error listing media

#### [IPC] browse_directory

**Description**: Lists one folder so the operator can walk to the folder their media lives in. No path is hardcoded in the application; omitting `path` starts at the user's home folder.

**Request**:

```json
{
  "path": "/Users/operator/Pictures"
}
```

**Response**:

```json
{
  "status": "success",
  "data": {
    "path": "/Users/operator/Pictures",
    "parent": "/Users/operator",
    "entries": [
      {
        "name": "Sunday",
        "path": "/Users/operator/Pictures/Sunday",
        "isDir": true
      },
      {
        "name": "baptism.jpg",
        "path": "/Users/operator/Pictures/baptism.jpg",
        "isDir": false,
        "mediaKind": "image"
      }
    ]
  }
}
```

**Errors**:

- 400: The path is not a folder, or cannot be read

#### [IPC] load_media_directory

**Description**: Reads a folder and registers the pictures, videos and sound files inside it — and, when `recursive` is set, the files in its sub-folders too. The folder is remembered in settings (the Media screen opens on it and reads it again next time) and opened to the projector window, so only the folder the operator chose is readable. `items` is what that folder now holds, which is what the Media screen lists; `skipped` and `truncated` are what the scan could not finish, so the interface can say so instead of leaving files apparently missing.

**Request**:

```json
{
  "request": {
    "path": "/Users/operator/Pictures",
    "recursive": false
  }
}
```

**Response**:

```json
{
  "status": "success",
  "data": {
    "directory": "/Users/operator/Pictures",
    "added": 12,
    "total": 12,
    "skipped": 0,
    "truncated": false,
    "items": []
  }
}
```

**Errors**:

- 400: The folder does not exist or cannot be read
- 500: Database error registering media

#### [IPC] project_media

**Description**: Puts a media file on the congregation's screen. The presentation window loads the file through Tauri's `asset:` protocol, because a webview cannot read a plain filesystem path. `clip` is optional and describes the part of a **video** to show: `startMs` is where playback begins, `endMs` where it stops (`null` runs to the end), and `repeat` whether it starts again at the end instead of stopping there. Leaving `clip` out plays the file whole and lets the saved Screen setting answer the repeat question; a picture and a piece of music never carry one.

**Request**:

```json
{
  "request": {
    "path": "/Users/operator/Pictures/worship.mp4",
    "title": null,
    "clip": {
      "startMs": 6000,
      "endMs": 30000,
      "repeat": true
    }
  }
}
```

**Response**:

```json
{
  "status": "success",
  "data": {
    "current": {
      "id": "8f1c…",
      "contentType": "video",
      "title": "worship.mp4",
      "payload": {
        "kind": "media",
        "path": "/Users/operator/Pictures/worship.mp4",
        "mediaKind": "video",
        "startMs": 6000,
        "endMs": 30000,
        "repeat": true
      }
    },
    "queue": [],
    "history": []
  }
}
```

**Errors**:

- 400: The file does not exist, or Selah cannot present that file type
- 500: Presentation window could not be opened

#### [IPC] set_media_clip

**Description**: Remembers which part of a video to show, for next time. The range and the repeat choice are kept with the file in the library (in the media record's metadata, so nothing is copied and no migration is needed), which is what makes **Show** behave the same way on a later Sunday without setting it up again. Sending `clip: null` forgets the range and plays the file whole. The ranges are clamped: an end that is not after the start is treated as "run to the end".

**Request**:

```json
{
  "request": {
    "id": "media_2",
    "clip": {
      "startMs": 6000,
      "endMs": 30000,
      "repeat": true
    }
  }
}
```

**Response**:

```json
{
  "status": "success",
  "data": {
    "id": "media_2",
    "kind": "video",
    "name": "worship.mp4",
    "path": "/Users/operator/Pictures/worship.mp4",
    "metadata": {
      "size": 5242880,
      "kind": "video",
      "clip": { "startMs": 6000, "endMs": 30000, "repeat": true }
    },
    "createdAt": "2026-09-21T09:00:00+00:00"
  }
}
```

**Errors**:

- 400: No media file with that id
- 500: Database error saving the range

#### [IPC] get_media_playback_state

**Description**: What the projector window says it is doing. Video and sound keep moving after they are on the screen, and only the projector knows whether they really started, so the operator screen reads the truth from here rather than guessing.

**Request**: none

**Response**:

```json
{
  "status": "success",
  "data": {
    "itemId": "8f1c…",
    "mediaKind": "video",
    "playing": true,
    "positionMs": 12400,
    "durationMs": 96000,
    "ended": false
  }
}
```

**Errors**: none — an empty screen returns an empty snapshot.

#### [IPC] control_media_playback

**Description**: Asks the projector window to play, pause, restart or stop what is on it. The instruction travels as the `media://playback` event because the projector is a separate webview: only it can touch the `<video>` element. It names the item it is for, so a button pressed a moment too late cannot affect the next file.

**Request**:

```json
{
  "request": {
    "action": "pause"
  }
}
```

**Response**: the same shape as `get_media_playback_state`, already updated optimistically; the projector's own report replaces it a moment later.

**Errors**:

- 400: the action is not `play`, `pause`, `restart` or `stop`
- 500: nothing is on the screen to play

#### [IPC] report_media_playback

**Description**: Called by the **projector window** — never by the operator screen — with what its player is actually doing. The state is stored and re-emitted on `media://playback-state`, so every open screen agrees. A report about an item that is no longer on the screen is ignored.

**Request**:

```json
{
  "report": {
    "playing": true,
    "positionMs": 12400,
    "durationMs": 96000,
    "ended": false
  }
}
```

**Response**: the stored snapshot, identical to `get_media_playback_state`.

**Errors**: none — a report from a stale item is dropped rather than refused.

**Description**: Registers a single local file (used for one-off files that live outside the chosen media folder).

**Request**:

```json
{
  "request": {
    "path": "/Users/operator/Pictures/sermon-slide.png"
  }
}
```

**Response**:

```json
{
  "status": "success",
  "data": {
    "id": "media_2",
    "kind": "image",
    "name": "sermon-slide.png",
    "path": "/Users/operator/Pictures/sermon-slide.png",
    "createdAt": "2023-10-01T12:05:00Z"
  }
}
```

**Errors**:

- 400: Invalid file path or file does not exist
- 500: Database error importing media

#### [IPC] remove_media

**Description**: Removes media metadata from the library without deleting the physical file.

**Request**:

```json
{
  "id": "media_1"
}
```

**Response**:

```json
{
  "status": "success",
  "data": null
}
```

**Errors**:

- 500: Database error removing media

#### [IPC] list_songs

**Description**: Lists every song in the library (titles and authors; sections are loaded with `get_song`).

**Request**:

```json
{}
```

**Response**:

```json
{
  "status": "success",
  "data": [
    {
      "id": "song_1",
      "title": "Amazing Grace",
      "author": "John Newton",
      "createdAt": "2023-10-01T12:00:00Z",
      "updatedAt": "2023-10-01T12:00:00Z"
    }
  ]
}
```

**Errors**:

- 500: Database error listing songs

#### [IPC] get_song

**Description**: Loads one song with its sections in presentation order.

**Request**:

```json
{
  "id": "song_1"
}
```

**Response**:

```json
{
  "status": "success",
  "data": {
    "id": "song_1",
    "title": "Amazing Grace",
    "author": "John Newton",
    "createdAt": "2023-10-01T12:00:00Z",
    "updatedAt": "2023-10-01T12:00:00Z",
    "sections": [
      {
        "id": "section_1",
        "label": "Verse 1",
        "text": "Amazing grace, how sweet the sound",
        "position": 1
      }
    ]
  }
}
```

**Errors**:

- 500: Database error reading the song

#### [IPC] save_song

**Description**: Creates a song, or replaces one when `id` is supplied. Sections are normalised: they are trimmed, blank sections are dropped, and a song with no words is rejected.

**Request**:

```json
{
  "request": {
    "id": null,
    "song": {
      "title": "Amazing Grace",
      "author": "John Newton",
      "sections": [
        { "label": "Verse 1", "text": "Amazing grace, how sweet the sound" },
        { "label": "Chorus", "text": "My chains are gone" }
      ]
    }
  }
}
```

**Response**:

```json
{
  "status": "success",
  "data": {
    "id": "song_1",
    "title": "Amazing Grace",
    "author": "John Newton",
    "createdAt": "2023-10-01T12:00:00Z",
    "updatedAt": "2023-10-01T12:00:00Z",
    "sections": []
  }
}
```

**Errors**:

- 400: The song has no title, or no section contains words
- 400: The song to replace no longer exists
- 500: Database error saving the song

#### [IPC] delete_song

**Description**: Removes a song and its sections from the library.

**Request**:

```json
{
  "id": "song_1"
}
```

**Response**:

```json
{
  "status": "success",
  "data": null
}
```

**Errors**:

- 500: Database error deleting the song

#### [IPC] project_song

**Description**: Shows a song on the projector, starting from a section. Every section becomes its own projectable item and the rest are queued, so `show_next_item` / `show_previous_item` step through the song section by section.

**Request**:

```json
{
  "request": {
    "songId": "song_1",
    "section": 1
  }
}
```

**Response**:

```json
{
  "status": "success",
  "data": {
    "current": {
      "id": "9ab2…",
      "contentType": "song",
      "title": "Amazing Grace — Verse 1",
      "payload": {
        "kind": "song",
        "title": "Amazing Grace",
        "label": "Verse 1",
        "text": "Amazing grace, how sweet the sound",
        "index": 1,
        "total": 2
      }
    },
    "queue": [],
    "history": []
  }
}
```

**Errors**:

- 400: The song does not exist, or has no sections yet
- 500: Presentation window could not be opened

#### [IPC] get_settings

**Description**: Retrieves the entire application settings document.

**Request**:

```json
{}
```

**Response**:

```json
{
  "status": "success",
  "data": {
    "general": {
      "appName": "Selah",
      "theme": "dark"
    },
    "audio": {
      "sampleRate": 0
    },
    "speech": {
      "recognizer": "mock",
      "threads": 4,
      "vadEnabled": true,
      "speechSampleRate": 16000
    },
    "presentation": {
      "fullscreen": true,
      "background": "#000000",
      "fontSize": 64,
      "fontFamily": "Creato Display",
      "followLive": true,
      "repeatVideos": true
    },
    "media": {
      "directory": null,
      "recursive": false
    }
  }
}
```

**Errors**:

- 500: Database error fetching settings

#### [IPC] update_settings

**Description**: Updates the application settings document.

**Request**:

```json
{
  "request": {
    "settings": {
      "general": {
        "appName": "Selah",
        "theme": "dark"
      },
      "audio": {
        "sampleRate": 0
      },
      "speech": {
        "recognizer": "mock",
        "threads": 4,
        "vadEnabled": true,
        "speechSampleRate": 16000
      },
      "presentation": {
        "fullscreen": true,
        "background": "#000000",
        "fontSize": 64,
        "fontFamily": "Creato Display",
        "followLive": true,
        "repeatVideos": true
      },
      "media": {
        "directory": null,
        "recursive": false
      }
    }
  }
}
```

**Response**:

```json
{
  "status": "success",
  "data": null
}
```

**Errors**:

- 500: Database error updating settings

#### [IPC] get_setup_state

**Description**: Returns the first-run configuration state of the application.

**Request**:

```json
{}
```

**Response**:

```json
{
  "status": "success",
  "data": {
    "completed": true,
    "hasTranslations": true,
    "hasModel": false
  }
}
```

**Errors**:

- 500: Database error reading setup state

#### [IPC] complete_setup

**Description**: Marks the first-run setup sequence as completed.

**Request**:

```json
{}
```

**Response**:

```json
{
  "status": "success",
  "data": null
}
```

**Errors**:

- 500: Database error writing setup state

## Contributing

Contributions are welcome. Please ensure your changes maintain the application's offline-first architecture. Do not introduce any cloud dependencies, external trackers, or telemetry into the codebase.

## Author Info

- LinkedIn: https://linkedin.com/in/thevalidcode
- X: https://x.com/thevalidcode

## Badges

[![TypeScript](https://img.shields.io/badge/TypeScript-3178C6?style=for-the-badge&logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![React](https://img.shields.io/badge/React-20232A?style=for-the-badge&logo=react&logoColor=61DAFB)](https://reactjs.org/)
[![Tailwind CSS](https://img.shields.io/badge/Tailwind_CSS-38B2AC?style=for-the-badge&logo=tailwind-css&logoColor=white)](https://tailwindcss.com/)
[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![SQLite](https://img.shields.io/badge/SQLite-07405E?style=for-the-badge&logo=sqlite&logoColor=white)](https://www.sqlite.org/)

[![Readme was generated by Dokugen](https://img.shields.io/badge/Readme%20was%20generated%20by-Dokugen-brightgreen)](https://dokugen.samueltuoyo.com)
