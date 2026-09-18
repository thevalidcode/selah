//! Display manager.
//!
//! Isolates multi-monitor logic: discovering monitors, choosing the
//! presentation target, and opening/closing/fullscreening the presentation
//! window. Nothing here knows about Bible content or live audio.

use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Monitor};

use crate::errors::AppError;
use crate::events::{self, PresentationDisplayEvent};

/// Label of the dedicated presentation window.
pub const PRESENTATION_WINDOW_LABEL: &str = "presentation";

/// A monitor available to the operating system.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayInfo {
    pub index: usize,
    pub name: Option<String>,
    pub size: (u32, u32),
    pub position: (i32, i32),
    pub scale_factor: f64,
    pub is_primary: bool,
}

pub struct DisplayManager {
    app: AppHandle,
    target: Mutex<Option<usize>>,
}

impl DisplayManager {
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            target: Mutex::new(None),
        }
    }

    /// Lists monitors reported by the OS, in a stable order.
    pub fn list_displays(&self) -> Result<Vec<DisplayInfo>, AppError> {
        let monitors = self.available_monitors()?;
        let primary = self.primary_monitor_index(&monitors);
        Ok(monitors
            .into_iter()
            .enumerate()
            .map(|(index, monitor)| DisplayInfo {
                index,
                name: monitor.name().cloned(),
                size: {
                    let size = monitor.size();
                    (size.width, size.height)
                },
                position: {
                    let pos = monitor.position();
                    (pos.x, pos.y)
                },
                scale_factor: monitor.scale_factor(),
                is_primary: primary == Some(index),
            })
            .collect())
    }

    /// Selects which monitor the presentation window should open on.
    pub fn set_target(&self, index: usize) -> Result<(), AppError> {
        let displays = self.list_displays()?;
        if !displays.iter().any(|d| d.index == index) {
            return Err(AppError::DisplayNotFound(format!(
                "no display with index {index}"
            )));
        }
        *self
            .target
            .lock()
            .map_err(|_| AppError::Internal("display target lock poisoned".to_string()))? =
            Some(index);
        Ok(())
    }

    /// Currently selected target display, if any.
    pub fn target(&self) -> Option<usize> {
        self.target.lock().ok().and_then(|g| *g)
    }

    /// Whether the presentation window currently exists.
    pub fn is_open(&self) -> bool {
        self.app
            .get_webview_window(PRESENTATION_WINDOW_LABEL)
            .is_some()
    }

    /// Opens the presentation window on the target display (or the primary
    /// display if no target is selected).
    pub fn ensure_open(&self, app: &AppHandle, fullscreen: bool) -> Result<(), AppError> {
        if self.is_open() {
            return Ok(());
        }
        self.open(app, fullscreen).map(|_| ())
    }

    /// Opens the presentation window, returning its display.
    pub fn open(&self, app: &AppHandle, fullscreen: bool) -> Result<DisplayInfo, AppError> {
        if self.is_open() {
            return self.current_display();
        }

        let displays = self.list_displays()?;
        if displays.is_empty() {
            return Err(AppError::DisplayNotFound(
                "no displays available".to_string(),
            ));
        }

        let target = self.target().unwrap_or(displays[0].index);
        let display = displays
            .iter()
            .find(|d| d.index == target)
            .or(displays.first())
            .ok_or_else(|| AppError::DisplayNotFound("no displays available".to_string()))?
            .clone();

        let scale = display.scale_factor.max(1.0);
        let position = tauri::PhysicalPosition {
            x: display.position.0,
            y: display.position.1,
        }
        .to_logical::<f64>(scale);
        let size = tauri::PhysicalSize {
            width: display.size.0,
            height: display.size.1,
        }
        .to_logical::<f64>(scale);

        tauri::WebviewWindowBuilder::new(
            app,
            PRESENTATION_WINDOW_LABEL,
            tauri::WebviewUrl::App("presentation.html".into()),
        )
        .title("Selah Presentation")
        .decorations(false)
        .resizable(false)
        .position(position.x, position.y)
        .inner_size(size.width, size.height)
        .fullscreen(fullscreen)
        .build()
        .map_err(|e| AppError::Presentation(format!("failed to open presentation window: {e}")))?;

        let _ = self.app.emit(
            events::PRESENTATION_DISPLAY_OPENED,
            PresentationDisplayEvent {
                open: true,
                display: display.name.clone(),
            },
        );

        Ok(display)
    }

    /// Closes the presentation window if it exists.
    pub fn close(&self) -> Result<(), AppError> {
        if let Some(window) = self.app.get_webview_window(PRESENTATION_WINDOW_LABEL) {
            window.close().map_err(|e| {
                AppError::Presentation(format!("failed to close presentation window: {e}"))
            })?;
            let _ = self.app.emit(
                events::PRESENTATION_DISPLAY_CLOSED,
                PresentationDisplayEvent {
                    open: false,
                    display: None,
                },
            );
        }
        Ok(())
    }

    /// Toggles fullscreen on the presentation window, if it exists.
    pub fn set_fullscreen(&self, fullscreen: bool) -> Result<(), AppError> {
        let Some(window) = self.app.get_webview_window(PRESENTATION_WINDOW_LABEL) else {
            return Err(AppError::DisplayNotFound(
                "presentation window is not open".to_string(),
            ));
        };
        window
            .set_fullscreen(fullscreen)
            .map_err(|e| AppError::Presentation(format!("failed to set fullscreen: {e}")))?;
        Ok(())
    }

    fn current_display(&self) -> Result<DisplayInfo, AppError> {
        let target = self.target().unwrap_or(0);
        let displays = self.list_displays()?;
        displays
            .into_iter()
            .find(|d| d.index == target)
            .ok_or_else(|| AppError::DisplayNotFound("no displays available".to_string()))
    }

    fn available_monitors(&self) -> Result<Vec<Monitor>, AppError> {
        let Some(window) = self.app.get_webview_window("main") else {
            return Err(AppError::DisplayNotFound(
                "main window is not available yet".to_string(),
            ));
        };
        window
            .available_monitors()
            .map_err(|e| AppError::DisplayNotFound(format!("cannot list monitors: {e}")))
    }

    fn primary_monitor_index(&self, monitors: &[Monitor]) -> Option<usize> {
        let primary = self
            .app
            .get_webview_window("main")
            .and_then(|w| w.primary_monitor().ok().flatten())?;
        monitors.iter().position(|m| {
            m.name() == primary.name()
                && m.position() == primary.position()
                && m.size() == primary.size()
        })
    }
}
