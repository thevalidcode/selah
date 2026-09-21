//! Media commands: folder browsing, library listing and projection.
//!
//! Media is loaded from a folder chosen at runtime — no path is baked into the
//! application — so the commands here are "look at this folder", "load this
//! folder" and "put this file on the screen".

use serde::Deserialize;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};

use crate::errors::{AppError, CommandResult};
use crate::media::{DirectoryListing, MediaItem, MediaLibrary, MediaScan};
use crate::models::presentation::{MediaClip, PresentationItem, PresentationState};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportMediaRequest {
    pub path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadMediaDirectoryRequest {
    pub path: String,
    /// Whether sub-folders are read as well.
    #[serde(default)]
    pub recursive: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMediaRequest {
    pub path: String,
    /// Heading shown with the file; falls back to the file name.
    #[serde(default)]
    pub title: Option<String>,
    /// The part of the video to show, and whether it starts again at the end.
    /// Omitted when the operator simply pressed Show on a file with no range.
    #[serde(default)]
    pub clip: Option<MediaClipRequest>,
}

/// A time range chosen on the Media screen.
///
/// Milliseconds, because that is what a `<video>` element reports and what the
/// sliders in the preview actually produce; converting to seconds would lose
/// the frame the operator was looking at.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaClipRequest {
    /// Where playback begins. Absent means the beginning of the file.
    #[serde(default)]
    pub start_ms: u64,
    /// Where playback ends. Absent runs to the end of the file.
    #[serde(default)]
    pub end_ms: Option<u64>,
    /// Whether it starts again instead of stopping. Absent follows the saved
    /// Screen setting, which is what a plain Show on a video does.
    #[serde(default)]
    pub repeat: Option<bool>,
}

impl MediaClipRequest {
    /// The clip as the presentation engine wants it.
    ///
    /// `repeat_default` is used when the request did not carry a choice, so the
    /// "stop or repeat" behaviour is the same whether the operator opened the
    /// preview or just pressed Show.
    fn into_clip(self, repeat_default: bool) -> MediaClip {
        MediaClip {
            start_ms: self.start_ms,
            end_ms: self.end_ms,
            repeat: self.repeat.unwrap_or(repeat_default),
        }
        .sanitized()
    }
}

#[tauri::command]
pub fn list_media(state: State<'_, AppState>) -> CommandResult<Vec<MediaItem>> {
    state.with_conn(crate::media::MediaLibrary::list)
}

/// Lists a folder so the operator can walk to the folder they want.
///
/// `path` omitted starts at the user's home folder.
#[tauri::command]
pub fn browse_directory(path: Option<String>) -> CommandResult<DirectoryListing> {
    let path = path.map(PathBuf::from);
    MediaLibrary::browse(path.as_deref())
}

/// Reads a folder and registers the media inside it.
///
/// The folder is remembered in settings (and granted to the projector window)
/// so the same library is available on the next launch without picking it again.
#[tauri::command]
pub fn load_media_directory(
    request: LoadMediaDirectoryRequest,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<MediaScan> {
    let requested = PathBuf::from(&request.path);
    let directory = requested
        .canonicalize()
        .map_err(|e| AppError::Media(format!("cannot open {}: {e}", requested.display())))?;

    // Open the folder to the projector window before anything is registered,
    // otherwise the pictures would list but never draw.
    crate::media::grant_directory(&app, &directory)?;

    let scan = state.with_conn(|conn| MediaLibrary::scan(conn, &directory, request.recursive))?;

    {
        let mut settings = state
            .settings
            .lock()
            .map_err(|_| AppError::Internal("settings lock poisoned".to_string()))?;
        settings.media.directory = Some(directory.to_string_lossy().to_string());
        settings.media.recursive = request.recursive;
    }
    state.save_settings()?;

    Ok(scan)
}

/// Registers a single file (used when a file is added by hand).
#[tauri::command]
pub fn import_media(
    request: ImportMediaRequest,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<MediaItem> {
    let path = PathBuf::from(&request.path);
    let item = state.with_conn(|conn| crate::media::MediaLibrary::import(conn, path.clone()))?;
    // Make sure the projector window may read it.
    crate::media::grant_file(&app, &path)?;
    Ok(item)
}

#[tauri::command]
pub fn remove_media(id: String, state: State<'_, AppState>) -> CommandResult<()> {
    state.with_conn(|conn| crate::media::MediaLibrary::remove(conn, &id))
}

/// Puts a media file on the congregation's screen.
///
/// The projector window is a separate webview, so the file is handed over as an
/// `asset:` URL by the frontend; this command checks the file is real and that
/// Selah can present it, then projects it.
#[tauri::command]
pub fn project_media(
    request: ProjectMediaRequest,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<PresentationState> {
    let path = PathBuf::from(&request.path);
    if !path.is_file() {
        return Err(AppError::Media(format!(
            "file does not exist: {}",
            path.display()
        )));
    }
    let kind = crate::media::kind_for_path(&path).ok_or_else(|| {
        AppError::Media(format!(
            "Selah cannot present {}",
            path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.display().to_string())
        ))
    })?;

    // A file added by hand may live outside the current media folder, so open
    // it explicitly before projecting.
    crate::media::grant_file(&app, &path)?;

    let title = request
        .title
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(str::to_string)
        .or_else(|| path.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "Media".to_string());

    // Only print a caption when one was given: a picture or video should fill
    // the screen without its file name written over it.
    let caption = request
        .title
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty());

    // Video is the one kind that can be a chosen part rather than a whole file.
    // The saved Screen setting fills in the "stop or repeat" answer when the
    // caller did not make a choice, so Show on a video behaves the same as the
    // preview did.
    let clip = request
        .clip
        .map(|clip| clip.into_clip(repeat_videos_default(&state)));

    let item =
        PresentationItem::media_clipped(title, request.path, Some(kind.to_string()), caption, clip);
    state.project(item, &app)?;
    Ok(state.presentation.state())
}

/// Whether video repeats by default, from the saved Screen settings.
///
/// On unless the operator turned it off, so a settings document that predates
/// the choice — or a locked settings mutex — cannot make video behave worse
/// than it used to.
fn repeat_videos_default(state: &State<'_, AppState>) -> bool {
    state
        .settings
        .lock()
        .map(|settings| settings.presentation.repeat_videos)
        .unwrap_or_else(|_| crate::models::settings::default_repeat_videos())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetMediaClipRequest {
    /// The library record the range belongs to.
    pub id: String,
    /// The range and repeat choice. Omit it (or send `null`) to forget the
    /// range and go back to playing the file whole.
    #[serde(default)]
    pub clip: Option<MediaClipRequest>,
}

/// Remembers which part of a video to show, for next time.
///
/// The choice lives in the media record's metadata, so reopening Selah — or
/// pressing Show on the file later — brings the same range back without the
/// operator setting it up again.
#[tauri::command]
pub fn set_media_clip(
    request: SetMediaClipRequest,
    state: State<'_, AppState>,
) -> CommandResult<MediaItem> {
    // The preview always sends its switch, so the fallback only matters for a
    // hand-made call: the saved setting is the sensible answer.
    let repeat_default = repeat_videos_default(&state);
    let clip = request.clip.map(|clip| clip.into_clip(repeat_default));
    state.with_conn(|conn| MediaLibrary::set_clip(conn, &request.id, clip))
}

// ---------------------------------------------------------
// Playback control (videos, and music)
// ---------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackRequest {
    /// `play`, `pause`, `restart` or `stop`.
    pub action: String,
}

/// The last known playback state, for a screen that has just opened.
#[tauri::command]
pub fn get_media_playback_state(state: State<'_, AppState>) -> CommandResult<PlaybackSnapshot> {
    Ok(snapshot(&state))
}

/// Tells the projector window to play, pause, restart or stop what is on it.
///
/// The action is sent as an event rather than a direct call because the
/// projector is a separate webview — only it can touch the `<video>` element.
/// The action names the item it is for, so a button pressed a moment too late
/// cannot affect the next file.
#[tauri::command]
pub fn control_media_playback(
    app: AppHandle,
    state: State<'_, AppState>,
    request: PlaybackRequest,
) -> CommandResult<PlaybackSnapshot> {
    let action = crate::events::PlaybackAction::parse(&request.action).ok_or_else(|| {
        AppError::InvalidConfiguration(format!(
            "{} is not something a player can do (try play, pause, restart or stop)",
            request.action
        ))
    })?;

    let item_id = state.presentation.state().current.map(|item| item.id);

    if item_id.is_none() {
        return Err(AppError::Presentation(
            "nothing is on the screen to play".to_string(),
        ));
    }

    if let Err(err) = app.emit(
        crate::events::MEDIA_PLAYBACK,
        crate::events::MediaPlaybackCommand {
            action,
            item_id: item_id.clone(),
        },
    ) {
        tracing::warn!(error = %err, "could not reach the projector about playback");
    }

    // Optimistically update the held state so the button responds immediately;
    // the projector's report replaces it with the truth in a moment.
    if let Ok(mut playback) = state.media_playback.lock() {
        if playback.item_id == item_id {
            match action {
                crate::events::PlaybackAction::Play => playback.playing = true,
                crate::events::PlaybackAction::Pause => playback.playing = false,
                crate::events::PlaybackAction::Restart => {
                    playback.playing = true;
                    playback.position_ms = 0;
                    playback.ended = false;
                }
                crate::events::PlaybackAction::Stop => {
                    playback.playing = false;
                    playback.position_ms = 0;
                    playback.ended = false;
                }
            }
        }
    }

    Ok(snapshot(&state))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackReport {
    pub playing: bool,
    #[serde(default)]
    pub position_ms: u64,
    #[serde(default)]
    pub duration_ms: u64,
    #[serde(default)]
    pub ended: bool,
}

/// Where the operator screen reads playback from.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackSnapshot {
    /// The item on screen, if any.
    pub item_id: Option<String>,
    /// True when that item is a picture or a video (music has no picture).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_kind: Option<String>,
    pub playing: bool,
    pub position_ms: u64,
    pub duration_ms: u64,
    pub ended: bool,
}

/// Called by the projector window with what its player is actually doing.
///
/// This is the only trustworthy source for "is it playing": the operator screen
/// cannot see the congregation's screen, so it is told rather than guessing.
#[tauri::command]
pub fn report_media_playback(
    app: AppHandle,
    state: State<'_, AppState>,
    report: PlaybackReport,
) -> CommandResult<PlaybackSnapshot> {
    let current = state.presentation.state().current;

    if let Ok(mut playback) = state.media_playback.lock() {
        // Only accept a report about the item that is really on screen, so a
        // late report from a file that has been replaced cannot flip the badge.
        if current
            .as_ref()
            .is_some_and(|item| playback.describes(&item.id))
        {
            playback.playing = report.playing;
            playback.position_ms = report.position_ms;
            playback.duration_ms = report.duration_ms;
            playback.ended = report.ended;
        }
    }

    let snapshot = snapshot(&state);
    if let Err(err) = app.emit(crate::events::MEDIA_PLAYBACK_STATE, &snapshot) {
        tracing::debug!(error = %err, "no listener for the playback report");
    }
    Ok(snapshot)
}

/// Builds the snapshot the Media screen renders.
fn snapshot(state: &AppState) -> PlaybackSnapshot {
    let current = state.presentation.state().current;
    let media_kind = current.as_ref().and_then(|item| match &item.payload {
        crate::models::presentation::ContentPayload::Media { media_kind, .. } => media_kind.clone(),
        _ => None,
    });

    let playback = state
        .media_playback
        .lock()
        .map(|playback| playback.clone())
        .unwrap_or_default();

    let describes_current = current
        .as_ref()
        .is_some_and(|item| playback.describes(&item.id));

    PlaybackSnapshot {
        item_id: current.map(|item| item.id),
        media_kind,
        playing: describes_current && playback.playing,
        position_ms: if describes_current {
            playback.position_ms
        } else {
            0
        },
        duration_ms: if describes_current {
            playback.duration_ms
        } else {
            0
        },
        ended: describes_current && playback.ended,
    }
}
