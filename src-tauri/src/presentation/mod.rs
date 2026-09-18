//! Presentation engine and multi-monitor display management.

pub mod display;
pub mod engine;
pub mod state;

pub use display::{DisplayInfo, DisplayManager, PRESENTATION_WINDOW_LABEL};
pub use engine::PresentationEngine;
