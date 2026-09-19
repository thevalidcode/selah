//! Media file access.
//!
//! Selah never hardcodes a filesystem path for media. The operator picks a
//! folder at runtime, and only that folder — plus any single file they added by
//! hand — is readable by the projector window through Tauri's `asset:`
//! protocol. The webview therefore never needs blanket access to the disk.
//!
//! Nothing here stores state: the access list lives in Tauri's own asset
//! protocol scope, which is granted at start-up from saved settings and again
//! whenever the operator chooses a folder.

use std::path::Path;

use tauri::{AppHandle, Manager};

use crate::errors::AppError;

/// Grants the projector window read access to `directory` and everything
/// inside it.
pub fn grant_directory(app: &AppHandle, directory: &Path) -> Result<(), AppError> {
    if !directory.is_dir() {
        return Err(AppError::Media(format!(
            "folder does not exist: {}",
            directory.display()
        )));
    }
    app.asset_protocol_scope()
        .allow_directory(directory, true)
        .map_err(|e| AppError::Media(format!("could not open {}: {e}", directory.display())))
}

/// Grants the projector window read access to a single file.
pub fn grant_file(app: &AppHandle, file: &Path) -> Result<(), AppError> {
    app.asset_protocol_scope()
        .allow_file(file)
        .map_err(|e| AppError::Media(format!("could not open {}: {e}", file.display())))
}

/// Whether the projector window is allowed to read `path` right now.
///
/// Used for diagnostics; the protocol handler enforces this itself.
pub fn is_allowed(app: &AppHandle, path: &Path) -> bool {
    app.asset_protocol_scope().is_allowed(path)
}
