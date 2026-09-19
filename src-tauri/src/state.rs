//! Centralized Rust application state.
//!
//! Managed once via `app.manage(AppState)`. No global mutable variables —
//! every Tauri command receives `State<'_, AppState>` and borrows what it
//! needs.

use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager};

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
    /// What the projector says it is playing, so the Media screen can show
    /// "playing" and offer start/stop without guessing.
    pub media_playback: Mutex<crate::events::MediaPlaybackState>,
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

        // Install the public-domain translations that ship with Selah, so a
        // fresh install can show Scripture without a setup step. Already
        // installed translations are left alone, making this safe every start.
        let resource_dir = app.path().resource_dir().ok();
        if let Some(dir) = crate::bible::resolve_bundled_dir(resource_dir.as_deref()) {
            match db.with_conn(|conn| crate::bible::seed_bundled_translations(conn, &dir)) {
                Ok(0) => tracing::debug!("bundled Bible translations already installed"),
                Ok(added) => {
                    tracing::info!(added, "bundled Bible translations installed")
                }
                Err(err) => tracing::warn!(
                    error = %err,
                    "bundled Bible translations could not be installed"
                ),
            }
        } else {
            tracing::warn!("no bundled Bible folder found; import a translation manually");
        }

        // Restore persisted settings (or defaults).
        let mut settings = {
            let conn = db
                .inner
                .lock()
                .map_err(|_| AppError::Internal("database lock poisoned".into()))?;
            let repo = crate::bible::repository::SettingsRepository::new(&conn);
            repo.get_json::<AppSettings>(crate::models::settings::SETTINGS_KEY)?
                .unwrap_or_default()
        };

        // Repair projector settings that would produce an unusable screen — a
        // malformed colour, or a font too small to read from the back of a
        // room. These values are persisted by hand-edited or partially written
        // settings documents, so fix them once and carry on.
        let (presentation, presentation_repaired) = settings.presentation.sanitized();
        if presentation_repaired {
            tracing::warn!("presentation settings were out of range; defaults restored");
            settings.presentation = presentation;
        }

        // Media is served to the projector window from a folder the operator
        // chose at runtime. Re-open that folder now so saved media still works
        // after a restart, without re-picking it.
        if let Some(directory) = settings.media.directory.clone() {
            let path = std::path::PathBuf::from(&directory);
            match crate::media::grant_directory(app, &path) {
                Ok(()) => tracing::info!(directory = %directory, "media folder opened"),
                Err(err) => tracing::warn!(
                    error = %err,
                    "saved media folder could not be opened; it can be chosen again on the Media screen"
                ),
            }
        }

        // A branding logo has to be readable by the projector window too, and
        // it lives outside the media folder. Re-grant it on every start so
        // branding survives a restart without re-picking the file.
        if let Some(logo) = settings.presentation.branding.logo.clone() {
            let path = std::path::PathBuf::from(&logo);
            match crate::media::grant_file(app, &path) {
                Ok(()) => tracing::info!(logo = %logo, "branding logo opened"),
                Err(err) => tracing::warn!(
                    error = %err,
                    "branding logo could not be opened; it can be chosen again in Settings"
                ),
            }
        }

        let audio = AudioManager::default();
        let recognizer_kind = settings.speech.recognizer;
        let recognizer = crate::speech::recognizer::recognizer_for(recognizer_kind);
        let vad = Box::new(EnergyVad::default()) as Box<dyn crate::speech::VoiceActivityDetector>;

        // Resolve the voice model folder.
        //
        // An older build stored the path to a single whisper model *file*
        // here. Anything that is not an existing folder is ignored in favour
        // of the application's own model folder, so an upgrade keeps working
        // without the operator hand-editing settings.
        let mut settings_persist_needed = presentation_repaired;
        if recognizer_kind == crate::models::settings::RecognizerKind::Moonshine {
            let configured = settings.speech.model_path.clone().unwrap_or_default();
            let usable = !configured.is_empty() && std::path::Path::new(&configured).is_dir();
            if !usable {
                if !configured.is_empty() {
                    tracing::info!(
                        configured = %configured,
                        fallback = %paths.moonshine_models_dir.display(),
                        "stored voice model path is not a folder; using the default model folder"
                    );
                }
                settings.speech.model_path = Some(paths.moonshine_models_dir.display().to_string());
                settings_persist_needed = true;
            }
        }

        // Load the configured Moonshine model up-front.
        //
        // A missing or unreadable model must NOT stop the app from starting:
        // a hard failure here would leave the operator with no way to correct
        // the path from Settings. Instead, log the problem and start with the
        // model unloaded — the UI then reports "voice model ready: no" and a
        // corrected path is picked up by `SpeechManager::reconfigure` the next
        // time settings are saved.
        let mut recognizer = recognizer;
        if recognizer_kind == crate::models::settings::RecognizerKind::Moonshine {
            if let Some(model) = settings.speech.model_path.clone() {
                if let Err(err) = recognizer.load_model(&model) {
                    tracing::warn!(
                        model = %model,
                        error = %err,
                        "speech model did not load; Selah starts without speech-to-text \
                         until the model path in Settings is corrected"
                    );
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

        let state = Self {
            db: Mutex::new(db),
            audio,
            speech,
            presentation: PresentationEngine::new(),
            display: DisplayManager::new(app.clone()),
            settings: Mutex::new(settings),
            media_playback: Mutex::new(crate::events::MediaPlaybackState::default()),
            paths,
        };

        // Persist repairs so they are not rediscovered on every launch.
        if settings_persist_needed {
            if let Err(err) = state.save_settings() {
                tracing::warn!(error = %err, "could not persist repaired settings");
            }
        }

        Ok(state)
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

    /// Projects an item, opening the presentation window if it is not already up.
    ///
    /// The saved "fill the whole screen" setting is used when the window has to
    /// be created, so the operator's choice is never silently overridden by a
    /// default on the way to the projector.
    pub fn project(
        &self,
        item: crate::models::presentation::PresentationItem,
        app: &AppHandle,
    ) -> Result<(), AppError> {
        let fullscreen = self
            .settings
            .lock()
            .map(|s| s.presentation.fullscreen)
            .unwrap_or(true);
        self.display.ensure_open(app, fullscreen)?;
        self.presentation
            .project(item.clone(), &self.display, app)?;

        // A new item means whatever was playing has gone: reset the playback
        // picture so the Media screen never shows "playing" for a video that is
        // no longer on the screen. The projector window reports the real state
        // as soon as it has loaded the file.
        if let Ok(mut playback) = self.media_playback.lock() {
            *playback = crate::events::MediaPlaybackState::for_item(Some(item.id));
        }
        Ok(())
    }

    /// Pushes the saved projector settings at the live presentation window.
    ///
    /// Saving Settings has to change what the congregation actually sees —
    /// background colour, text size, typeface and fullscreen — so this is the
    /// step that makes a Save real rather than just a database write:
    ///
    /// 1. the chosen screen becomes the projection target;
    /// 2. an already-open projector window follows the fullscreen setting;
    /// 3. the window is told the new look over `presentation://settings`.
    ///
    /// A stale display index (for example a projector that was unplugged) is
    /// logged and skipped; it must never block saving everything else.
    pub fn apply_presentation_settings(&self, app: &AppHandle) -> Result<(), AppError> {
        let presentation = self
            .settings
            .lock()
            .map_err(|_| AppError::Internal("settings lock poisoned".to_string()))?
            .presentation
            .clone();

        if let Some(index) = presentation.display_index {
            if let Err(err) = self.display.set_target(index) {
                tracing::warn!(error = %err, "could not select the saved screen");
            }
        }

        if self.display.is_open() {
            if let Err(err) = self.display.set_fullscreen(presentation.fullscreen) {
                tracing::warn!(error = %err, "could not change the projector's fullscreen state");
            }
        }

        if let Err(err) = app.emit(
            crate::events::PRESENTATION_SETTINGS,
            crate::events::PresentationSettingsEvent::from(&presentation),
        ) {
            tracing::warn!(error = %err, "could not tell the projector about the new settings");
        }
        Ok(())
    }
}
