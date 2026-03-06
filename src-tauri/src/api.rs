// api.rs — Axum HTTP server that the Chrome extension talks to.
//
// Architecture:
//   Chrome extension  ──POST /api/volume──►  Axum handler  ──► AppState (shared with Tauri)
//   Chrome extension  ──POST /api/media───►  Axum handler  ──► AppState
//   Chrome extension  ──GET  /api/status──►  Axum handler  ──► AppState (read-only)
//
// Axum handlers are async functions; Axum extracts their arguments automatically
// (JSON body, shared state, etc.) based on the type signature — this is called
// "extractors" in Axum terminology.

use axum::{
    extract::State as AxumState, // "State" extractor — injects our AppState into handlers
    routing::{get, post},
    Json,
    Router,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

use crate::state::{AppState, MediaInfo};

// ─── Request / Response types ─────────────────────────────────────────────────

/// Body Chrome extension sends to POST /api/volume
#[derive(Deserialize)]
pub struct VolumeRequest {
    pub volume: f32,
    pub tab_id: Option<i32>, // Not used server-side yet, but logged for debugging
}

/// Body Chrome extension sends to POST /api/media
#[derive(Deserialize)]
pub struct MediaRequest {
    pub title: String,
    pub artist: String,
    pub tab_id: i32,
    pub is_playing: bool,
}

/// Generic JSON response sent back to the extension
#[derive(Serialize)]
pub struct ApiResponse {
    pub success: bool,
    pub message: String,
}

// ─── Handlers ─────────────────────────────────────────────────────────────────

/// GET /api/status — returns the full current state as JSON.
async fn get_status(
    AxumState(state): AxumState<AppState>,
) -> Json<serde_json::Value> {
    match state.to_json() {
        Ok(json) => Json(json),
        Err(e) => Json(serde_json::json!({ "error": e })),
    }
}

/// POST /api/volume — update the volume stored in AppState.
///
/// Expected body: { "volume": 0.5, "tab_id": 123 }
async fn set_volume_handler(
    AxumState(state): AxumState<AppState>,
    Json(payload): Json<VolumeRequest>,
) -> Json<ApiResponse> {
    println!(
        "[api] Volume request: {:.2} (tab: {:?})",
        payload.volume, payload.tab_id
    );

    match state.set_volume(payload.volume) {
        Ok(()) => Json(ApiResponse {
            success: true,
            message: format!(
                "Volume set to {:.2} for tab {:?}",
                payload.volume, payload.tab_id
            ),
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            message: e,
        }),
    }
}

/// POST /api/media — update the currently playing media info.
///
/// Expected body: { "title": "Song", "artist": "Artist", "tab_id": 123, "is_playing": true }
async fn update_media_handler(
    AxumState(state): AxumState<AppState>,
    Json(payload): Json<MediaRequest>,
) -> Json<ApiResponse> {
    println!(
        "[api] Media update: \"{}\" by \"{}\" (tab: {}, playing: {})",
        payload.title, payload.artist, payload.tab_id, payload.is_playing
    );

    let media = MediaInfo {
        title: payload.title,
        artist: payload.artist,
        tab_id: payload.tab_id,
        is_playing: payload.is_playing,
    };

    match state.update_media(media) {
        Ok(()) => Json(ApiResponse {
            success: true,
            message: "Media info updated".to_string(),
        }),
        Err(e) => Json(ApiResponse {
            success: false,
            message: e,
        }),
    }
}

// ─── Server startup ───────────────────────────────────────────────────────────

/// Build and run the Axum HTTP server.
/// This function runs forever (until the process exits), so call it in a background thread.
pub async fn start_server(state: AppState) {
    let app = Router::new()
        // Health / status endpoint — useful for the extension to check if the app is running
        .route("/api/status", get(get_status))
        // Volume endpoint — extension calls this when the slider changes
        .route("/api/volume", post(set_volume_handler))
        // Media endpoint — extension calls this when the playing track changes
        .route("/api/media", post(update_media_handler))
        // Inject AppState into every handler
        .with_state(state)
        // Allow requests from any origin — necessary for an unpacked Chrome extension
        // (its origin is something like chrome-extension://<id>)
        .layer(CorsLayer::permissive());

    // Bind to localhost only — the extension talks locally, no public exposure needed
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3030")
        .await
        .expect("Failed to bind HTTP server to 127.0.0.1:3030 — is port 3030 already in use?");

    println!("[api] HTTP server listening on http://localhost:3030");

    axum::serve(listener, app)
        .await
        .expect("HTTP server encountered a fatal error");
}
