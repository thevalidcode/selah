-- Initial Selah schema.
-- Applied atomically inside a transaction by db/migrations.rs.

CREATE TABLE translations (
    id           TEXT PRIMARY KEY,
    name         TEXT NOT NULL,
    language     TEXT NOT NULL,
    abbreviation TEXT,
    is_default   INTEGER NOT NULL DEFAULT 0,
    created_at   TEXT NOT NULL
);

CREATE TABLE books (
    id           INTEGER PRIMARY KEY,
    name         TEXT NOT NULL,
    testament    TEXT NOT NULL,
    abbreviation TEXT
);

CREATE TABLE verses (
    translation_id TEXT NOT NULL,
    book_id        INTEGER NOT NULL,
    chapter        INTEGER NOT NULL,
    verse          INTEGER NOT NULL,
    text           TEXT NOT NULL,

    PRIMARY KEY (translation_id, book_id, chapter, verse),

    FOREIGN KEY (translation_id) REFERENCES translations (id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    FOREIGN KEY (book_id) REFERENCES books (id)
);

-- Fast path for `get_passage` range queries and `search`.
CREATE INDEX idx_verses_book_range
    ON verses (translation_id, book_id, chapter, verse);

CREATE VIRTUAL TABLE IF NOT EXISTS verses_fts USING fts5 (
    translation_id UNINDEXED,
    text,
    tokenize = 'porter unicode61'
);

-- Keep the FTS index in sync with manual verse writes.
CREATE TRIGGER verses_ai AFTER INSERT ON verses BEGIN
    INSERT INTO verses_fts (translation_id, text)
    VALUES (new.translation_id, new.text);
END;

CREATE TRIGGER verses_ad AFTER DELETE ON verses BEGIN
    INSERT INTO verses_fts (verses_fts, translation_id, rowid, text)
    VALUES ('delete', old.translation_id, old.rowid, old.text);
END;

CREATE TRIGGER verses_au AFTER UPDATE ON verses BEGIN
    INSERT INTO verses_fts (verses_fts, translation_id, rowid, text)
    VALUES ('delete', old.translation_id, old.rowid, old.text);
    INSERT INTO verses_fts (translation_id, text)
    VALUES (new.translation_id, new.text);
END;

CREATE TABLE settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE media (
    id         TEXT PRIMARY KEY,
    type       TEXT NOT NULL,
    name       TEXT NOT NULL,
    path       TEXT NOT NULL,
    metadata   TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE presentations (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE presentation_items (
    id              TEXT PRIMARY KEY,
    presentation_id TEXT NOT NULL,
    type            TEXT NOT NULL,
    position        INTEGER NOT NULL,
    payload         TEXT NOT NULL,

    FOREIGN KEY (presentation_id) REFERENCES presentations (id)
        ON DELETE CASCADE
);

CREATE INDEX idx_presentation_items_order
    ON presentation_items (presentation_id, position);

-- ---------------------------------------------------------------------------
-- Canonical books (protestant canon of 66). IDs are stable and shared by the
-- deterministic scripture registry in `scripture::books`.
-- ---------------------------------------------------------------------------
INSERT INTO books (id, name, testament, abbreviation) VALUES
  (1,  'Genesis',            'old', 'Gen'),
  (2,  'Exodus',             'old', 'Exo'),
  (3,  'Leviticus',          'old', 'Lev'),
  (4,  'Numbers',            'old', 'Num'),
  (5,  'Deuteronomy',        'old', 'Deu'),
  (6,  'Joshua',             'old', 'Jos'),
  (7,  'Judges',             'old', 'Jdg'),
  (8,  'Ruth',               'old', 'Rut'),
  (9,  '1 Samuel',           'old', '1Sa'),
  (10, '2 Samuel',           'old', '2Sa'),
  (11, '1 Kings',            'old', '1Ki'),
  (12, '2 Kings',            'old', '2Ki'),
  (13, '1 Chronicles',       'old', '1Ch'),
  (14, '2 Chronicles',       'old', '2Ch'),
  (15, 'Ezra',               'old', 'Ezr'),
  (16, 'Nehemiah',           'old', 'Neh'),
  (17, 'Esther',             'old', 'Est'),
  (18, 'Job',                'old', 'Job'),
  (19, 'Psalms',             'old', 'Psa'),
  (20, 'Proverbs',           'old', 'Pro'),
  (21, 'Ecclesiastes',       'old', 'Ecc'),
  (22, 'Song of Solomon',    'old', 'Sng'),
  (23, 'Isaiah',             'old', 'Isa'),
  (24, 'Jeremiah',           'old', 'Jer'),
  (25, 'Lamentations',       'old', 'Lam'),
  (26, 'Ezekiel',            'old', 'Ezk'),
  (27, 'Daniel',             'old', 'Dan'),
  (28, 'Hosea',              'old', 'Hos'),
  (29, 'Joel',               'old', 'Jol'),
  (30, 'Amos',               'old', 'Amo'),
  (31, 'Obadiah',            'old', 'Oba'),
  (32, 'Jonah',              'old', 'Jon'),
  (33, 'Micah',              'old', 'Mic'),
  (34, 'Nahum',              'old', 'Nam'),
  (35, 'Habakkuk',           'old', 'Hab'),
  (36, 'Zephaniah',          'old', 'Zep'),
  (37, 'Haggai',             'old', 'Hag'),
  (38, 'Zechariah',          'old', 'Zec'),
  (39, 'Malachi',            'old', 'Mal'),
  (40, 'Matthew',            'new', 'Mat'),
  (41, 'Mark',               'new', 'Mar'),
  (42, 'Luke',               'new', 'Luk'),
  (43, 'John',               'new', 'Jhn'),
  (44, 'Acts',               'new', 'Act'),
  (45, 'Romans',             'new', 'Rom'),
  (46, '1 Corinthians',      'new', '1Co'),
  (47, '2 Corinthians',      'new', '2Co'),
  (48, 'Galatians',          'new', 'Gal'),
  (49, 'Ephesians',          'new', 'Eph'),
  (50, 'Philippians',        'new', 'Php'),
  (51, 'Colossians',         'new', 'Col'),
  (52, '1 Thessalonians',    'new', '1Th'),
  (53, '2 Thessalonians',    'new', '2Th'),
  (54, '1 Timothy',          'new', '1Ti'),
  (55, '2 Timothy',          'new', '2Ti'),
  (56, 'Titus',              'new', 'Tit'),
  (57, 'Philemon',           'new', 'Phm'),
  (58, 'Hebrews',            'new', 'Heb'),
  (59, 'James',              'new', 'Jas'),
  (60, '1 Peter',            'new', '1Pe'),
  (61, '2 Peter',            'new', '2Pe'),
  (62, '1 John',             'new', '1Jn'),
  (63, '2 John',             'new', '2Jn'),
  (64, '3 John',             'new', '3Jn'),
  (65, 'Jude',               'new', 'Jud'),
  (66, 'Revelation',         'new', 'Rev');