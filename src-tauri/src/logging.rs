//! Logging setup.
//!
//! Selah uses **one** logging stack: `tracing` + `tracing-subscriber`.
//! (Tauri's `tauri-plugin-log` is intentionally not used — it installs its own
//! global logger, which conflicts with the tracing subscriber.)
//!
//! Logs go to stderr and to a daily rolling file under the application data
//! directory (`logs/selah.log`).
//!
//! Never logged: microphone audio, file contents of imported media, or any user
//! personal data. Audio is processed in memory only.

use std::path::PathBuf;
use std::sync::OnceLock;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

/// Keeps the file appender's worker thread alive for the process lifetime.
static FILE_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

/// Installs the global tracing subscriber. Safe to call more than once — only
/// the first call has an effect.
pub fn init(logs_dir: &PathBuf) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("selah_lib=info,selah=info"));

    let stderr_layer = fmt::layer()
        .with_target(false)
        .compact()
        .with_writer(std::io::stderr);

    // The logs directory is created by `AppPaths::init` before this runs.
    let file_appender = tracing_appender::rolling::daily(logs_dir, "selah.log");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);
    let _ = FILE_GUARD.set(guard);

    let file_layer = fmt::layer()
        .with_target(true)
        .with_ansi(false)
        .with_writer(file_writer);

    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(stderr_layer)
        .with(file_layer)
        .try_init();
}
