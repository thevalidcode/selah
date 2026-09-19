//! Selah — offline-first church/service presentation application.
//!
//! Architecture:
//!
//! ```text
//! Microphone -> CPAL -> AudioBuffer -> VAD -> SpeechRecognizer -> Transcript
//!     -> deterministic ContentDetector -> ScriptureResolver -> SQLite
//!     -> PresentationEngine -> second monitor
//! ```
//!
//! Tauri commands are thin adapters (see [`commands`]); business logic lives
//! in the modules below.

pub mod audio;
pub mod bible;
pub mod commands;
pub mod db;
pub mod errors;
pub mod events;
pub mod logging;
pub mod media;
pub mod models;
pub mod presentation;
pub mod scripture;
pub mod songs;
pub mod speech;
pub mod state;
pub mod storage;

pub use errors::{AppError, CommandResult};
pub use state::AppState;

/// Builds and runs the Tauri application.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use tauri::Manager;

    tauri::Builder::default()
        .setup(|app| {
            // Paths first: logging writes into `<AppData>/logs`.
            let paths = storage::AppPaths::init(app.handle())?;
            logging::init(&paths.logs_dir);
            tracing::info!("selah starting");

            // Speech-to-text runs on ONNX Runtime. Resolve the shared library
            // once, up-front, so a missing runtime is reported clearly at
            // start-up instead of as an opaque failure on first transcription.
            #[cfg(feature = "moonshine")]
            {
                let resource_dir = app.path().resource_dir().ok();
                match crate::speech::moonshine::runtime::ensure_available(resource_dir.as_deref()) {
                    Ok(path) => {
                        tracing::info!(runtime = %path.display(), "ONNX Runtime ready")
                    }
                    Err(err) => tracing::warn!(
                        error = %err,
                        "ONNX Runtime unavailable; speech-to-text will remain off"
                    ),
                }
            }

            let state = AppState::init_with_paths(app.handle(), paths)?;
            app.manage(state);

            match app.handle().state::<AppState>().display.list_displays() {
                Ok(displays) => tracing::info!(count = displays.len(), "displays discovered"),
                Err(e) => tracing::warn!(error = %e, "could not enumerate displays"),
            }
            Ok(())
        })
        .invoke_handler(commands::all_handlers())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
