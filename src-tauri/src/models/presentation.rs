//! Presentation domain types.
//!
//! These are the generic content model for the presentation engine. The
//! engine is not Bible-specific: any [`ContentType`] with a serializable
//! [`ContentPayload`] can be pushed onto the projector.

use serde::{Deserialize, Serialize};

use crate::errors::AppError;

/// The kind of content a presentation item can carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Scripture,
    Lyrics,
    Text,
    Image,
    Video,
    Announcement,
    Slide,
    /// A song from the library, presented one section at a time.
    Song,
}

impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::Scripture => "scripture",
            ContentType::Lyrics => "lyrics",
            ContentType::Text => "text",
            ContentType::Image => "image",
            ContentType::Video => "video",
            ContentType::Announcement => "announcement",
            ContentType::Slide => "slide",
            ContentType::Song => "song",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "scripture" => ContentType::Scripture,
            "lyrics" => ContentType::Lyrics,
            "text" => ContentType::Text,
            "image" => ContentType::Image,
            "video" => ContentType::Video,
            "announcement" => ContentType::Announcement,
            "slide" => ContentType::Slide,
            "song" => ContentType::Song,
            _ => return None,
        })
    }
}

/// The payload of a presentation item. Tagged so the payload can be extended
/// without changing the presentation engine.
///
/// `#[serde(default)]` on the newer fields keeps items stored by an earlier
/// build loading cleanly — a saved item must never become unreadable just
/// because a field was added.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ContentPayload {
    /// A resolved passage, ready for display (no DB access needed in the
    /// presentation window).
    Scripture {
        reference: String,
        translation: String,
        text: String,
        /// Operator-supplied heading shown above the reference.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        heading: Option<String>,
        /// Exact heading size in CSS pixels. The Bible screen can set this for
        /// one verse without changing the saved projector defaults.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        heading_size: Option<u32>,
        /// Exact verse size in CSS pixels, set the same way as `heading_size`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        text_size: Option<u32>,
    },
    /// Plain text (announcements, custom text, notes).
    Text {
        /// Heading typed by the operator. This is what makes a custom heading
        /// survive a save/reload round-trip and reach the projector.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        heading: Option<String>,
        text: String,
    },
    /// Local media file (image or video) referenced by filesystem path.
    Media {
        path: String,
        /// `image`, `video` or `audio`. Absent for files imported by an
        /// earlier build; the presentation window then guesses from the
        /// extension. Named `mediaKind` because `kind` is the payload's tag.
        #[serde(default, rename = "mediaKind", skip_serializing_if = "Option::is_none")]
        media_kind: Option<String>,
        /// Optional caption shown over the media.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        heading: Option<String>,
    },
    /// One section of a song (verse, chorus, bridge...). Songs are stepped
    /// through section by section, exactly like verses of Scripture.
    Song {
        /// Song title, drawn as the heading.
        title: String,
        /// Section label (`Verse 1`, `Chorus`). Absent when the section has
        /// none.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        text: String,
        /// 1-based position of this section within the song.
        #[serde(default = "one")]
        index: u32,
        /// Total number of sections in the song.
        #[serde(default = "one")]
        total: u32,
    },
}

/// Serde default for counters that cannot sensibly be zero.
fn one() -> u32 {
    1
}

/// True when `value` is present and contains something other than whitespace.
fn non_empty(value: &Option<String>) -> Option<String> {
    value
        .as_ref()
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
        .map(|v| v.to_string())
}

/// An item the presentation engine can display.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresentationItem {
    pub id: String,
    pub content_type: ContentType,
    pub title: String,
    pub payload: ContentPayload,
}

impl PresentationItem {
    pub fn scripture(reference: &str, translation: &str, text: String) -> Self {
        Self::scripture_sized(reference, translation, text, None, None)
    }

    /// A passage whose heading and verse sizes were chosen on the Bible screen.
    ///
    /// `None` for either size means "use the saved projector setting", so this
    /// can be used for every passage without the Bible screen having to know
    /// what Settings currently holds.
    pub fn scripture_sized(
        reference: &str,
        translation: &str,
        text: String,
        heading_size: Option<u32>,
        text_size: Option<u32>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            content_type: ContentType::Scripture,
            title: format!("{translation} {reference}"),
            payload: ContentPayload::Scripture {
                reference: reference.to_string(),
                translation: translation.to_string(),
                text,
                heading: None,
                heading_size,
                text_size,
            },
        }
    }

    pub fn plain_text(title: impl Into<String>, text: impl Into<String>) -> Self {
        let heading = title.into();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            content_type: ContentType::Text,
            title: heading.clone(),
            payload: ContentPayload::Text {
                heading: Some(heading).filter(|h| !h.trim().is_empty()),
                text: text.into(),
            },
        }
    }

    /// A media file ready for the projector.
    ///
    /// `heading` is only set when a caption was actually supplied: a picture or
    /// video should take over the screen without the file name printed over it.
    pub fn media(
        title: impl Into<String>,
        path: impl Into<String>,
        kind: Option<String>,
        heading: Option<String>,
    ) -> Self {
        let title = title.into();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            content_type: match kind.as_deref() {
                Some("video") => ContentType::Video,
                _ => ContentType::Image,
            },
            title,
            payload: ContentPayload::Media {
                path: path.into(),
                media_kind: kind,
                heading: non_empty(&heading),
            },
        }
    }

    /// One section of a song, ready for the projector.
    ///
    /// Every section becomes its own projectable item so the operator can walk
    /// through a song verse by verse with Next/Back, exactly as they do with
    /// Scripture.
    pub fn song_section(
        song_title: &str,
        label: Option<String>,
        text: String,
        index: u32,
        total: u32,
    ) -> Self {
        let label = label.filter(|l| !l.trim().is_empty());
        let title = match &label {
            Some(label) => format!("{song_title} — {label}"),
            None => format!("{song_title} — {index}/{total}"),
        };
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            content_type: ContentType::Song,
            title,
            payload: ContentPayload::Song {
                title: song_title.to_string(),
                label,
                text,
                index,
                total,
            },
        }
    }
}

/// The state of the live presentation engine (what is on the projector).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresentationState {
    pub current: Option<PresentationItem>,
    pub queue: Vec<PresentationItem>,
    pub history: Vec<PresentationItem>,
}

/// A saved presentation (rows in the `presentations` table).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Presentation {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    /// Lightweight list of item records (type name + JSON payload).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<PresentationItemRecord>,
}

/// Row shape of a persisted presentation item.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresentationItemRecord {
    pub id: String,
    pub presentation_id: String,
    pub type_name: String,
    pub position: i64,
    pub payload: String,
}

impl PresentationItemRecord {
    /// Rebuilds a projectable item from a stored row.
    ///
    /// Saved items keep their content as JSON so new content kinds never need
    /// a schema change. Without this step a saved verse or text block can be
    /// listed in the interface but never actually reach the screen — which is
    /// exactly the gap this closes.
    pub fn to_item(&self) -> Result<PresentationItem, AppError> {
        let payload: ContentPayload = serde_json::from_str(&self.payload).map_err(|e| {
            AppError::InvalidConfiguration(format!(
                "saved item {} has content Selah cannot read: {e}",
                self.id
            ))
        })?;

        let content_type = ContentType::parse(&self.type_name).unwrap_or(match payload {
            ContentPayload::Scripture { .. } => ContentType::Scripture,
            ContentPayload::Text { .. } => ContentType::Text,
            ContentPayload::Media { .. } => ContentType::Image,
            ContentPayload::Song { .. } => ContentType::Song,
        });

        Ok(PresentationItem {
            id: self.id.clone(),
            content_type,
            title: item_title(&payload),
            payload,
        })
    }
}

/// A short heading for an item, used where only a label is needed.
///
/// An operator-supplied heading always wins: if someone typed a heading for
/// their words, that is what should appear on the projector.
fn item_title(payload: &ContentPayload) -> String {
    match payload {
        ContentPayload::Scripture {
            reference,
            translation,
            heading,
            ..
        } => {
            if let Some(heading) = non_empty(heading) {
                return heading;
            }
            let combined = format!("{translation} {reference}");
            let trimmed = combined.trim();
            if trimmed.is_empty() {
                "Bible verse".to_string()
            } else {
                trimmed.to_string()
            }
        }
        ContentPayload::Text { heading, text } => non_empty(heading).unwrap_or_else(|| {
            text.lines()
                .find(|line| !line.trim().is_empty())
                .map(|line| line.trim().chars().take(60).collect())
                .unwrap_or_else(|| "Words".to_string())
        }),
        ContentPayload::Media { path, heading, .. } => non_empty(heading).unwrap_or_else(|| {
            std::path::Path::new(path)
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| "Media".to_string())
        }),
        ContentPayload::Song { title, label, .. } => match non_empty(label) {
            Some(label) => format!("{title} — {label}"),
            None => title.clone(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(type_name: &str, payload: &str) -> PresentationItemRecord {
        PresentationItemRecord {
            id: "item-1".to_string(),
            presentation_id: "deck-1".to_string(),
            type_name: type_name.to_string(),
            position: 1,
            payload: payload.to_string(),
        }
    }

    #[test]
    fn stored_scripture_becomes_a_projectable_item() {
        // This is the step that was missing: without it a saved verse could be
        // listed in the interface but never reach the screen.
        let row = record(
            "scripture",
            r#"{"kind":"scripture","reference":"John 3:16","translation":"WEB","text":"For God so loved"}"#,
        );
        let item = row.to_item().expect("should convert");
        assert_eq!(item.content_type, ContentType::Scripture);
        assert_eq!(item.title, "WEB John 3:16");
        assert!(matches!(
            item.payload,
            ContentPayload::Scripture { ref text, .. } if text == "For God so loved"
        ));
    }

    #[test]
    fn stored_text_uses_its_first_line_as_a_title() {
        let row = record("text", r#"{"kind":"text","text":"\n  Welcome everyone\n"}"#);
        let item = row.to_item().expect("should convert");
        assert_eq!(item.title, "Welcome everyone");
    }

    #[test]
    fn an_unreadable_payload_is_reported_not_ignored() {
        let err = record("text", "not json").to_item().unwrap_err();
        let message = err.to_string();
        assert!(message.contains("item-1"), "got: {message}");
    }

    #[test]
    fn an_unknown_kind_falls_back_to_the_payload() {
        let row = record("mystery", r#"{"kind":"text","text":"Hello"}"#);
        let item = row.to_item().expect("should convert");
        assert_eq!(item.content_type, ContentType::Text);
    }

    #[test]
    fn a_stored_heading_is_kept_and_becomes_the_title() {
        // The heading an operator typed used to be dropped on the way to the
        // screen; it now survives the round-trip.
        let row = record(
            "text",
            r#"{"kind":"text","heading":"Welcome","text":"Good morning everyone"}"#,
        );
        let item = row.to_item().expect("should convert");
        assert_eq!(item.title, "Welcome");
        assert!(matches!(
            item.payload,
            ContentPayload::Text {
                ref heading,
                ..
            } if heading.as_deref() == Some("Welcome")
        ));
    }

    #[test]
    fn stored_media_keeps_its_kind_and_caption() {
        // A file saved by an earlier build has no `kind`; it must still load
        // and simply fall back to the file name as its label.
        let legacy = record(
            "image",
            r#"{"kind":"media","path":"/tmp/slide.png","unexpected":true}"#,
        );
        let item = legacy.to_item().expect("should convert");
        assert_eq!(item.title, "slide.png");
        assert!(matches!(
            item.payload,
            ContentPayload::Media {
                media_kind: None,
                ..
            }
        ));

        let row = record(
            "video",
            r#"{"kind":"media","path":"/tmp/clip.mp4","mediaKind":"video","heading":"Baptism"}"#,
        );
        let item = row.to_item().expect("should convert");
        assert_eq!(item.content_type, ContentType::Video);
        assert_eq!(item.title, "Baptism");
    }

    #[test]
    fn a_picture_is_projected_without_a_caption_unless_one_is_given() {
        let plain = PresentationItem::media(
            "baptism.jpg",
            "/tmp/baptism.jpg",
            Some("image".to_string()),
            None,
        );
        assert_eq!(plain.title, "baptism.jpg");
        assert!(matches!(
            plain.payload,
            ContentPayload::Media { heading: None, .. }
        ));

        let captioned = PresentationItem::media(
            "Baptism",
            "/tmp/baptism.jpg",
            Some("image".to_string()),
            Some("Baptism".to_string()),
        );
        assert!(matches!(
            captioned.payload,
            ContentPayload::Media { ref heading, .. } if heading.as_deref() == Some("Baptism")
        ));
    }

    #[test]
    fn a_song_section_becomes_its_own_projectable_item() {
        let item = PresentationItem::song_section(
            "Amazing Grace",
            Some("Verse 1".to_string()),
            "Amazing grace, how sweet the sound".to_string(),
            1,
            3,
        );
        assert_eq!(item.content_type, ContentType::Song);
        assert_eq!(item.title, "Amazing Grace — Verse 1");
        assert!(matches!(
            item.payload,
            ContentPayload::Song { index: 1, total: 3, ref text, .. }
                if text == "Amazing grace, how sweet the sound"
        ));
    }

    #[test]
    fn sizes_chosen_on_the_bible_screen_reach_the_projector() {
        // The Bible screen can enlarge a heading or shrink a long verse for one
        // passage only; those sizes travel with the item.
        let item = PresentationItem::scripture_sized(
            "John 3:16",
            "KJV",
            "For God so loved the world".to_string(),
            Some(120),
            Some(72),
        );
        assert!(matches!(
            item.payload,
            ContentPayload::Scripture {
                heading_size: Some(120),
                text_size: Some(72),
                ..
            }
        ));

        // Leaving them out keeps the saved projector defaults in charge.
        let plain = PresentationItem::scripture("John 3:16", "KJV", "…".to_string());
        assert!(matches!(
            plain.payload,
            ContentPayload::Scripture {
                heading_size: None,
                text_size: None,
                ..
            }
        ));
    }

    #[test]
    fn a_passage_saved_before_sizes_existed_still_loads() {
        // Saved items keep their payload as JSON, so an item stored by an
        // earlier build has no size fields at all.
        let row = record(
            "scripture",
            r#"{"kind":"scripture","reference":"Psalm 23","translation":"WEB","text":"The Lord is my shepherd"}"#,
        );
        let item = row.to_item().expect("should convert");
        assert!(matches!(
            item.payload,
            ContentPayload::Scripture {
                heading_size: None,
                text_size: None,
                ..
            }
        ));
    }
}
