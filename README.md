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
- **Media Folders**: Selah never hardcodes a media path. The operator picks a folder at runtime, Selah reads the pictures, videos and sound files inside it, and only that folder is readable by the projector window.
- **Projector Look**: Background colour, text size and typeface are set in Settings and applied to the congregation's screen when Save is pressed — an already-open projector window updates immediately.

## Design System

### Typeface

Selah uses **Creato Display** throughout the operator interface, and lets the
*projected* text use any of five bundled typefaces — so the words on the wall can
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

| Token          | Value     | Role                                     |
| -------------- | --------- | ---------------------------------------- |
| Logo gold      | `#C9A24B` | Brand mark, active navigation, buttons   |
| Logo navy      | `#141B2E` | Interface base, logo tile, icon          |

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
*On*, and point **Voice model folder** at that folder.

> **Careful with "int8" files.** A dynamically-quantised `int8` export contains
> `ConvInteger` nodes, which ONNX Runtime's CPU provider cannot execute (it
> supports only the uint8 form). Selah then reports
> `Could not find an implementation for ConvInteger(10) node`. Use the
> `_quantized` (QDQ) or unquantised files from the same model instead —
> `models/moonshine/README.md` explains which is which.

If the model or the runtime is missing, Selah still runs — it just reports
*"Voice model ready: no"* on the Live screen and returns no words.

## Usage

When you first launch the application, you will be greeted by the setup screen. Follow the prompts to configure your environment.

1.  **Select a Microphone**: Choose the input device you want Selah to listen to.
2.  **Import a Bible Translation**: Navigate to Settings, select Database, and import a translation JSON file.
3.  **Choose a Display**: Select the monitor where the presentation window will appear.
4.  **Start Listening**: Go to the Live screen and click the Listen button to activate the audio pipeline.

Detected scripture will appear in the review panel. Click Display to send the content to the presentation window.

### A round trip you can demo

1. **Songs** → write a title and two sections (label + words) → **Save** → **Show on screen**. Each section is one screen; **Next** / **Back** step through the song verse by verse.
2. **Media** → **Choose folder…** → walk to the folder with your pictures and videos → **Read this folder** → **Show** on any file. Images and video take over the projector window; the folder you picked is the only one Selah may read.
3. **Settings → Screen** → change the background colour, drag the text size, pick a typeface (each option is drawn in its own font) → **Save settings**. An already-open projector window updates immediately, and the "How it will look" panel previews the result.
4. **Presentations** → add a heading and some words → **Show on screen**: the heading you typed appears above the words, and the item list shows it instead of raw JSON.

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

**Description**: Performs a full-text search across the Bible translation index.

**Request**:

```json
{
  "translationId": "web",
  "query": "love",
  "limit": 25
}
```

**Response**:

```json
{
  "status": "success",
  "data": [
    {
      "translationId": "web",
      "bookId": 43,
      "chapter": 3,
      "verse": 16,
      "text": "For God so loved the world..."
    }
  ]
}
```

**Errors**:

- 500: Database error during search

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

**Description**: Sends a Bible passage to the presentation display.

**Request**:

```json
{
  "passage": {
    "translationId": "web",
    "reference": "John 3:16",
    "verses": [],
    "text": "For God so loved the world..."
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
        "text": "For God so loved the world..."
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

**Description**: Lists the metadata of every media file Selah has read. Files themselves are never copied — SQLite stores metadata only.

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
      "kind": "image",
      "name": "sermon-slide.png",
      "path": "/Users/operator/Pictures/sermon-slide.png",
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

**Description**: Reads a folder and registers the pictures, videos and sound files inside it. The folder is remembered in settings and opened to the projector window, so only the folder the operator chose is readable.

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
    "items": []
  }
}
```

**Errors**:

- 400: The folder does not exist or cannot be read
- 500: Database error registering media

#### [IPC] project_media

**Description**: Puts a media file on the congregation's screen. The presentation window loads the file through Tauri's `asset:` protocol, because a webview cannot read a plain filesystem path.

**Request**:

```json
{
  "request": {
    "path": "/Users/operator/Pictures/baptism.jpg",
    "title": "Baptism"
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
      "contentType": "image",
      "title": "Baptism",
      "payload": {
        "kind": "media",
        "path": "/Users/operator/Pictures/baptism.jpg",
        "mediaKind": "image",
        "heading": "Baptism"
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

#### [IPC] import_media

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
      "followLive": true
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
        "followLive": true
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
