//! Installs the Bible translations that travel with the application.
//!
//! Selah still imports nothing behind the operator's back from the network —
//! these files are bundled on disk next to the application, and the same
//! checked import path is used as for a file the operator chooses by hand.
//!
//! Seeding is idempotent: a translation already present is left untouched, so
//! this is safe to run on every start.

use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::errors::AppError;

use super::models::TranslationInfo;
use super::repository::BibleRepository;

/// One translation shipped with the application.
struct Bundled {
    /// File name inside the bundled folder.
    file: &'static str,
    /// Short id used everywhere afterwards.
    id: &'static str,
    name: &'static str,
    abbreviation: &'static str,
}

/// The translations that ship with Selah. All three are public domain, which
/// is why they can be redistributed here.
const BUNDLED: [Bundled; 3] = [
    Bundled {
        file: "web.sqlite",
        id: "web",
        name: "World English Bible",
        abbreviation: "WEB",
    },
    Bundled {
        file: "kjv.sqlite",
        id: "kjv",
        name: "King James Version",
        abbreviation: "KJV",
    },
    Bundled {
        file: "asv.sqlite",
        id: "asv",
        name: "American Standard Version",
        abbreviation: "ASV",
    },
];

/// Installs every bundled translation that is not installed yet.
///
/// Returns how many were added. A failure to read one file is logged and
/// skipped rather than aborting start-up — the rest of the app still works.
pub fn seed_bundled_translations(conn: &Connection, dir: &Path) -> Result<usize, AppError> {
    let repo = BibleRepository::new(conn);
    let already_has_translations = !repo.list_translations()?.is_empty();

    let mut installed = 0usize;
    for bundled in BUNDLED {
        let path = dir.join(bundled.file);
        if !path.is_file() {
            continue;
        }
        if repo.get_translation(bundled.id)?.is_some() {
            continue;
        }

        let info = TranslationInfo {
            id: bundled.id.to_string(),
            name: bundled.name.to_string(),
            language: "en".to_string(),
            abbreviation: Some(bundled.abbreviation.to_string()),
            // Only the very first translation becomes the default, and only
            // when the operator has not already chosen one.
            is_default: !already_has_translations && installed == 0,
        };

        match super::import::import_sqlite_translation(conn, &path, &info) {
            Ok(outcome) => {
                installed += 1;
                tracing::info!(
                    translation = %outcome.translation_id,
                    verses = outcome.verses_imported,
                    "bundled translation installed"
                );
            }
            Err(err) => {
                tracing::warn!(
                    file = %path.display(),
                    error = %err,
                    "bundled translation could not be installed"
                );
            }
        }
    }

    Ok(installed)
}

/// Finds the folder holding the bundled translations.
///
/// The location differs between a development run and an installed build, so
/// several candidates are tried. In debug builds the source tree is used as a
/// last resort, which keeps `pnpm tauri dev` working without a copy step.
pub fn resolve_bundled_dir(resource_dir: Option<&Path>) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Some(dir) = resource_dir {
        candidates.push(dir.join("bible"));
        candidates.push(dir.join("resources").join("bible"));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("bible"));
            candidates.push(dir.join("resources").join("bible"));
        }
    }
    #[cfg(debug_assertions)]
    candidates.push(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("bible"),
    );

    candidates.into_iter().find(|dir| dir.is_dir())
}
