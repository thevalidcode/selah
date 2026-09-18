# Bible translations

Selah bundles **three public-domain translations** so a fresh install can show
Scripture straight away, and can import others from a file you supply.

Copyright in modern translations belongs to their publishers, so Selah only
ever ships text that is public domain. You are responsible for having the
right to use anything you import yourself.

## Bundled translations

Installed automatically on first start from `src-tauri/resources/bible/`:

| File         | Translation               | Id    |
| ------------ | ------------------------- | ----- |
| `web.sqlite` | World English Bible       | `web` |
| `kjv.sqlite` | King James Version        | `kjv` |
| `asv.sqlite` | American Standard Version | `asv` |

Each is copied into Selah's own database once. Re-running is harmless: a
translation that is already installed is skipped, so this runs safely on every
launch. `web` becomes the default translation only when nothing is installed
yet.

These files use the widely published layout:

```sql
CREATE TABLE verses (
  book_id INTEGER,   -- 1..=66, matching Selah's canonical book registry
  chapter INTEGER,
  number  INTEGER,   -- the verse number
  text    TEXT
);
```

Any other file in this shape can be added with **Import a Bible file** on the
Bible screen, or by copying it beside the bundled ones.

## Importing other translations

There are two import formats:

1. **SQLite** (`.sqlite`) — the layout above.
2. **JSON** — a flat document described below.

### JSON document shape

```json
{
  "translation": {
    "id": "web",
    "name": "World English Bible",
    "language": "en",
    "abbreviation": "WEB",
    "isDefault": true
  },
  "verses": [
    { "bookId": 43, "chapter": 3, "verse": 16, "text": "…" }
  ]
}
```

### `translation`

| Field          | Type    | Notes                                                              |
| -------------- | ------- | ------------------------------------------------------------------ |
| `id`           | string  | Required, stable identifier (primary key of `translations`).        |
| `name`         | string  | Required, human readable name shown in the UI.                      |
| `language`     | string  | Required, BCP-47-ish tag such as `en`.                              |
| `abbreviation` | string? | Optional short label, e.g. `KJV`.                                   |
| `isDefault`    | boolean | When `true`, every other translation is un-flagged as default.       |

### `verses`

A flat list. `bookId` refers to the deterministic book registry
(`src-tauri/src/scripture/books.rs`, also seeded into the `books` table):

| Id | Book             | Id | Book        | Id | Book            |
| -- | ---------------- | -- | ----------- | -- | --------------- |
| 1  | Genesis          | 23 | Isaiah      | 45 | Romans          |
| 2  | Exodus           | 24 | Jeremiah    | 46 | 1 Corinthians   |
| 19 | Psalms           | 40 | Matthew     | 66 | Revelation      |

Run the full list from the Bible screen (Book selector) or:

```sql
SELECT id, name FROM books ORDER BY id;
```

## Import behaviour

* The whole import runs in **one SQLite transaction**.
* Every `bookId` is validated against the canon **before** anything is written —
  an unknown book id aborts the import and rolls it back.
* Re-importing the same `translation.id` upserts verses (`ON CONFLICT … DO
  UPDATE`), so a corrected export can simply be imported again.
* Verse text is indexed in the `verses_fts` FTS5 table automatically, which is
  what powers the Bible screen's search box.

## Generating an import file

Any offline tooling can produce this JSON. Example shape of a small conversion
script (not shipped, illustrative only):

```python
import json, sqlite3

rows = sqlite3.connect("my-source.sqlite").execute(
    "SELECT book, chapter, verse, text FROM verses"
)
json.dump(
    {
        "translation": {
            "id": "my-translation",
            "name": "My Translation",
            "language": "en",
            "abbreviation": "MT",
            "isDefault": False,
        },
        "verses": [
            {"bookId": b, "chapter": c, "verse": v, "text": t}
            for b, c, v, t in rows
        ],
    },
    open("my-translation.json", "w", encoding="utf-8"),
    ensure_ascii=False,
)
```

## Licensing

Only import text you are allowed to use. Selah performs no validation of
licensing — that responsibility stays with the operator.
