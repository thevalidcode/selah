//! Application storage paths.
//!
//! Everything lives under the Tauri application data directory — no absolute
//! paths, everything offline, nothing in the source repo at runtime.

use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use crate::errors::AppError;

/// Resolved storage locations for the application.
#[derive(Debug, Clone)]
pub struct AppPaths {
    pub data_dir: PathBuf,
    pub db_path: PathBuf,
    pub models_dir: PathBuf,
    pub moonshine_models_dir: PathBuf,
    pub vad_models_dir: PathBuf,
    pub media_dir: PathBuf,
    pub logs_dir: PathBuf,
}

impl AppPaths {
    /// Resolves directories from the Tauri app-data directory and creates
    /// them if missing.
    pub fn init(app: &AppHandle) -> Result<Self, AppError> {
        let data_dir = app
            .path()
            .app_data_dir()
            .map_err(|e| AppError::Internal(format!("cannot resolve app data directory: {e}")))?;

        let paths = Self {
            db_path: data_dir.join("database").join("selah.db"),
            models_dir: data_dir.join("models"),
            moonshine_models_dir: data_dir.join("models").join("moonshine"),
            vad_models_dir: data_dir.join("models").join("vad"),
            media_dir: data_dir.join("media"),
            logs_dir: data_dir.join("logs"),
            data_dir,
        };

        for dir in [
            &paths.data_dir,
            &paths
                .db_path
                .parent()
                .map(PathBuf::from)
                .unwrap_or_default(),
            &paths.models_dir,
            &paths.moonshine_models_dir,
            &paths.vad_models_dir,
            &paths.media_dir,
            &paths.media_dir.join("images"),
            &paths.media_dir.join("videos"),
            &paths.media_dir.join("audio"),
            &paths.logs_dir,
        ] {
            if dir.as_os_str().is_empty() {
                continue;
            }
            std::fs::create_dir_all(dir)
                .map_err(|e| AppError::Internal(format!("cannot create {}: {e}", dir.display())))?;
        }

        Ok(paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn speech_models_nest_under_models() {
        let root = PathBuf::from("/tmp/selah-test-paths");
        let paths = AppPaths {
            data_dir: root.clone(),
            db_path: root.join("database").join("selah.db"),
            models_dir: root.join("models"),
            moonshine_models_dir: root.join("models").join("moonshine"),
            vad_models_dir: root.join("models").join("vad"),
            media_dir: root.join("media"),
            logs_dir: root.join("logs"),
        };
        assert!(paths.moonshine_models_dir.starts_with(&paths.models_dir));
        assert!(paths.vad_models_dir.starts_with(&paths.models_dir));
    }
}
