-- Songs library + media folder support.
--
-- Songs live in the library alongside Bible text, and are stored as ordered
-- sections so a song can be presented one verse/chorus at a time instead of a
-- single wall of words.

CREATE TABLE songs (
    id         TEXT PRIMARY KEY,
    title      TEXT NOT NULL,
    author     TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE song_sections (
    id       TEXT PRIMARY KEY,
    song_id  TEXT NOT NULL,
    position INTEGER NOT NULL,
    label    TEXT,
    text     TEXT NOT NULL,

    FOREIGN KEY (song_id) REFERENCES songs (id)
        ON UPDATE CASCADE ON DELETE CASCADE
);

CREATE INDEX idx_song_sections_order
    ON song_sections (song_id, position);

-- ---------------------------------------------------------------------------
-- Media files are read from a folder the operator picks at runtime, so the
-- same file can be re-read without creating duplicate library rows. Clear out
-- any duplicate paths recorded by an earlier build first, otherwise the unique
-- index could not be created on an existing database.
-- ---------------------------------------------------------------------------

DELETE FROM media
 WHERE rowid NOT IN (SELECT MIN(rowid) FROM media GROUP BY path);

CREATE UNIQUE INDEX idx_media_path ON media (path);
