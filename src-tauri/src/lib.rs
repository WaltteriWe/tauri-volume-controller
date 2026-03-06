// lib.rs — Application entry point and Tauri command definitions.
//
// This crate is structured across four modules:
//   state  — AppState struct (shared data, thread-safe)
//   api    — Axum HTTP server (Chrome extension communicates here)
//   tray   — System tray icon and menu
//   lib    — Tauri commands + `run()` startup function  (this file)

mod api;
mod state;
mod tray;

use state::AppState;

// ─── Tauri Commands ───────────────────────────────────────────────────────────
//
// Commands are called from the TypeScript frontend via `invoke("command_name", args)`.
// They receive a `tauri::State<T>` extractor to access our shared AppState.

/// Return a JSON snapshot of the current state (volume, is_playing, current_media).
/// Callable from TS: invoke("get_current_media")
#[tauri::command]
fn get_current_media(
    state: tauri::State<AppState>,
) -> Result<serde_json::Value, String> {
    state.to_json()
}

/// Set the playback volume (0.0 – 1.0).
/// Callable from TS: invoke("set_volume", { volume: 0.5 })
#[tauri::command]
fn set_volume(
    volume: f32,
    state: tauri::State<AppState>,
) -> Result<String, String> {
    state.set_volume(volume)?;
    Ok(format!("Volume set to {:.2}", volume))
}

/// Toggle play / pause and return the new is_playing value.
/// Callable from TS: invoke("toggle_play_pause")
#[tauri::command]
fn toggle_play_pause(
    state: tauri::State<AppState>,
) -> Result<bool, String> {
    state.toggle_playback()
}

// ─── Application startup ──────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Create the shared state once; it will be cloned (cheaply, via Arc) for both
    // the HTTP server and the Tauri command layer.
    let app_state = AppState::new();

    // Clone the handle *before* moving app_state into Tauri's .manage()
    let api_state = app_state.clone();

    // Spawn a dedicated OS thread for the HTTP server.
    //
    // Why a thread instead of tokio::spawn?
    //   tauri::Builder::default().run() blocks the main thread.  We cannot await
    //   it, so we give the HTTP server its own Tokio runtime on a separate thread.
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new()
            .expect("Failed to create Tokio runtime for the HTTP server");
        rt.block_on(api::start_server(api_state));
    });

    // Build and run the Tauri application on the main thread (required by most OS).
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // Register our shared state so commands can access it via tauri::State<AppState>
        .manage(app_state)
        // Register all Tauri commands defined above
        .invoke_handler(tauri::generate_handler![
            get_current_media,
            set_volume,
            toggle_play_pause,
        ])
        // .setup() runs once after the app is initialised but before the window shows —
        // the perfect place to create the tray icon
        .setup(|app| {
            tray::setup_tray(&app.handle())
                .map_err(|e| format!("Failed to set up system tray: {}", e))?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
