//! Typed Scripture reference model.
//!
//! A reference is never stored as an opaque string across the application —
//! they are parsed into [`ScriptureReference`] and only formatted for human
//! display at the edges.

use serde::{Deserialize, Serialize};

use super::books::{book_by_id, BibleBook};

/// A canonical Scripture reference.
///
/// `start_verse == None` means the whole chapter is referenced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptureReference {
    pub book_id: i32,
    pub chapter: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_verse: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_verse: Option<u16>,
}

impl ScriptureReference {
    pub fn new(book_id: i32, chapter: u16) -> Self {
        Self {
            book_id,
            chapter,
            start_verse: None,
            end_verse: None,
        }
    }

    pub fn with_verse(mut self, verse: u16) -> Self {
        self.start_verse = Some(verse);
        self.end_verse = Some(verse);
        self
    }

    pub fn with_range(mut self, start: u16, end: u16) -> Self {
        self.start_verse = Some(start);
        self.end_verse = Some(end);
        self
    }

    pub fn book(&self) -> Option<&'static BibleBook> {
        book_by_id(self.book_id)
    }

    /// Human readable canonical label, e.g. "John 3:16", "Psalm 23",
    /// "1 Corinthians 13:4-7".
    pub fn display(&self) -> String {
        let book = match self.book() {
            Some(b) => b.name,
            None => "Unknown Book",
        };
        match (self.start_verse, self.end_verse) {
            (Some(start), Some(end)) if start == end => {
                format!("{book} {}:{start}", self.chapter)
            }
            (Some(start), Some(end)) => format!("{book} {}:{start}-{end}", self.chapter),
            _ => format!("{book} {}", self.chapter),
        }
    }

    /// Human readable spoken label ("John chapter three verse sixteen") when
    /// available, otherwise the canonical label. Used for logging/diagnostics.
    pub fn to_spoken(&self) -> String {
        let book = self.book().map(|b| b.name).unwrap_or_default();
        match (self.start_verse, self.end_verse) {
            (Some(start), Some(end)) if start == end => {
                format!("{book} chapter {} verse {start}", self.chapter)
            }
            (Some(start), Some(end)) => {
                format!("{book} chapter {} verses {start} to {end}", self.chapter)
            }
            _ => format!("{book} chapter {}", self.chapter),
        }
    }

    /// Structural validation against the canonical chapter counts.
    pub fn is_valid(&self) -> bool {
        match self.book() {
            None => false,
            Some(book) => {
                self.chapter >= 1
                    && usize::from(self.chapter) <= book.chapters as usize
                    && match (self.start_verse, self.end_verse) {
                        (None, None) => true,
                        (Some(s), Some(e)) => s >= 1 && e >= s,
                        _ => false,
                    }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_forms() {
        assert_eq!(
            ScriptureReference::new(43, 3).with_verse(16).display(),
            "John 3:16"
        );
        assert_eq!(ScriptureReference::new(19, 23).display(), "Psalms 23");
        assert_eq!(
            ScriptureReference::new(46, 13).with_range(4, 7).display(),
            "1 Corinthians 13:4-7"
        );
        assert_eq!(
            ScriptureReference::new(45, 8).with_range(28, 30).display(),
            "Romans 8:28-30"
        );
    }

    #[test]
    fn validation() {
        // John has 21 chapters.
        assert!(ScriptureReference::new(43, 21).is_valid());
        assert!(!ScriptureReference::new(43, 22).is_valid());
        assert!(!ScriptureReference::new(43, 0).is_valid());
        assert!(!ScriptureReference::new(43, 3).with_range(9, 4).is_valid());
        assert!(!ScriptureReference::new(9999, 1).is_valid());
    }

    #[test]
    fn serde_roundtrip() {
        let r = ScriptureReference::new(43, 3).with_verse(16);
        let json = serde_json::to_string(&r).unwrap();
        assert_eq!(
            json,
            r#"{"bookId":43,"chapter":3,"startVerse":16,"endVerse":16}"#
        );
        let back: ScriptureReference = serde_json::from_str(&json).unwrap();
        assert_eq!(back, r);
    }
}
