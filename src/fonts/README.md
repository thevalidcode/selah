# Creato Display

Selah's brand typeface. Self-hosted so the application never makes a network
request for a font — Selah must work on a machine with no internet connection.

## Files

| File                        | Weight | Used for                                 |
| --------------------------- | ------ | ---------------------------------------- |
| `CreatoDisplay-Light.woff`  | 300    | Large projected Scripture text           |
| `CreatoDisplay-Regular.woff`| 400    | Body copy, descriptions                  |
| `CreatoDisplay-Medium.woff` | 500    | Buttons, nav items, field values         |
| `CreatoDisplay-Bold.woff`   | 700    | Card titles, emphasis                    |
| `CreatoDisplay-Black.woff`  | 900    | The `SELAH` wordmark                     |

Italic files are intentionally omitted: nothing in the interface uses italics
and the extra weights would add ~50 KB to the bundle for no benefit.

## License

Licensed under the **SIL Open Font License, Version 1.1** — free for commercial
use, and redistribution/embedding in an application is permitted.

- Copyright (c) 2021 by Anugrah Pasau, with Reserved Font Name "Creato Display".
- Full license text: [`OFL.txt`](./OFL.txt).
- Upstream: <https://www.dafont.com/creato-display.font> (Lafontype).

The reserved font name means the font files must **not** be renamed or modified,
and any derivative work may not use the name "Creato Display".

## Adding a weight

Download the `.woff`/`.otf` from upstream, drop it in this folder, then add a
matching `@font-face` block in `src/index.css`. Vite fingerprints and copies
the file automatically.
