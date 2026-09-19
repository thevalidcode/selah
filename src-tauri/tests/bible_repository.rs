//! Bible subsystem tests: repository, registry enrichment and the JSON import
//! seed mechanism. These exercise the real SQLite schema (in-memory) so the
//! migrations are covered too.

use selah_lib::bible::import::{apply_import, parse_import, ImportDocument};
use selah_lib::bible::models::{TranslationInfo, Verse};
use selah_lib::bible::repository::BibleRepository;
use selah_lib::db;
use selah_lib::errors::AppError;
use selah_lib::scripture::reference::ScriptureReference;

/// A migrated in-memory database — the same schema the app ships.
fn memory_db() -> rusqlite::Connection {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    db::migrations::run_migrations(&conn).unwrap();
    conn
}

fn sample_translation(id: &str) -> TranslationInfo {
    TranslationInfo {
        id: id.to_string(),
        name: "Test Translation".to_string(),
        language: "en".to_string(),
        abbreviation: Some("TST".to_string()),
        is_default: true,
    }
}

#[test]
fn books_carry_canonical_chapter_counts() {
    let conn = memory_db();
    let repo = BibleRepository::new(&conn);
    let books = repo.list_books().unwrap();

    assert_eq!(books.len(), 66, "protestant canon");

    let john = books.iter().find(|b| b.id == 43).unwrap();
    assert_eq!(john.name, "John");
    assert_eq!(john.chapters, 21);

    let psalms = books.iter().find(|b| b.id == 19).unwrap();
    assert_eq!(psalms.chapters, 150);

    // Old and new testament rows are both present and tagged.
    assert!(books.iter().any(|b| b.testament == "old"));
    assert!(books.iter().any(|b| b.testament == "new"));
}

#[test]
fn find_book_by_name_is_case_insensitive() {
    let conn = memory_db();
    let repo = BibleRepository::new(&conn);
    let found = repo.find_book_by_name("john").unwrap().unwrap();
    assert_eq!(found.id, 43);
    assert!(repo.find_book_by_name("Not A Book").unwrap().is_none());
}

#[test]
fn passage_round_trip_and_search() {
    let conn = memory_db();
    let repo = BibleRepository::new(&conn);
    repo.upsert_translation(&sample_translation("tst")).unwrap();

    for (verse, text) in [
        (16, "For God so loved the world"),
        (17, "For God sent not his Son to condemn the world"),
    ] {
        repo.insert_verse(&Verse {
            translation_id: "tst".into(),
            book_id: 43,
            chapter: 3,
            verse,
            text: text.into(),
        })
        .unwrap();
    }

    let passage = repo
        .get_passage("tst", &ScriptureReference::new(43, 3).with_range(16, 17))
        .unwrap();
    assert_eq!(passage.verses.len(), 2);
    assert_eq!(passage.reference, "John 3:16-17");
    assert!(passage.text.contains("God so loved"));

    // FTS-backed search finds verses across the translation.
    let hits = repo.search(Some("tst"), &fts_terms("loved"), 10).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].verse, 16);

    // `None` searches every installed translation, which is what makes a
    // half-remembered phrase findable whichever version it came from.
    let anywhere = repo.search(None, &fts_terms("loved"), 10).unwrap();
    assert_eq!(anywhere.len(), 1);
}

/// Builds the same quoted FTS5 expression the search command builds.
fn fts_terms(query: &str) -> String {
    query
        .split_whitespace()
        .map(|word| format!("\"{}\"", word.to_lowercase()))
        .collect::<Vec<_>>()
        .join(" AND ")
}

#[test]
fn the_bundled_translations_cannot_be_renamed_or_removed() {
    // Selah ships WEB, KJV and ASV. They are the Bibles the application can
    // always rely on, so the interface must not be able to rename them away.
    let conn = memory_db();
    let repo = BibleRepository::new(&conn);
    repo.upsert_translation(&sample_translation("web")).unwrap();
    repo.mark_builtin("web").unwrap();

    let stored = repo.get_translation("web").unwrap().expect("installed");
    assert!(stored.builtin);
    assert_eq!(stored.origin, "bundled");

    let renamed = repo.rename_translation("web", "My Own Bible", None, "en");
    assert!(renamed.is_err());
    assert!(renamed
        .unwrap_err()
        .to_string()
        .contains("cannot be renamed"));

    let removed = repo.delete_translation("web");
    assert!(removed.is_err());
    assert!(removed
        .unwrap_err()
        .to_string()
        .contains("cannot be removed"));

    // Nothing changed behind the refusal.
    assert_eq!(
        repo.get_translation("web").unwrap().unwrap().name,
        "Test Translation"
    );
}

#[test]
fn a_translation_the_operator_added_can_be_renamed_and_removed() {
    let conn = memory_db();
    let repo = BibleRepository::new(&conn);

    // A published translation the operator added from the catalogue.
    let info = TranslationInfo {
        id: "msg".to_string(),
        name: "The Message".to_string(),
        language: "en".to_string(),
        abbreviation: Some("MSG".to_string()),
        is_default: true,
    };
    repo.add_catalogue_translation(&info).unwrap();
    repo.save_verses(
        &info,
        &[Verse {
            translation_id: "msg".into(),
            book_id: 43,
            chapter: 3,
            verse: 16,
            text: "This is how much God loved the world".into(),
        }],
    )
    .unwrap();

    let stored = repo.get_translation("msg").unwrap().expect("installed");
    assert!(!stored.builtin);
    assert_eq!(stored.origin, "catalogue");

    repo.rename_translation("msg", "The Message (paraphrase)", Some("MSG"), "en")
        .unwrap();
    assert_eq!(
        repo.get_translation("msg").unwrap().unwrap().name,
        "The Message (paraphrase)"
    );

    // Put a second translation in place so removing the default has somewhere
    // to hand the flag to.
    repo.upsert_translation(&sample_translation("kjv")).unwrap();
    repo.delete_translation("msg").unwrap();

    assert!(repo.get_translation("msg").unwrap().is_none());
    // Its verses went with it.
    assert_eq!(repo.verse_count("msg").unwrap(), 0);
    // …and the Bible screen still has a translation to open on.
    assert_eq!(
        repo.default_translation().unwrap().map(|t| t.id),
        Some("kjv".to_string())
    );
}

#[test]
fn saving_verses_never_renames_a_translation_that_already_exists() {
    // Adding verses to an installed Bible must not overwrite the name it was
    // added under — the bug this guards against renamed "The Message" to
    // "My verses" the moment a chapter was pasted in.
    let conn = memory_db();
    let repo = BibleRepository::new(&conn);

    let info = TranslationInfo {
        id: "msg".to_string(),
        name: "The Message".to_string(),
        language: "en".to_string(),
        abbreviation: Some("MSG".to_string()),
        is_default: false,
    };
    repo.add_catalogue_translation(&info).unwrap();

    let casual = TranslationInfo {
        id: "msg".to_string(),
        name: "My verses".to_string(),
        language: "en".to_string(),
        abbreviation: None,
        is_default: true,
    };
    repo.save_verses(
        &casual,
        &[Verse {
            translation_id: "msg".into(),
            book_id: 19,
            chapter: 23,
            verse: 1,
            text: "God, my shepherd! I don't need a thing".into(),
        }],
    )
    .unwrap();

    let stored = repo.get_translation("msg").unwrap().expect("installed");
    assert_eq!(stored.name, "The Message");
    assert_eq!(stored.abbreviation.as_deref(), Some("MSG"));
    assert!(!stored.is_default, "the default flag must not move either");
    assert_eq!(repo.verse_count("msg").unwrap(), 1);
}

#[test]
fn operator_supplied_verses_are_saved_and_readable() {
    // The manual-entry path: verses typed or pasted by the operator become a
    // translation they can look up like any other.
    let conn = memory_db();
    let repo = BibleRepository::new(&conn);

    let info = TranslationInfo {
        id: "custom".to_string(),
        name: "My verses".to_string(),
        language: "en".to_string(),
        abbreviation: None,
        is_default: false,
    };
    let verses: Vec<Verse> = [
        (1, "In the beginning God created"),
        (2, "And the earth was void"),
    ]
    .into_iter()
    .map(|(verse, text)| Verse {
        translation_id: "custom".into(),
        book_id: 1,
        chapter: 1,
        verse,
        text: text.into(),
    })
    .collect();

    assert_eq!(repo.save_verses(&info, &verses).unwrap(), 2);

    let passage = repo
        .get_passage("custom", &ScriptureReference::new(1, 1).with_range(1, 2))
        .unwrap();
    assert_eq!(passage.reference, "Genesis 1:1-2");
    assert!(passage.text.contains("God created"));

    // Saved verses are searchable too, because the FTS triggers fire on insert.
    let hits = repo
        .search(Some("custom"), &fts_terms("created"), 10)
        .unwrap();
    assert_eq!(hits.len(), 1);

    // Re-saving the same reference replaces rather than duplicates.
    let updated = vec![Verse {
        translation_id: "custom".into(),
        book_id: 1,
        chapter: 1,
        verse: 1,
        text: "In the beginning God made the heavens".into(),
    }];
    repo.save_verses(&info, &updated).unwrap();
    let single = repo
        .get_passage("custom", &ScriptureReference::new(1, 1).with_verse(1))
        .unwrap();
    assert!(single.text.contains("made the heavens"));
    assert_eq!(repo.verse_count("custom").unwrap(), 2);
}

#[test]
fn import_document_validates_required_fields() {
    let dir = std::env::temp_dir().join(format!("selah-import-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();

    let missing_id = dir.join("missing-id.json");
    std::fs::write(
        &missing_id,
        r#"{"translation":{"id":"","name":"X","language":"en","abbreviation":null,"isDefault":false},"verses":[]}"#,
    )
    .unwrap();
    let err = parse_import(&missing_id).unwrap_err();
    assert!(matches!(err, AppError::InvalidConfiguration(_)));

    let not_json = dir.join("garbage.json");
    std::fs::write(&not_json, b"not json at all").unwrap();
    assert!(matches!(
        parse_import(&not_json).unwrap_err(),
        AppError::InvalidConfiguration(_)
    ));

    assert!(parse_import(&dir.join("does-not-exist.json")).is_err());

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn import_apply_is_atomic_and_rejects_unknown_books() {
    let conn = memory_db();

    let bad = ImportDocument {
        translation: sample_translation("bad"),
        verses: vec![selah_lib::bible::import::ImportVerse {
            book_id: 99, // outside the canon
            chapter: 1,
            verse: 1,
            text: "nope".into(),
        }],
    };
    let err = apply_import(&conn, &bad).unwrap_err();
    assert!(matches!(err, AppError::InvalidScriptureReference(_)));

    // The whole import was rolled back: no translation row survived.
    let repo = BibleRepository::new(&conn);
    assert!(repo.get_translation("bad").unwrap().is_none());
}

#[test]
fn import_apply_stores_verses_and_flags_default() {
    let conn = memory_db();

    let document = ImportDocument {
        translation: sample_translation("web"),
        verses: vec![
            selah_lib::bible::import::ImportVerse {
                book_id: 19,
                chapter: 23,
                verse: 1,
                text: "The Lord is my shepherd".into(),
            },
            selah_lib::bible::import::ImportVerse {
                book_id: 45,
                chapter: 8,
                verse: 28,
                text: "All things work together for good".into(),
            },
        ],
    };

    let (count, id) = apply_import(&conn, &document).unwrap();
    assert_eq!(count, 2);
    assert_eq!(id, "web");

    let repo = BibleRepository::new(&conn);
    let translation = repo.get_translation("web").unwrap().unwrap();
    assert!(translation.is_default);
    assert_eq!(repo.verse_count("web").unwrap(), 2);

    let passage = repo
        .get_passage("web", &ScriptureReference::new(19, 23).with_verse(1))
        .unwrap();
    assert_eq!(passage.reference, "Psalms 23:1");
}
