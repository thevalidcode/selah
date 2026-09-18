//! Presentation engine.
//!
//! Decides HOW content is displayed. Content itself (Bible passages, media,
//! text) comes from other modules — the engine only manages the projection
//! queue and pushes state changes to the frontend.

use std::sync::Mutex;

use tauri::{AppHandle, Emitter};

use crate::errors::AppError;
use crate::events::{self, PresentationChangedEvent};
use crate::models::presentation::{PresentationItem, PresentationState};

use super::display::DisplayManager;

pub struct PresentationEngine {
    state: Mutex<PresentationState>,
}

impl Default for PresentationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PresentationEngine {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(PresentationState::empty()),
        }
    }

    /// Projects `item` and notifies the frontend + presentation window.
    pub fn project(
        &self,
        item: PresentationItem,
        display: &DisplayManager,
        app: &AppHandle,
    ) -> Result<(), AppError> {
        {
            let mut state = self
                .state
                .lock()
                .map_err(|_| AppError::Internal("presentation state lock poisoned".to_string()))?;
            state.set_current(item.clone());
        }
        display.ensure_open(app, true)?;
        self.emit_changed(app, display)
    }

    /// Clears the projector.
    pub fn clear(&self, display: &DisplayManager, app: &AppHandle) -> Result<(), AppError> {
        {
            let mut state = self
                .state
                .lock()
                .map_err(|_| AppError::Internal("presentation state lock poisoned".to_string()))?;
            state.clear();
        }
        let _ = app.emit(
            events::PRESENTATION_CHANGED,
            PresentationChangedEvent {
                item: None,
                projected: display.is_open(),
            },
        );
        Ok(())
    }

    pub fn show_next(&self, app: &AppHandle) -> Result<(), AppError> {
        let advanced = {
            let mut state = self
                .state
                .lock()
                .map_err(|_| AppError::Internal("presentation state lock poisoned".to_string()))?;
            state.show_next()
        };
        if advanced {
            tracing::info!("presentation advanced");
            let _ = app.emit(
                events::PRESENTATION_CHANGED,
                PresentationChangedEvent {
                    item: self.state().current,
                    projected: true,
                },
            );
        }
        Ok(())
    }

    pub fn show_previous(&self, app: &AppHandle) -> Result<(), AppError> {
        let went_back = {
            let mut state = self
                .state
                .lock()
                .map_err(|_| AppError::Internal("presentation state lock poisoned".to_string()))?;
            state.show_previous()
        };
        if went_back {
            let _ = app.emit(
                events::PRESENTATION_CHANGED,
                PresentationChangedEvent {
                    item: self.state().current,
                    projected: true,
                },
            );
        }
        Ok(())
    }

    pub fn queue(&self, item: PresentationItem) -> Result<(), AppError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| AppError::Internal("presentation state lock poisoned".to_string()))?;
        state.queue.push(item);
        Ok(())
    }

    pub fn state(&self) -> PresentationState {
        self.state.lock().map(|g| g.clone()).unwrap_or_else(|_| {
            tracing::warn!("presentation state lock poisoned; returning empty state");
            PresentationState::empty()
        })
    }

    fn emit_changed(&self, app: &AppHandle, display: &DisplayManager) -> Result<(), AppError> {
        let _ = app.emit(
            events::PRESENTATION_CHANGED,
            PresentationChangedEvent {
                item: self.state().current,
                projected: display.is_open(),
            },
        );
        Ok(())
    }
}
