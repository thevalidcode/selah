//! Media library: files on disk, metadata in SQLite.
//!
//! - [`library`] browses folders and registers the media inside them.
//! - [`access`] grants the projector window read access to the folder the
//!   operator chose (never to the whole disk, and never to a hardcoded path).

pub mod access;
mod library;

pub use access::{grant_directory, grant_file, is_allowed};
pub use library::{
    kind_for_path, DirectoryEntry, DirectoryListing, MediaItem, MediaLibrary, MediaScan,
};
