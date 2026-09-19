//! Song library: songs made of ordered sections (verses, choruses, bridges).
//!
//! Songs are presented one section at a time, so this module deliberately
//! mirrors the shape of `bible::models::Verse` rather than storing a song as a
//! single block of text.

pub mod models;
pub mod repository;

pub use models::{Song, SongInput, SongSection, SongSectionInput};
pub use repository::SongRepository;
