//! Centralized Rust application state.
//!
//! Managed once via `app.manage(AppState)`. No global mutable variables —
//! every Tauri command receives `State<'_, AppState>` and borrows what it
//! needs.

use std::sync::Mutex;

use tauri::AppHandle;

use crate::audio::AudioManager;
use crate::bible::Database;
use crate::errors::AppError;
use crate::models::settings::AppSettings;
use crate::presentation::{DisplayManager, PresentationEngine};
use crate::speech::{EnergyVad, SpeechManager};
use crate::storage::AppPaths;

/// The application's shared services.
pub struct AppState {
    pub db: Mutex<Database>,
    pub audio: AudioManager,
    pub speech: SpeechManager,
    pub presentation: PresentationEngine,
    pub display: DisplayManager,
    pub settings: Mutex<AppSettings>,
    pub paths: AppPaths,
}

impl AppState {
    /// Builds all services during Tauri `setup`.
    pub fn init(app: &AppHandle) -> Result<Self, AppError> {
        let paths = AppPaths::init(app)?;
        Self::init_with_paths(app, paths)
    }

    /// Builds all services with already-resolved storage paths.
    ///
    /// `setup` resolves the paths first so logging can start before any service
    /// is constructed.
    pub fn init_with_paths(app: &AppHandle, paths: AppPaths) -> Result<Self, AppError> {
        let db = Database::open(&paths.db_path)?;

        // Restore persisted settings (or defaults).
        let settings = {
            let conn = db
                .inner
                .lock()
                .map_err(|_| AppError::Internal("database lock poisoned".into()))?;
            let repo = crate::bible::repository::SettingsRepository::new(&conn);
            repo.get_json::<AppSettings>(crate::models::settings::SETTINGS_KEY)?
                .unwrap_or_default()
        };

        let audio = AudioManager::default();
        let recognizer_kind = settings.speech.recognizer;
        let recognizer = crate::speech::recognizer::recognizer_for(recognizer_kind);
        let vad = Box::new(EnergyVad::default()) as Box<dyn crate::speech::VoiceActivityDetector>;

        // Load the configured whisper model up-front when asked for it.
        let mut recognizer = recognizer;
        if recognizer_kind == crate::models::settings::RecognizerKind::Whisper {
            if let Some(model) = settings.speech.model_path.clone() {
                if !model.is_empty() {
                    recognizer.load_model(&model)?;
                }
            }
        }

        let speech = SpeechManager::new(
            app.clone(),
            audio.buffer(),
            recognizer,
            vad,
            &settings.speech,
        );

        Ok(Self {
            db: Mutex::new(db),
            audio,
            speech,
            presentation: PresentationEngine::new(),
            display: DisplayManager::new(app.clone()),
            settings: Mutex::new(settings),
            paths,
        })
    }

    /// Locks the database connection and runs `f` against it.
    pub fn with_conn<T>(
        &self,
        f: impl FnOnce(&rusqlite::Connection) -> Result<T, AppError>,
    ) -> Result<T, AppError> {
        let db = self
            .db
            .lock()
            .map_err(|_| AppError::Internal("database lock poisoned".to_string()))?;
        db.with_conn(f)
    }

    /// Persists the current settings document.
    pub fn save_settings(&self) -> Result<(), AppError> {
        let settings = self
            .settings
            .lock()
            .map_err(|_| AppError::Internal("settings lock poisoned".to_string()))?
            .clone();
        self.with_conn(|conn| {
            crate::bible::repository::SettingsRepository::new(conn)
                .set_json(crate::models::settings::SETTINGS_KEY, &settings)
        })
    }
}
