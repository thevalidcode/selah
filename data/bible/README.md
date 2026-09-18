# Bible import format

Selah **ships no Bible text**. Copyright in modern translations belongs to their
publishers, so the application provides the schema and an import mechanism
instead of bundling a translation.

Import a translation from the **Settings → Database** screen by pointing at a
UTF-8 JSON file on disk that you have the legal right to use (for example KJV,
WEB, or another public-domain / licensed text).

## Document shape

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
