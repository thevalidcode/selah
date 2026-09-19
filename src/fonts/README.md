# Fonts

Selah's projected text can use any of the typefaces below. They are all
self-hosted and bundled by Vite, because Selah must work on a machine with no
internet connection — it never fetches a font over the network.

## Creato Display (Selah's own typeface)

| File                        | Weight | Used for                                 |
| --------------------------- | ------ | ---------------------------------------- |
| `CreatoDisplay-Light.woff`  | 300    | Large projected Scripture text           |
| `CreatoDisplay-Regular.woff`| 400    | Body copy, descriptions                  |
| `CreatoDisplay-Medium.woff` | 500    | Buttons, nav items, field values         |
| `CreatoDisplay-Bold.woff`   | 700    | Card titles, emphasis                    |
| `CreatoDisplay-Black.woff`  | 900    | The `SELAH` wordmark                     |

Italic files are intentionally omitted: nothing in the interface uses italics
and the extra weights would add ~50 KB to the bundle for no benefit.

**License**: SIL Open Font License 1.1 — free for commercial use, and
redistribution/embedding in an application is permitted. Full text in
[`OFL.txt`](./OFL.txt). Copyright (c) 2021 by Anugrah Pasau, with Reserved Font
Name "Creato Display". The reserved font name means the files must **not** be
renamed or modified, and any derivative work may not use the name "Creato
Display". Upstream: <https://www.dafont.com/creato-display.font> (Lafontype).

## Extra projector typefaces

These are the choices offered by the **typeface picker** in
Settings → Screen. All are variable fonts carrying the full weight range in one
latin-subset file, so a whole family costs 28–48 KB.

| File                          | Family         | Good for                                   |
| ----------------------------- | -------------- | ------------------------------------------ |
| `Inter-Variable.woff2`        | Inter          | Plain, very legible from a distance        |
| `Lora-Variable.woff2`         | Lora           | Warm serif that suits Bible readings        |
| `Oswald-Variable.woff2`       | Oswald         | Tall and condensed, fits long song lines    |
| `JetBrainsMono-Variable.woff2`| JetBrains Mono | Evenly spaced, for notices and references   |

**Licenses**: all four are licensed under the SIL Open Font License 1.1. The
verbatim license text that ships with each font is kept in
[`licenses/`](./licenses):

- `licenses/inter-OFL.txt` — Copyright 2020 The Inter Project Authors.
- `licenses/lora-OFL.txt` — Copyright 2011 The Lora Project Authors, with
  Reserved Font Name "Lora".
- `licenses/oswald-OFL.txt` — Copyright 2016 The Oswald Project Authors.
- `licenses/jetbrainsmono-OFL.txt` — Copyright 2020 The JetBrains Mono Project
  Authors.

They were downloaded as the `latin` subset from the Google Fonts API; the files
are redistributed unmodified, with their names untouched.

## Adding another family

1. Drop the `.woff`/`.woff2` file in this folder (keep the upstream file name if
   the license reserves one).
2. Add a matching `@font-face` block in `src/index.css`.
3. List the family in `FONT_OPTIONS` in `src/lib/fonts.ts` — that is what the
   typeface picker reads.
4. Add the font's OFL text under `licenses/`.

Vite fingerprints and copies the file automatically, so nothing else needs
changing.
