//! Deterministic Scripture parsing, exercised through the public API.
//!
//! These are the acceptance cases from the specification: the parser must
//! resolve real spoken references and must never invent one from unrelated
//! numbers.

use selah_lib::models::content::{ContentDetector, DetectedContent};
use selah_lib::scripture::books::find_book;
use selah_lib::scripture::normalizer::words_to_number;
use selah_lib::scripture::reference::ScriptureReference;
use selah_lib::scripture::{find_references, parse_reference};

fn expect(text: &str, expected: ScriptureReference) {
    let parsed = parse_reference(text)
        .unwrap_or_else(|| panic!("expected a reference for {text:?}, got none"));
    assert_eq!(parsed, expected, "input {text:?}");
    assert_eq!(parsed.display(), expected.display());
}

#[test]
fn resolves_typed_references() {
    expect("John 3:16", ScriptureReference::new(43, 3).with_verse(16));
    expect(
        "Romans 8:28-30",
        ScriptureReference::new(45, 8).with_range(28, 30),
    );
    expect(
        "1 Corinthians 13:4-7",
        ScriptureReference::new(46, 13).with_range(4, 7),
    );
    expect("Psalm 23", ScriptureReference::new(19, 23));
}

#[test]
fn resolves_spoken_references_in_flowing_speech() {
    expect(
        "and today we are going to read John chapter three verse sixteen",
        ScriptureReference::new(43, 3).with_verse(16),
    );
    expect(
        "let's read Romans chapter eight verse twenty eight",
        ScriptureReference::new(45, 8).with_verse(28),
    );
    expect(
        "please turn to first Corinthians chapter thirteen verses four to seven",
        ScriptureReference::new(46, 13).with_range(4, 7),
    );
    expect(
        "Psalm chapter ninety one verse one",
        ScriptureReference::new(19, 91).with_verse(1),
    );
}

#[test]
fn number_words_convert_to_numbers() {
    assert_eq!(
        words_to_number(&["twenty", "eight"], 0).map(|(n, _)| n),
        Some(28)
    );
    assert_eq!(
        words_to_number(&["ninety", "one"], 0).map(|(n, _)| n),
        Some(91)
    );
    assert_eq!(
        words_to_number(&["one", "hundred", "nineteen"], 0).map(|(n, _)| n),
        Some(119)
    );
    assert_eq!(words_to_number(&["sixteen"], 0).map(|(n, _)| n), Some(16));
    assert!(words_to_number(&["chapter"], 0).is_none());
}

#[test]
fn book_registry_normalizes_spoken_variants() {
    let cases: &[(&[&str], &str)] = &[
        (&["john"], "John"),
        (&["first", "corinthians"], "1 Corinthians"),
        (&["i", "corinthians"], "1 Corinthians"),
        (&["second", "corinthians"], "2 Corinthians"),
        (&["psalm"], "Psalms"),
        (&["revelations"], "Revelation"),
    ];

    for (tokens, expected_name) in cases {
        let owned: Vec<String> = tokens.iter().map(|t| t.to_string()).collect();
        let refs: Vec<&str> = owned.iter().map(|s| s.as_str()).collect();
        let found = find_book(&refs, 0)
            .unwrap_or_else(|| panic!("no book matched {tokens:?}"))
            .0;
        assert_eq!(found.name, *expected_name, "tokens {tokens:?}");
    }
}

#[test]
fn leading_filler_words_are_skipped() {
    // "the book of John" — the book is not at the start of the transcript.
    expect(
        "the book of John chapter three verse sixteen",
        ScriptureReference::new(43, 3).with_verse(16),
    );
    expect(
        "let's open John's gospel chapter one verse one",
        ScriptureReference::new(43, 1).with_verse(1),
    );
}

#[test]
fn ignores_text_without_a_recognizable_book() {
    for text in [
        "I arrived at 3pm",
        "we have twenty people",
        "chapter three was difficult",
        "there are 4 cats and 2 dogs",
        "the verse was interesting",
        "John and Mary came to church",
    ] {
        assert!(
            parse_reference(text).is_none(),
            "must not detect a reference in {text:?}"
        );
    }
}

#[test]
fn rejects_references_outside_the_canon() {
    assert!(parse_reference("John 45:1").is_none());
    assert!(parse_reference("Psalm 151").is_none());
}

#[test]
fn finds_multiple_references_in_one_transcript() {
    let refs = find_references("read John 3:16 and then Psalm 23");
    assert_eq!(refs.len(), 2);
    assert_eq!(refs[0], ScriptureReference::new(43, 3).with_verse(16));
    assert_eq!(refs[1], ScriptureReference::new(19, 23));
}

#[test]
fn content_detector_reports_scripture_with_confidence() {
    let results = ContentDetector.detect("turn with me to John 3:16");
    assert_eq!(results.len(), 1);
    let detection = &results[0];
    assert!(detection.content.is_scripture());
    assert!(detection.confidence.unwrap_or(0.0) > 0.9);
    assert!(matches!(
        detection.content,
        DetectedContent::Scripture { .. }
    ));
}

#[test]
fn content_detector_falls_back_to_text() {
    let results = ContentDetector.detect("remember the offering plates");
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0].content, DetectedContent::Text { .. }));
    assert!(ContentDetector.detect("   ").is_empty());
}
