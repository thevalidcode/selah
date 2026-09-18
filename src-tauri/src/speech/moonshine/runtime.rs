//! Locates the ONNX Runtime shared library.
//!
//! The `ort` crate is built with `load-dynamic`, so nothing is downloaded or
//! linked at build time; the runtime is `dlopen`ed on first use from
//! `ORT_DYLIB_PATH`. Selah fills that variable in during start-up so a normal
//! `pnpm tauri dev` works without the operator exporting anything by hand.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::errors::AppError;

/// Environment variable `ort` reads when loading the runtime.
const DYLIB_ENV: &str = "ORT_DYLIB_PATH";

/// File name of the macOS ONNX Runtime library.
#[cfg(target_os = "macos")]
const DYLIB_NAME: &str = "libonnxruntime.dylib";
#[cfg(target_os = "windows")]
const DYLIB_NAME: &str = "onnxruntime.dll";
#[cfg(all(unix, not(target_os = "macos")))]
const DYLIB_NAME: &str = "libonnxruntime.so";

/// Remembers the outcome so repeated calls are free.
static RESOLVED: OnceLock<Result<PathBuf, String>> = OnceLock::new();

/// Makes ONNX Runtime available to `ort`, returning the library that will be
/// used.
///
/// Searches, in order: an existing `ORT_DYLIB_PATH`, the application's own
/// resource directory (for bundled builds), then the usual Homebrew and
/// system locations. Fails with a message naming every path tried, so a
/// missing runtime is diagnosable without a debugger.
pub fn ensure_available(resource_dir: Option<&Path>) -> Result<PathBuf, AppError> {
    let outcome = RESOLVED
        .get_or_init(|| resolve(resource_dir, &system_search_dirs()).map_err(|e| e.to_string()));
    outcome
        .clone()
        .map_err(|message| AppError::Unavailable(format!("ONNX Runtime not found: {message}")))
}

/// Standard install locations for the runtime, most likely first.
fn system_search_dirs() -> Vec<PathBuf> {
    [
        // Homebrew: Apple Silicon, then Intel.
        "/opt/homebrew/opt/onnxruntime/lib",
        "/usr/local/opt/onnxruntime/lib",
        "/opt/homebrew/lib",
        "/usr/local/lib",
    ]
    .iter()
    .map(PathBuf::from)
    .collect()
}

/// Performs the actual search. Errors carry the list of candidate paths.
fn resolve(resource_dir: Option<&Path>, search_dirs: &[PathBuf]) -> Result<PathBuf, AppError> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Some(existing) = std::env::var_os(DYLIB_ENV) {
        let path = PathBuf::from(existing);
        if path.is_file() {
            tracing::info!(path = %path.display(), "using ONNX Runtime from ORT_DYLIB_PATH");
            return Ok(path);
        }
    }

    // Bundled alongside the executable / in the app's resource folder.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join(DYLIB_NAME));
        }
    }
    if let Some(dir) = resource_dir {
        candidates.push(dir.join(DYLIB_NAME));
        candidates.push(dir.join("lib").join(DYLIB_NAME));
    }

    candidates.extend(search_dirs.iter().map(|dir| dir.join(DYLIB_NAME)));

    if let Some(found) = candidates.iter().find(|path| path.is_file()) {
        // Set before `ort` performs its one-time load.
        std::env::set_var(DYLIB_ENV, found);
        tracing::info!(path = %found.display(), "ONNX Runtime located");
        return Ok(found.clone());
    }

    let tried = candidates
        .iter()
        .map(|path| format!("  - {}", path.display()))
        .collect::<Vec<_>>()
        .join("\n");

    Err(AppError::Unavailable(format!(
        "no {DYLIB_NAME} found. Install it with `brew install onnxruntime`, or point \
         {DYLIB_ENV} at your copy. Paths tried:\n{tried}"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_runtime_reports_every_candidate_path() {
        let previous = std::env::var_os(DYLIB_ENV);
        // A stale override must not be used.
        std::env::set_var(DYLIB_ENV, "/nonexistent/definitely/not/here.dylib");

        // With an empty search list the result cannot depend on whatever
        // happens to be installed on the machine running the tests.
        let err = resolve(Some(Path::new("/nonexistent/resources")), &[])
            .expect_err("resolution should fail when no candidate exists");

        let message = err.to_string();
        assert!(
            message.contains(DYLIB_NAME),
            "message should name the library"
        );
        assert!(
            message.contains("/nonexistent/resources"),
            "message should list the paths it tried"
        );

        match previous {
            Some(value) => std::env::set_var(DYLIB_ENV, value),
            None => std::env::remove_var(DYLIB_ENV),
        }
    }

    #[test]
    fn an_explicit_override_wins() {
        let dir = std::env::temp_dir().join(format!("selah-ort-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let library = dir.join(DYLIB_NAME);
        std::fs::write(&library, b"stub").unwrap();

        let previous = std::env::var_os(DYLIB_ENV);
        std::env::set_var(DYLIB_ENV, &library);

        let found = resolve(None, &[]).expect("the override should be used");
        assert_eq!(found, library);

        match previous {
            Some(value) => std::env::set_var(DYLIB_ENV, value),
            None => std::env::remove_var(DYLIB_ENV),
        }
        std::fs::remove_dir_all(&dir).ok();
    }
}
