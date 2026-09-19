-- Which translations belong to Selah, and what the projector is playing.
--
-- `builtin` marks the translations Selah ships (WEB, KJV, ASV). They are the
-- ones the application can always read, so they may not be renamed or removed.
-- Every translation the operator adds — from the published catalogue or by
-- hand — stays editable and deletable.

ALTER TABLE translations ADD COLUMN builtin INTEGER NOT NULL DEFAULT 0;

-- Databases written before this migration already hold the bundled three.
UPDATE translations SET builtin = 1 WHERE id IN ('web', 'kjv', 'asv');

-- Which translation a translation came from, for the interface's benefit:
--   'bundled'   — shipped with Selah (read-only)
--   'catalogue' — added from the published list; verses added by the operator
--   'operator'  — added by hand, or imported from a file the operator owns
ALTER TABLE translations ADD COLUMN origin TEXT NOT NULL DEFAULT 'operator';

UPDATE translations SET origin = 'bundled' WHERE builtin = 1;

-- Remembers what the projector is playing, so the Media screen can show
-- "playing" and offer play/pause/stop without having to guess.
CREATE TABLE IF NOT EXISTS media_playback (
    id           INTEGER PRIMARY KEY CHECK (id = 1),
    item_id      TEXT,
    playing      INTEGER NOT NULL DEFAULT 0,
    position_ms  INTEGER NOT NULL DEFAULT 0,
    duration_ms  INTEGER NOT NULL DEFAULT 0,
    ended        INTEGER NOT NULL DEFAULT 0,
    updated_at   TEXT NOT NULL
);

INSERT OR IGNORE INTO media_playback (id, playing, updated_at)
VALUES (1, 0, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'));
