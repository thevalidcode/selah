//! Local media library.
//!
//! Media files remain on disk; SQLite stores only metadata (see the `media`
//! table). This bootstrap validates imports and records them. Copying files
//! into the application media directory is a later-phase feature.

use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::Serialize;
use serde_json::Value;

use crate::bible::repository::MediaRepository;
use crate::errors::AppError;
use rusqlite::Connection;

const IMAGE_EXTS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "bmp"];
const VIDEO_EXTS: &[&str] = &["mp4", "mov", "m4v", "mkv", "webm"];
const AUDIO_EXTS: &[&str] = &["mp3", "wav", "flac", "ogg", "m4a"];

/// Flag protecting the frontend from unexpected metadata shapes.
pub type MediaMetadata = Value;

/// A media record as exposed to the rest of the application.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaItem {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub path: String,
    pub metadata: Option<MediaMetadata>,
    pub created_at: String,
}

impl MediaItem {
    fn new_from_path(path: &Path) -> Result<Self, AppError> {
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            return Err(AppError::Media(format!(
                "{} has no file extension",
                path.display()
            )));
        };
        let ext = ext.to_lowercase();
        let kind = if IMAGE_EXTS.contains(&ext.as_str()) {
            "image"
        } else if VIDEO_EXTS.contains(&ext.as_str()) {
            "video"
        } else if AUDIO_EXTS.contains(&ext.as_str()) {
            "audio"
        } else {
            return Err(AppError::Media(format!("unsupported media type .{ext}")));
        };

        let metadata = std::fs::metadata(path)
            .ok()
            .map(|m| serde_json::json!({ "size": m.len(), "kind": kind }));

        Ok(MediaItem {
            id: uuid::Uuid::new_v4().to_string(),
            kind: kind.to_string(),
            name: path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unnamed")
                .to_string(),
            path: path.to_string_lossy().to_string(),
            metadata,
            created_at: Utc::now().to_rfc3339(),
        })
    }
}

pub struct MediaLibrary;

impl MediaLibrary {
    pub fn list(conn: &Connection) -> Result<Vec<MediaItem>, AppError> {
        MediaRepository::new(conn).list()
    }

    /// Registers a local media file.
    pub fn import(conn: &Connection, path: PathBuf) -> Result<MediaItem, AppError> {
        if !path.is_absolute() {
            return Err(AppError::Media(format!(
                "path must be absolute: {}",
                path.display()
            )));
        }
        if !path.is_file() {
            return Err(AppError::Media(format!(
                "file does not exist: {}",
                path.display()
            )));
        }
        let item = MediaItem::new_from_path(&path)?;
        MediaRepository::new(conn).insert(&item)?;
        Ok(item)
    }

    pub fn remove(conn: &Connection, id: &str) -> Result<(), AppError> {
        MediaRepository::new(conn).remove(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_media_extensions() {
        // A real temp file keeps path validation honest.
        let dir = std::env::temp_dir().join(format!("selah-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let txt = dir.join("notes.txt");
        std::fs::write(&txt, "hello").unwrap();

        let err = MediaItem::new_from_path(&txt).unwrap_err();
        assert!(matches!(err, AppError::Media(_)));

        std::fs::remove_dir_all(&dir).unwrap();
    }
}
