//! Application models shared across modules.

pub mod content;
pub mod presentation;
pub mod settings;

pub use content::{DetectedContent, DetectionResult};
pub use presentation::{
    ContentPayload, ContentType, Presentation, PresentationItem, PresentationItemRecord,
    PresentationState,
};
pub use settings::AppSettings;
