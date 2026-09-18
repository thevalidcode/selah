-- Fix the verse search-index triggers.
--
-- Migration 0001 built `verses_fts` as an ordinary FTS5 table but wrote the
-- delete/update triggers using FTS5's external-content "delete" command:
--
--     INSERT INTO verses_fts (verses_fts, translation_id, rowid, text)
--     VALUES ('delete', old.translation_id, old.rowid, old.text);
--
-- That command is only valid for a contentless or external-content table.
-- Against an ordinary FTS5 table SQLite raises "SQL logic error", so *any*
-- update to a verse — re-importing a corrected translation, for example —
-- failed. Deletes and updates are rewritten below to operate on the index
-- rowid directly, which is the correct form here.
--
-- Triggers hold no data, so dropping and recreating them is safe.

DROP TRIGGER IF EXISTS verses_ad;
DROP TRIGGER IF EXISTS verses_au;

CREATE TRIGGER verses_ad AFTER DELETE ON verses BEGIN
    DELETE FROM verses_fts WHERE rowid = old.rowid;
END;

CREATE TRIGGER verses_au AFTER UPDATE ON verses BEGIN
    DELETE FROM verses_fts WHERE rowid = old.rowid;
    INSERT INTO verses_fts (translation_id, text)
    VALUES (new.translation_id, new.text);
END;
