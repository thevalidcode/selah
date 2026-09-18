//! Deterministic Scripture reference parser.
//!
//! Designed to be independent of speech-transcription quality: it operates on
//! a normalized token stream and only ever emits a reference when a
//! recognizable Bible book is followed by a structurally valid chapter/verse
//! sequence. If a transcript has no recognizable book sequence it produces
//! nothing — it never invents references from random numbers.

use super::books::{find_book, BibleBook};
use super::normalizer::{is_number_word, words_to_number};
use super::reference::ScriptureReference;

/// A normalized token derived from text.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(u32),
    Word(String),
    Colon,
    Dash,
}

impl Token {
    fn text(&self) -> String {
        match self {
            Token::Number(n) => n.to_string(),
            Token::Word(w) => w.clone(),
            Token::Colon => ":".to_string(),
            Token::Dash => "-".to_string(),
        }
    }
}

/// Converts typed/spoken text into a token stream.
///
/// `:` becomes [`Token::Colon`], dashes become [`Token::Dash`], digit runs
/// become [`Token::Number`]. Trailing possessives ("john's") are stripped.
pub fn tokenize(input: &str) -> Vec<Token> {
    let normalized = super::normalizer::normalize_text(input);
    let chars: Vec<char> = normalized.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];
        if ch.is_whitespace() {
            i += 1;
            continue;
        }
        if ch.is_ascii_digit() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            let s: String = chars[start..i].iter().collect();
            if let Ok(n) = s.parse::<u32>() {
                tokens.push(Token::Number(n));
            }
            continue;
        }
        if ch.is_alphabetic() {
            let start = i;
            while i < chars.len() && (chars[i].is_alphabetic() || chars[i] == '\'') {
                i += 1;
            }
            let mut word: String = chars[start..i].iter().collect::<String>().to_lowercase();
            if word.len() > 2 && word.ends_with("'s") {
                word.truncate(word.len() - 2);
            }
            tokens.push(Token::Word(word));
            continue;
        }
        match ch {
            ':' => tokens.push(Token::Colon),
            '-' | '\u{2013}' | '\u{2014}' => tokens.push(Token::Dash),
            _ => {} // period, comma, etc. are separators
        }
        i += 1;
    }
    tokens
}

fn token_strings(tokens: &[Token]) -> Vec<String> {
    tokens.iter().map(Token::text).collect()
}

fn is_chapter_marker(t: &Token) -> bool {
    matches!(
        t,
        Token::Word(w) if matches!(w.as_str(), "chapter" | "chapters" | "ch" | "chs")
    )
}

fn is_verse_marker(t: &Token) -> bool {
    matches!(
        t,
        Token::Word(w) if matches!(w.as_str(), "verse" | "verses" | "v" | "vs")
    )
}

fn is_range_connector(t: &Token) -> bool {
    match t {
        Token::Dash | Token::Colon => true,
        Token::Word(w) => matches!(w.as_str(), "through" | "thru" | "to"),
        _ => false,
    }
}

/// Reads a number starting at `i`: either a digit token, a digit-spelled
/// word, or a compound number word sequence.
fn number_at(tokens: &[Token], i: usize) -> Option<(u32, usize)> {
    match tokens.get(i)? {
        Token::Number(n) => Some((*n, i + 1)),
        Token::Word(w) => {
            if let Ok(n) = w.parse::<u32>() {
                return Some((n, i + 1));
            }
            if !is_number_word(w) {
                return None;
            }
            let mut window: Vec<&str> = Vec::new();
            for t in tokens.iter().skip(i).take(4) {
                match t {
                    Token::Word(w) if is_number_word(w) => window.push(w),
                    _ => break,
                }
            }
            words_to_number(&window, 0).map(|(n, consumed)| (n, i + consumed))
        }
        _ => None,
    }
}

/// Attempts to parse `book` followed by a reference from `tokens[n0..]`.
///
/// Grammar (all parts after the book are optional):
///   [chapter|ch] <chapter> [(:|colon|verse|verses) <verse> [range <verse>]]
///   [chapter|ch] <chapter> <bare-number-verse> [<bare-range>]
fn parse_after_book(book: &BibleBook, n0: usize, tokens: &[Token]) -> Option<ScriptureReference> {
    let mut cursor = n0;

    // Optional filler directly after the book ("John's gospel chapter one").
    if let Some(Token::Word(w)) = tokens.get(cursor) {
        if matches!(w.as_str(), "gospel" | "book" | "epistle" | "letter") {
            cursor += 1;
        }
    }

    // Optional "chapter" / "chapters" marker.
    while cursor < tokens.len() && is_chapter_marker(&tokens[cursor]) {
        cursor += 1;
    }

    let (chapter, after_chapter) = number_at(tokens, cursor)?;
    if chapter < 1 || chapter > u32::from(book.chapters) {
        return None;
    }
    cursor = after_chapter;

    // Optional separator: ":" token or the spoken word "colon".
    let mut saw_separator = false;
    if let Some(t) = tokens.get(cursor) {
        if matches!(t, Token::Colon) {
            cursor += 1;
            saw_separator = true;
        } else if let Token::Word(w) = t {
            if w == "colon" {
                cursor += 1;
                saw_separator = true;
            }
        }
    }

    // Optional verse marker ("verse"/"verses"/"v"/"vs").
    let mut saw_verse_marker = false;
    if let Some(t) = tokens.get(cursor) {
        if is_verse_marker(t) {
            cursor += 1;
            saw_verse_marker = true;
        }
    }

    // The verse itself: a number token or number words.
    let mut start_verse: Option<u16> = None;
    if let Some((v, after)) = number_at(tokens, cursor) {
        if (1..1000).contains(&v) {
            start_verse = Some(v as u16);
            cursor = after;
        }
    }

    // A separator or verse marker that was not followed by a number means the
    // reference is incomplete — do not guess a chapter-only reference.
    if start_verse.is_none() && (saw_separator || saw_verse_marker) {
        return None;
    }

    if start_verse.is_none() {
        return Some(ScriptureReference::new(book.id, chapter as u16));
    }

    // Optional range: "-", ":", "through", "to" (or "and") + verse.
    let mut end_verse = start_verse;
    if let Some(t) = tokens.get(cursor) {
        let is_connector =
            is_range_connector(t) || matches!(t, Token::Word(w) if w.as_str() == "and");
        if is_connector {
            cursor += 1;
            // Allow an optional verse marker after the connector.
            if let Some(t2) = tokens.get(cursor) {
                if is_verse_marker(t2) {
                    cursor += 1;
                }
            }
            if let Some((end_v, _)) = number_at(tokens, cursor) {
                if end_v >= u32::from(start_verse.unwrap_or(1)) && end_v < 1000 {
                    end_verse = Some(end_v as u16);
                }
            }
        }
    }

    let reference = ScriptureReference {
        book_id: book.id,
        chapter: chapter as u16,
        start_verse,
        end_verse,
    };
    reference.is_valid().then_some(reference)
}

/// Parses the first Scripture reference found in `text`, if any.
pub fn parse_reference(text: &str) -> Option<ScriptureReference> {
    find_references(text).into_iter().next()
}

/// Finds every Scripture reference that can be extracted from `text`.
///
/// A reference is only emitted when a canonical book is followed by a valid
/// chapter (and optional verse/range). Other numbers in the text are ignored.
pub fn find_references(text: &str) -> Vec<ScriptureReference> {
    let tokens = tokenize(text);
    let texts = token_strings(&tokens);
    let strs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();

    let mut refs = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        if let Some((book, consumed)) = find_book(&strs, i) {
            if let Some(reference) = parse_after_book(book, i + consumed, &tokens) {
                refs.push(reference);
                i += consumed;
                continue;
            }
        }
        i += 1;
    }
    refs
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_ref(text: &str, expected: ScriptureReference) {
        let parsed =
            parse_reference(text).unwrap_or_else(|| panic!("expected reference for {text:?}"));
        assert_eq!(parsed, expected, "for input {text:?}");
        assert_eq!(parsed.display(), expected.display(), "display for {text:?}");
    }

    fn assert_none(text: &str) {
        assert!(
            parse_reference(text).is_none(),
            "unexpected match for {text:?}"
        );
    }

    #[test]
    fn typed_colon_forms() {
        assert_ref("John 3:16", ScriptureReference::new(43, 3).with_verse(16));
        assert_ref(
            "John 3:16-18",
            ScriptureReference::new(43, 3).with_range(16, 18),
        );
        assert_ref("Psalm 23:1", ScriptureReference::new(19, 23).with_verse(1));
        assert_ref(
            "1 Corinthians 13:4-7",
            ScriptureReference::new(46, 13).with_range(4, 7),
        );
        assert_ref("Romans 8:28", ScriptureReference::new(45, 8).with_verse(28));
        assert_ref(
            "Romans 8:28-30",
            ScriptureReference::new(45, 8).with_range(28, 30),
        );
    }

    #[test]
    fn space_separated_typed_form() {
        // "John 3 16" — two adjacent numbers mean chapter + verse.
        assert_ref("John 3 16", ScriptureReference::new(43, 3).with_verse(16));
    }

    #[test]
    fn chapter_only() {
        assert_ref("Psalm 23", ScriptureReference::new(19, 23));
        assert_ref("John chapter 3", ScriptureReference::new(43, 3));
    }

    #[test]
    fn spoken_forms_with_markers() {
        assert_ref(
            "John chapter three verse sixteen",
            ScriptureReference::new(43, 3).with_verse(16),
        );
        assert_ref(
            "Romans chapter eight verse twenty eight",
            ScriptureReference::new(45, 8).with_verse(28),
        );
        assert_ref(
            "first Corinthians chapter thirteen verses four to seven",
            ScriptureReference::new(46, 13).with_range(4, 7),
        );
        assert_ref("Psalm twenty three", ScriptureReference::new(19, 23));
        assert_ref(
            "Psalm chapter ninety one verse one",
            ScriptureReference::new(19, 91).with_verse(1),
        );
    }

    #[test]
    fn spoken_number_word_forms() {
        // No markers: chapter + verse adjacency, like "John three sixteen".
        assert_ref(
            "John three sixteen",
            ScriptureReference::new(43, 3).with_verse(16),
        );
        assert_ref(
            "First Corinthians thirteen verse four",
            ScriptureReference::new(46, 13).with_verse(4),
        );
    }

    #[test]
    fn spoken_range_with_dash_words() {
        assert_ref(
            "John chapter three verse sixteen through eighteen",
            ScriptureReference::new(43, 3).with_range(16, 18),
        );
    }

    #[test]
    fn book_of_and_possessive_forms() {
        assert_ref(
            "the book of John chapter three verse sixteen",
            ScriptureReference::new(43, 3).with_verse(16),
        );
        assert_ref(
            "let's open John's gospel chapter one verse one",
            ScriptureReference::new(43, 1).with_verse(1),
        );
    }

    #[test]
    fn second_corinthians_form() {
        assert_ref(
            "second Corinthians chapter five verse seven",
            ScriptureReference::new(47, 5).with_verse(7),
        );
    }

    #[test]
    fn embedded_in_conversation() {
        assert_ref(
            "and today we are going to read John chapter three verse sixteen",
            ScriptureReference::new(43, 3).with_verse(16),
        );
        assert_ref(
            "please turn to first Corinthians chapter thirteen verses four to seven",
            ScriptureReference::new(46, 13).with_range(4, 7),
        );
    }

    #[test]
    fn negative_cases() {
        assert_none("I arrived at 3pm");
        assert_none("we have twenty people");
        assert_none("chapter three was difficult");
        assert_none("there are 4 cats and 2 dogs");
        assert_none("the verse was interesting");
        assert_none("John and Mary came to church"); // book but no ref
        assert_none("I love my job"); // "Job" is not followed by a chapter
    }

    #[test]
    fn invalid_chapter_rejected() {
        // John has only 21 chapters — these must not parse.
        assert_none("John 45:1");
        assert_none("Psalm 151");
    }

    #[test]
    fn multiple_references() {
        let refs = find_references("read John 3:16 and then Psalm 23");
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0], ScriptureReference::new(43, 3).with_verse(16));
        assert_eq!(refs[1], ScriptureReference::new(19, 23));
    }
}
