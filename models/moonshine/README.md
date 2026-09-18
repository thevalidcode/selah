# Moonshine speech-to-text models

Selah never downloads a speech model. You supply a folder; Selah reads it.

## What Selah looks for

```
models/moonshine/
  encoder_model_quantized.onnx     ← or encoder_model_int8.onnx / encoder_model.onnx
  decoder_model_quantized.onnx     ← or decoder_model_int8.onnx / decoder_model.onnx
  tokenizer.json
  decoder_with_past_model_*.onnx   ← optional, only used for cached decoding
```

Any of these filename styles work, and the quantised file is preferred when
both are present:

| Slot | Accepted names (in order of preference) |
|------|------------------------------------------|
| Encoder | `encoder_model_quantized.onnx`, `encoder_model_int8.onnx`, `encoder_model.onnx` |
| Decoder | `decoder_model_quantized.onnx`, `decoder_model_int8.onnx`, `decoder_model.onnx` |

At startup the model is loaded from `models/moonshine` inside the application
data folder. On macOS that is:

```
~/Library/Application Support/app.selah.desktop/models/moonshine/
```

You can point Selah somewhere else with the **Voice model folder** setting.

## ⚠️ "int8" files are not all runnable on a CPU

This catches almost everyone, so it is worth stating plainly.

There are two different ways ONNX models get quantised to 8-bit, and they are
not interchangeable:

| Quantisation style | Ops in the graph | Runs on ONNX Runtime **CPU**? |
|---|---|---|
| **Dynamic quantisation** | `ConvInteger` | ❌ **No.** The CPU execution provider only implements `ConvInteger` for **uint8**, and these exports are **int8**. This style targets mobile NPUs (QNN / NNAPI). |
| **QDQ / `_quantized`** | `QuantizeLinear` + `DequantizeLinear` + `MatMulInteger` | ✅ Yes |
| **FP32** | plain `Conv` / `MatMul` | ✅ Yes |

If you load a dynamic-quantised int8 export you will get an error like:

```
Could not find an implementation for ConvInteger(10) node
```

That message means "this file cannot run on a CPU", not "Selah is broken".
Replace the encoder (the decoder is usually fine, because it contains no
convolutions) with the `_quantized` or unquantised file from the same model.

## ⚠️ The encoder and decoder must come from the same export

The two graphs are not interchangeable between releases. Moonshine's hidden
size differs by export even for "the same" model — for example the
`onnx-community` **tiny** export uses 288, while other tiny exports pad the
hidden size to 320 for 8-bit speed. A mismatch fails at run time with:

```
Got invalid dimensions for input: encoder_hidden_states
index: 2 Got: 288 Expected: 320
```

So always take every `.onnx` file from a single release. Check with:

```bash
python3 - <<'PY'
import onnx, glob
for f in sorted(glob.glob('*.onnx')):
    m = onnx.load(f, load_external_data=False)
    print(f, [tuple(d.dim_value or d.dim_param for d in v.type.tensor_type.shape.dim)
              for v in m.graph.output][:1])
PY
```

The encoder's last output dimension must equal the decoder's
`encoder_hidden_states` dimension.

## Where to get a model

[`onnx-community/moonshine-tiny-ONNX`](https://huggingface.co/onnx-community/moonshine-tiny-ONNX)
is the export Selah is tested against. With Hugging Face reachable, three
files are enough:

```bash
BASE=https://huggingface.co/onnx-community/moonshine-tiny-ONNX/resolve/main
DEST="$HOME/Library/Application Support/app.selah.desktop/models/moonshine"
mkdir -p "$DEST"
curl -L -o "$DEST/encoder_model_quantized.onnx" "$BASE/onnx/encoder_model_quantized.onnx"
curl -L -o "$DEST/decoder_model_quantized.onnx" "$BASE/onnx/decoder_model_quantized.onnx"
curl -L -o "$DEST/tokenizer.json"                "$BASE/tokenizer.json"
```

Moonshine `tiny` is the right starting point for CPU-only machines. `base` is
more accurate and about three times slower.

## ONNX Runtime itself

Selah loads ONNX Runtime at start-up rather than linking it. It looks in, in
order:

1. `ORT_DYLIB_PATH`, if set
2. the folder holding the Selah executable
3. the application's resource folder (bundled builds)
4. `/opt/homebrew/opt/onnxruntime/lib`, `/usr/local/opt/onnxruntime/lib`,
   `/opt/homebrew/lib`, `/usr/local/lib`

`brew install onnxruntime` may try to build from source on some machines; the
official release bundle and the `onnxruntime` Python wheel both contain a
`libonnxruntime.dylib` that works. Copy it to `/usr/local/lib`, or set
`ORT_DYLIB_PATH` to point at it.

## Checking a model folder

```bash
grep -c ConvInteger "$DEST/encoder_model_quantized.onnx"   # expect 0
```
