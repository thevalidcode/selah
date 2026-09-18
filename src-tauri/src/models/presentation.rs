//! Presentation domain types.
//!
//! These are the generic content model for the presentation engine. The
//! engine is not Bible-specific: any [`ContentType`] with a serializable
//! [`ContentPayload`] can be pushed onto the projector.

use serde::{Deserialize, Serialize};

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
