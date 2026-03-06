// state.rs — Shared application state, wrapped in Arc<Mutex<>> for thread safety.
//
// Why Arc<Mutex<>>?
//   - Arc  = "Atomically Reference Counted" — lets multiple threads OWN the same data
//   - Mutex = "Mutual Exclusion"           — ensures only ONE thread accesses the data at a time
//
// The pattern: Arc<Mutex<InnerState>> means we can cheaply clone the handle (Arc),
// pass it to the HTTP server AND Tauri commands, and lock it only when reading/writing.

use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

/// Information about the media currently playing in Chrome.
/// `Serialize` lets us convert this to JSON; `Deserialize` lets us parse JSON into it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaInfo {
    pub title: String,
    pub artist: String,
    pub tab_id: i32,
    pub is_playing: bool,
}

/// The actual data we want to protect behind a Mutex.
/// We keep this private — callers go through AppState methods instead.
#[derive(Debug)]
pub struct InnerState {
    pub volume: f32,
    pub current_media: Option<MediaInfo>, // None = nothing playing yet
    pub is_playing: bool,
}

/// Public handle that every module uses.
/// Cheap to clone because it's just an Arc pointer — no deep copy happens.
#[derive(Clone)]
pub struct AppState {
    pub inner: Arc<Mutex<InnerState>>,
}

impl AppState {
    /// Create a fresh state with sensible defaults.
    pub fn new() -> Self {
        AppState {
            inner: Arc::new(Mutex::new(InnerState {
                volume: 1.0,           // full volume
                current_media: None,   // nothing playing yet
                is_playing: false,
            })),
        }
    }

    /// Return the current volume (0.0 – 1.0).
    pub fn get_volume(&self) -> Result<f32, String> {
        // .lock() can fail if another thread panicked while holding the lock
        let state = self.inner.lock()
            .map_err(|e| format!("Failed to acquire state lock: {}", e))?;
        Ok(state.volume)
    }

    /// Update the volume. Returns Err if the value is outside [0.0, 1.0].
    pub fn set_volume(&self, volume: f32) -> Result<(), String> {
        if !(0.0..=1.0).contains(&volume) {
            return Err(format!(
                "Volume {:.2} is out of range — must be between 0.0 and 1.0",
                volume
            ));
        }

        let mut state = self.inner.lock()
            .map_err(|e| format!("Failed to acquire state lock: {}", e))?;
        state.volume = volume;

        println!("[state] Volume updated to {:.2}", volume);
        Ok(())
    }

    /// Replace the currently tracked media with new information from the extension.
    pub fn update_media(&self, media: MediaInfo) -> Result<(), String> {
        let mut state = self.inner.lock()
            .map_err(|e| format!("Failed to acquire state lock: {}", e))?;

        state.is_playing = media.is_playing;
        state.current_media = Some(media);

        Ok(())
    }

    /// Flip the playback state and return the new value.
    pub fn toggle_playback(&self) -> Result<bool, String> {
        let mut state = self.inner.lock()
            .map_err(|e| format!("Failed to acquire state lock: {}", e))?;

        state.is_playing = !state.is_playing;

        // Copy the new value into a local *before* the mutable borrow of
        // state.current_media, so the borrow checker is happy.
        let new_is_playing = state.is_playing;

        // Keep the MediaInfo in sync so the UI reflects the change
        if let Some(ref mut media) = state.current_media {
            media.is_playing = new_is_playing;
        }

        println!("[state] Playback toggled — is_playing = {}", state.is_playing);
        Ok(state.is_playing)
    }

    /// Serialize the whole state to a JSON value (for the /api/status endpoint and
    /// the `get_current_media` Tauri command).
    pub fn to_json(&self) -> Result<serde_json::Value, String> {
        let state = self.inner.lock()
            .map_err(|e| format!("Failed to acquire state lock: {}", e))?;

        Ok(serde_json::json!({
            "volume":        state.volume,
            "is_playing":    state.is_playing,
            "current_media": state.current_media, // serializes to null when None
        }))
    }
}
