//! Local media library.
//!
//! Media files remain on disk; SQLite stores only metadata (see the `media`
//! table). Files are read from a folder the operator picks at runtime — no path
//! is ever hardcoded — which is why this module offers both a directory browser
//! and a folder scan.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::Serialize;
use serde_json::Value;

use crate::bible::repository::MediaRepository;
use crate::errors::AppError;
use crate::models::presentation::MediaClip;
use rusqlite::Connection;

const IMAGE_EXTS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "bmp"];
const VIDEO_EXTS: &[&str] = &["mp4", "mov", "m4v", "mkv", "webm"];
const AUDIO_EXTS: &[&str] = &["mp3", "wav", "flac", "ogg", "m4a"];

/// How deep a folder scan will walk.
///
/// A media folder is meant to be browsed by a person, so a handful of levels is
/// plenty; the cap stops a scan of a home directory from running forever.
const MAX_SCAN_DEPTH: usize = 4;

/// Upper bound on files registered by a single scan, protecting the library
/// from an accidental scan of an entire disk.
const MAX_SCAN_FILES: usize = 2_000;

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
        let kind = kind_for_path(path).ok_or_else(|| {
            AppError::Media(format!(
                "Selah cannot present {}",
                path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.display().to_string())
            ))
        })?;

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

/// The media kind for a path, or `None` when Selah cannot present it.
pub fn kind_for_path(path: &Path) -> Option<&'static str> {
    let ext = path.extension()?.to_str()?.to_lowercase();
    if IMAGE_EXTS.contains(&ext.as_str()) {
        Some("image")
    } else if VIDEO_EXTS.contains(&ext.as_str()) {
        Some("video")
    } else if AUDIO_EXTS.contains(&ext.as_str()) {
        Some("audio")
    } else {
        None
    }
}

/// One row in the folder browser.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    /// `image` / `video` / `audio` for files Selah can present, else `None`.
    pub media_kind: Option<String>,
}

/// The contents of one folder, as shown by the media folder picker.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryListing {
    pub path: String,
    /// Parent folder, absent when already at the filesystem root.
    pub parent: Option<String>,
    pub entries: Vec<DirectoryEntry>,
}

/// The result of loading a folder into the library.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaScan {
    pub directory: String,
    /// Files registered for the first time by this scan.
    pub added: usize,
    /// Media files now known inside the folder (added + already registered).
    pub total: usize,
    /// Folders inside the chosen folder that could not be read.
    ///
    /// A locked folder, or one on a drive that was unplugged mid-scan, must not
    /// stop the rest of the library from loading — but the operator has to be
    /// told, otherwise files appear to be missing with no explanation.
    pub skipped: usize,
    /// True when Selah stopped before the end of the folder: either the file
    /// limit was reached or the folder tree was deeper than Selah walks.
    pub truncated: bool,
    pub items: Vec<MediaItem>,
}

/// What one walk of a folder tree found.
#[derive(Debug, Default)]
struct ScanWalk {
    files: Vec<PathBuf>,
    /// Folders that could not be read (permissions, a vanished folder).
    skipped: usize,
    /// True when the walk stopped early because of a limit.
    truncated: bool,
}

pub struct MediaLibrary;

impl MediaLibrary {
    pub fn list(conn: &Connection) -> Result<Vec<MediaItem>, AppError> {
        MediaRepository::new(conn).list()
    }

    /// Registers a local media file.
    ///
    /// Re-adding a file that is already in the library returns the existing
    /// record rather than creating a duplicate.
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
        let repo = MediaRepository::new(conn);
        if let Some(existing) = repo.find_by_path(&path.to_string_lossy())? {
            return Ok(existing);
        }
        let item = MediaItem::new_from_path(&path)?;
        repo.insert(&item)?;
        Ok(item)
    }

    /// Reads a folder and registers every media file inside it.
    ///
    /// This is the "pick a folder, load what is in it" flow: the caller never
    /// has to know a file path in advance.
    pub fn scan(
        conn: &Connection,
        directory: &Path,
        recursive: bool,
    ) -> Result<MediaScan, AppError> {
        if !directory.is_dir() {
            return Err(AppError::Media(format!(
                "not a folder: {}",
                directory.display()
            )));
        }

        // The folder the operator chose must itself be readable: a failure here
        // is a mistake they need to see. Folders *inside* it are a different
        // matter (see `collect_files`).
        if let Err(e) = std::fs::read_dir(directory) {
            return Err(AppError::Media(format!(
                "cannot read {}: {e}",
                directory.display()
            )));
        }

        let repo = MediaRepository::new(conn);
        let mut known: Vec<MediaItem> = repo.list()?;
        let mut known_paths: HashSet<String> = known.iter().map(|item| item.path.clone()).collect();

        let mut walk = ScanWalk::default();
        collect_files(directory, recursive, 0, &mut walk);
        walk.files.sort();

        let mut added = 0usize;
        for file in walk.files {
            let path = file.to_string_lossy().to_string();
            if known_paths.contains(&path) {
                continue;
            }
            let item = MediaItem::new_from_path(&file)?;
            repo.insert(&item)?;
            known_paths.insert(path);
            known.push(item);
            added += 1;
        }

        // Report only what lives inside the folder that was just loaded. The
        // path is compared by component, so `/media2` is never treated as being
        // inside `/media`.
        let mut items: Vec<MediaItem> = known
            .into_iter()
            .filter(|item| Path::new(&item.path).starts_with(directory))
            .collect();
        items.sort_by_key(|item| item.name.to_lowercase());

        Ok(MediaScan {
            directory: directory.to_string_lossy().to_string(),
            added,
            total: items.len(),
            skipped: walk.skipped,
            truncated: walk.truncated,
            items,
        })
    }

    /// Remembers the part of a video the operator wants to show.
    ///
    /// The range and the "stop or repeat" choice live in the record's metadata
    /// (JSON), so nothing is copied and no migration is needed. `None` forgets
    /// the range, putting the file back to playing whole.
    pub fn set_clip(
        conn: &Connection,
        id: &str,
        clip: Option<MediaClip>,
    ) -> Result<MediaItem, AppError> {
        let repo = MediaRepository::new(conn);
        let item = repo.find_by_id(id)?.ok_or_else(|| {
            AppError::Media(format!("no media file with id {id}; read the folder again"))
        })?;

        // Metadata is free-form, so anything unexpected (or missing) starts from
        // an empty object rather than being thrown away silently.
        let mut metadata = match item.metadata.clone() {
            Some(serde_json::Value::Object(map)) => map,
            _ => serde_json::Map::new(),
        };

        match clip.map(MediaClip::sanitized) {
            Some(clip) => {
                metadata.insert("clip".to_string(), clip.to_metadata_value());
            }
            None => {
                metadata.remove("clip");
            }
        }

        let metadata = serde_json::Value::Object(metadata);
        repo.update_metadata(id, &metadata)?;

        Ok(MediaItem {
            metadata: Some(metadata),
            ..item
        })
    }

    pub fn remove(conn: &Connection, id: &str) -> Result<(), AppError> {
        MediaRepository::new(conn).remove(id)
    }

    /// Lists one folder so the operator can walk to the folder they want.
    ///
    /// `None` starts at the user's home directory, which keeps the first screen
    /// somewhere recognisable instead of at the filesystem root.
    pub fn browse(path: Option<&Path>) -> Result<DirectoryListing, AppError> {
        let current = match path {
            Some(path) => path.to_path_buf(),
            None => home_directory()?,
        };
        if !current.is_dir() {
            return Err(AppError::Media(format!(
                "not a folder: {}",
                current.display()
            )));
        }

        let mut entries = Vec::new();
        let read = std::fs::read_dir(&current)
            .map_err(|e| AppError::Media(format!("cannot read {}: {e}", current.display())))?;

        for entry in read.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            // Hidden entries are noise in a picker; skip them.
            if name.starts_with('.') {
                continue;
            }
            let entry_path = entry.path();
            let is_dir = entry_path.is_dir();
            entries.push(DirectoryEntry {
                name,
                media_kind: if is_dir {
                    None
                } else {
                    kind_for_path(&entry_path).map(str::to_string)
                },
                path: entry_path.to_string_lossy().to_string(),
                is_dir,
            });
        }

        entries.sort_by(|a, b| {
            b.is_dir
                .cmp(&a.is_dir)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });

        Ok(DirectoryListing {
            parent: current.parent().map(|p| p.to_string_lossy().to_string()),
            path: current.to_string_lossy().to_string(),
            entries,
        })
    }
}

/// Collects presentable files under `directory`, up to [`MAX_SCAN_DEPTH`].
///
/// A folder inside the chosen one that cannot be read (permissions, a network
/// share that dropped, a folder deleted while the scan runs) is **skipped and
/// counted** rather than failing the whole scan: a church service should never
/// lose the rest of its library because one folder was locked. The count is
/// reported back so the operator is told, instead of files simply going
/// missing.
fn collect_files(directory: &Path, recursive: bool, depth: usize, out: &mut ScanWalk) {
    if depth > MAX_SCAN_DEPTH {
        // Deeper than Selah walks; say so rather than pretending the folder is
        // empty.
        out.truncated = true;
        return;
    }
    if out.files.len() >= MAX_SCAN_FILES {
        out.truncated = true;
        return;
    }

    let read = match std::fs::read_dir(directory) {
        Ok(read) => read,
        Err(err) => {
            tracing::warn!(
                error = %err,
                folder = %directory.display(),
                "a media folder could not be read; skipping it"
            );
            out.skipped += 1;
            return;
        }
    };

    for entry in read.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            if recursive {
                collect_files(&path, recursive, depth + 1, out);
            }
            continue;
        }
        if kind_for_path(&path).is_some() {
            out.files.push(path);
            if out.files.len() >= MAX_SCAN_FILES {
                // Anything after this is not read, so the folder is reported as
                // partly scanned rather than quietly incomplete.
                out.truncated = true;
                break;
            }
        }
    }
}

/// The user's home directory, without pulling in a platform crate.
fn home_directory() -> Result<PathBuf, AppError> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
        .ok_or_else(|| AppError::Media("cannot work out where your files are".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    /// A throwaway folder holding a couple of presentable files.
    fn scratch_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("selah-media-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn memory_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        db::migrations::run_migrations(&conn).unwrap();
        conn
    }

    #[test]
    fn rejects_non_media_extensions() {
        // A real temp file keeps path validation honest.
        let dir = scratch_dir();
        let txt = dir.join("notes.txt");
        std::fs::write(&txt, "hello").unwrap();

        let err = MediaItem::new_from_path(&txt).unwrap_err();
        assert!(matches!(err, AppError::Media(_)));

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn knows_which_files_it_can_present() {
        assert_eq!(kind_for_path(Path::new("/tmp/slide.PNG")), Some("image"));
        assert_eq!(kind_for_path(Path::new("/tmp/clip.mp4")), Some("video"));
        assert_eq!(kind_for_path(Path::new("/tmp/hymn.wav")), Some("audio"));
        assert_eq!(kind_for_path(Path::new("/tmp/notes.txt")), None);
        assert_eq!(kind_for_path(Path::new("/tmp/no-extension")), None);
    }

    #[test]
    fn scanning_a_folder_loads_its_media_once() {
        let dir = scratch_dir();
        std::fs::write(dir.join("slide.png"), b"not really a png").unwrap();
        std::fs::write(dir.join("notes.txt"), b"ignored").unwrap();
        let nested = dir.join("videos");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(nested.join("clip.mp4"), b"not really an mp4").unwrap();

        let conn = memory_db();

        // Without recursion only the top level is read.
        let first = MediaLibrary::scan(&conn, &dir, false).unwrap();
        assert_eq!(first.added, 1);
        assert_eq!(first.total, 1);
        assert_eq!(first.items[0].kind, "image");

        // With recursion the nested video is picked up too, and the already
        // registered picture is not duplicated.
        let second = MediaLibrary::scan(&conn, &dir, true).unwrap();
        assert_eq!(second.added, 1, "only the new file should be registered");
        assert_eq!(second.total, 2);
        assert!(second.items.iter().any(|item| item.kind == "video"));

        let reread = MediaLibrary::scan(&conn, &dir, true).unwrap();
        assert_eq!(reread.added, 0);
        assert_eq!(reread.total, 2);
        // Nothing was skipped and nothing was left unread, so the screen can
        // say "everything here is loaded" with confidence.
        assert_eq!(reread.skipped, 0);
        assert!(!reread.truncated);

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_chosen_time_range_is_remembered_for_a_file() {
        let dir = scratch_dir();
        let clip_file = dir.join("worship.mp4");
        std::fs::write(&clip_file, b"not really an mp4").unwrap();

        let conn = memory_db();
        let scan = MediaLibrary::scan(&conn, &dir, false).unwrap();
        let item = scan.items.first().cloned().unwrap();

        // 6s → 30s, repeating: the operator's choice for this file.
        let clip = MediaClip {
            start_ms: 6_000,
            end_ms: Some(30_000),
            repeat: true,
        };
        let updated = MediaLibrary::set_clip(&conn, &item.id, Some(clip)).unwrap();
        assert_eq!(
            MediaClip::from_metadata(updated.metadata.as_ref()),
            Some(clip)
        );

        // Re-reading the folder must not lose the range: the file is already
        // registered, so its record (and therefore its metadata) is kept.
        let again = MediaLibrary::scan(&conn, &dir, false).unwrap();
        assert_eq!(again.added, 0);
        assert_eq!(
            MediaClip::from_metadata(again.items[0].metadata.as_ref()),
            Some(clip)
        );

        // Forgetting the range puts the file back to playing whole.
        let cleared = MediaLibrary::set_clip(&conn, &item.id, None).unwrap();
        assert_eq!(MediaClip::from_metadata(cleared.metadata.as_ref()), None);

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_range_that_ends_before_it_starts_is_not_kept() {
        // A slip of the end slider would leave the player stuck on one frame,
        // so the end is dropped: "from 30s to the end of the file".
        let repaired = MediaClip {
            start_ms: 30_000,
            end_ms: Some(6_000),
            repeat: false,
        }
        .sanitized();
        assert_eq!(repaired.start_ms, 30_000);
        assert_eq!(repaired.end_ms, None);

        // A whole-file clip is recognised as such, which is how Selah knows
        // there is no range to carry.
        assert!(MediaClip::default().is_whole_file());
        assert!(!repaired.is_whole_file());
    }

    #[test]
    fn a_folder_that_is_not_there_is_reported() {
        let dir = scratch_dir();
        std::fs::write(dir.join("slide.png"), b"x").unwrap();

        let conn = memory_db();
        let scan = MediaLibrary::scan(&conn, &dir, true).unwrap();
        assert_eq!(scan.skipped, 0);
        assert!(!scan.truncated);

        // A folder that is not there is a mistake the operator must see, not a
        // silently empty library.
        let missing = dir.join("nothing-here");
        assert!(MediaLibrary::scan(&conn, &missing, false).is_err());

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn browsing_a_folder_lists_folders_first() {
        let dir = scratch_dir();
        std::fs::create_dir_all(dir.join("subfolder")).unwrap();
        std::fs::write(dir.join("slide.jpg"), b"x").unwrap();
        std::fs::write(dir.join(".hidden.jpg"), b"x").unwrap();
        std::fs::create_dir_all(dir.join(".git")).unwrap();

        let listing = MediaLibrary::browse(Some(&dir)).unwrap();
        let names: Vec<&str> = listing.entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec!["subfolder", "slide.jpg"]);
        assert!(listing.entries[0].is_dir);
        assert_eq!(listing.entries[1].media_kind.as_deref(), Some("image"));

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn browsing_something_that_is_not_a_folder_is_reported() {
        let err = MediaLibrary::browse(Some(Path::new("/definitely/not/here"))).unwrap_err();
        assert!(matches!(err, AppError::Media(_)));
    }
}
