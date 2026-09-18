# whisper.cpp speech models

Place `ggml-*.bin` model files here when running Selah from the repository
(checkout-time convenience only — at runtime Selah reads models from the
application data directory).

Selah **never downloads models automatically**. Get them from the official
whisper.cpp distribution and copy the file in:

```bash
cp ~/Downloads/ggml-base.en.bin models/whisper/
```

Then set the path in **Settings → Speech → Whisper model path**.

Recommended for an Intel Mac (CPU-only inference):

| Model           | Size    | Notes                                         |
| --------------- | ------- | --------------------------------------------- |
| `ggml-tiny.en`  | ~75 MB  | Fastest, lowest accuracy.                     |
| `ggml-base.en`  | ~142 MB | Good balance for live service speech.         |
| `ggml-small.en` | ~466 MB | Slower; noticeably better on accented speech. |

The recognizer is compiled only with the `whisper` cargo feature:

```bash
pnpm tauri dev -- --features whisper   # requires cmake + a C++ toolchain
```

Without that feature Selah still runs the real microphone → VAD pipeline and
uses the clearly-marked development recognizer, which returns no text.
