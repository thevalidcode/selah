# whisper.cpp speech models

Place `ggml-*.bin` model files here when running Selah from the repository
(checkout-time convenience only — at runtime Selah reads models from the
application data directory).

Selah **never downloads models automatically**. Get them from the official
whisper.cpp distribution and copy the file in:

```bash
cp ~/Downloads/ggml-base.en.bin models/whisper/
```

Then set the path in **Settings → Microphone → Speech recognizer / model path**.
Prefer a **full absolute path** — a relative path is resolved against the
process working directory, so it can break when the app is launched differently.

## Making speech-to-text actually work

All three of these are required. Missing any one of them means Selah hears the
room but produces no words:

| Step | What to do                                             | If it is missing                                  |
| ---- | ------------------------------------------------------ | ------------------------------------------------- |
| 1    | `brew install cmake`                                   | The `whisper` feature cannot compile at all.      |
| 2    | Build with `--features whisper`                        | Logs *"whisper … feature is disabled; falling back to mock"*. |
| 3    | Set **Speech recognizer** to *whisper* + a valid path   | Live screen shows *"Voice model ready: no"*.      |

Selah reports its real state on the **Live** screen (*Listening for words* /
*Voice model ready*), so you never have to guess which step is missing.

The recognizer is compiled only with the `whisper` cargo feature:

```bash
pnpm tauri dev -- --features whisper   # requires cmake + a C++ toolchain
```

Without that feature Selah still runs the real microphone → VAD pipeline and
uses the clearly-marked development recognizer, which returns no text.

## Choosing a model

| Model           | Size    | Notes                                         |
| --------------- | ------- | --------------------------------------------- |
| `ggml-tiny.en`  | ~75 MB  | Fastest, lowest accuracy.                     |
| `ggml-base.en`  | ~142 MB | Good balance for live service speech.         |
| `ggml-small.en` | ~466 MB | Slower; noticeably better on accented speech. |

Quantized variants (for example `ggml-base.en-q5_1.bin`, ~57 MB) are smaller and
faster on CPU-only machines at a small accuracy cost.
