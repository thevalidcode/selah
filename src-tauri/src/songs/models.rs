//! Song domain types.
//!
//! A song is a title plus an ordered list of sections (verse, chorus, bridge…).
//! Selah presents one section at a time, exactly the way it presents one verse
//! of Scripture at a time, so a long song never has to be squeezed onto a
//! single screen.

use serde::{Deserialize, Serialize};

use crate::errors::AppError;

/// One section of a song.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongSection {
    pub id: String,
    /// Optional label drawn above the words ("Verse 1", "Chorus").
    pub label: Option<String>,
    pub text: String,
    /// 1-based order within the song.
    pub position: i64,
}

/// A song in the library, with its sections in order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    pub id: String,
    pub title: String,
    pub author: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sections: Vec<SongSection>,
}

/// What a caller supplies to create or replace a song.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongInput {
    pub title: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub sections: Vec<SongSectionInput>,
}

/// A section supplied by the caller (no id — the store assigns one).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongSectionInput {
    #[serde(default)]
    pub label: Option<String>,
    pub text: String,
}

impl SongInput {
    /// Trims the input and rejects anything that would present badly.
    ///
    /// Blank sections are dropped rather than rejected: an operator who leaves
    /// an empty section row is tidying up, not making a mistake. A song with no
    /// words left, however, is a real error.
    pub fn normalized(self) -> Result<Self, AppError> {
        let title = self.title.trim().to_string();
        if title.is_empty() {
            return Err(AppError::InvalidConfiguration(
                "a song needs a title".to_string(),
            ));
        }

        let author = self
            .author
            .map(|a| a.trim().to_string())
            .filter(|a| !a.is_empty());

        let sections = self
            .sections
            .into_iter()
            .filter_map(|section| {
                let text = section.text.trim().to_string();
                if text.is_empty() {
                    return None;
                }
                Some(SongSectionInput {
                    label: section
                        .label
                        .map(|l| l.trim().to_string())
                        .filter(|l| !l.is_empty()),
                    text,
                })
            })
            .collect::<Vec<_>>();

        if sections.is_empty() {
            return Err(AppError::InvalidConfiguration(
                "a song needs at least one section with words".to_string(),
            ));
        }

        Ok(Self {
            title,
            author,
            sections,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn section(label: &str, text: &str) -> SongSectionInput {
        SongSectionInput {
            label: Some(label.to_string()),
            text: text.to_string(),
        }
    }

    #[test]
    fn normalizing_trims_and_drops_blank_sections() {
        let input = SongInput {
            title: "  Amazing Grace  ".to_string(),
            author: Some("  John Newton ".to_string()),
            sections: vec![
                section("Verse 1", "  Amazing grace  "),
                // Dropped: a section with no words is not worth presenting.
                section("", "   "),
                SongSectionInput {
                    // Kept, but the whitespace-only label is dropped.
                    label: Some("   ".to_string()),
                    text: "  My chains are gone  ".to_string(),
                },
            ],
        }
        .normalized()
        .expect("should normalise");

        assert_eq!(input.title, "Amazing Grace");
        assert_eq!(input.author.as_deref(), Some("John Newton"));
        assert_eq!(input.sections.len(), 2);
        assert_eq!(input.sections[0].text, "Amazing grace");
        assert_eq!(input.sections[1].label, None);
        assert_eq!(input.sections[1].text, "My chains are gone");
    }

    #[test]
    fn a_song_without_a_title_is_rejected() {
        let err = SongInput {
            title: "   ".to_string(),
            author: None,
            sections: vec![section("Verse 1", "words")],
        }
        .normalized()
        .unwrap_err();
        assert!(err.to_string().contains("title"));
    }

    #[test]
    fn a_song_without_words_is_rejected() {
        let err = SongInput {
            title: "Untitled".to_string(),
            author: None,
            sections: vec![section("Verse 1", "  ")],
        }
        .normalized()
        .unwrap_err();
        assert!(err.to_string().contains("section"));
    }
}
