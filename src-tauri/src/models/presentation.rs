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
            _ => return None,
        })
    }
}

/// The payload of a presentation item. Tagged so the payload can be extended
/// without changing the presentation engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ContentPayload {
    /// A resolved passage, ready for display (no DB access needed in the
    /// presentation window).
    Scripture {
        reference: String,
        translation: String,
        text: String,
    },
    /// Plain text (announcements, custom text, notes).
    Text { text: String },
    /// Local media file (image or video) referenced by filesystem path.
    Media { path: String },
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
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            content_type: ContentType::Scripture,
            title: format!("{translation} {reference}"),
            payload: ContentPayload::Scripture {
                reference: reference.to_string(),
                translation: translation.to_string(),
                text,
            },
        }
    }

    pub fn plain_text(title: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            content_type: ContentType::Text,
            title: title.into(),
            payload: ContentPayload::Text { text: text.into() },
        }
    }

    pub fn media(title: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            content_type: ContentType::Image,
            title: title.into(),
            payload: ContentPayload::Media { path: path.into() },
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
fn item_title(payload: &ContentPayload) -> String {
    match payload {
        ContentPayload::Scripture {
            reference,
            translation,
            ..
        } => {
            let combined = format!("{translation} {reference}");
            let trimmed = combined.trim();
            if trimmed.is_empty() {
                "Bible verse".to_string()
            } else {
                trimmed.to_string()
            }
        }
        ContentPayload::Text { text } => text
            .lines()
            .find(|line| !line.trim().is_empty())
            .map(|line| line.trim().chars().take(60).collect())
            .unwrap_or_else(|| "Words".to_string()),
        ContentPayload::Media { path } => std::path::Path::new(path)
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "Media".to_string()),
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
}
