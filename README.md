# Desktop Media Controller

A Tauri 2.0 desktop app for displaying and controlling media playing in your browser, built with Rust and TypeScript. Works in tandem with a companion browser extension running on `localhost:3030`.

## Features

- 🎵 Displays currently playing track title and artist from the browser
- 🔊 Volume slider synced in real time with the browser extension
- ⏯ Play/Pause control that remotely commands the browser tab
- 🖥 Lives in the system tray — show/hide the window at any time
- 🔗 Browser extension integration:
  - Receives media info pushed from the extension every 2s
  - Serves volume and play/pause commands pulled by the extension every 500ms
  - Command lock prevents the extension from overwriting server state immediately after a remote command
  - https://github.com/WaltteriWe/chrome-volume-controller

## Tech Stack

- Rust + Tauri 2.0
- Axum (HTTP server on `localhost:3030`)
- TypeScript + Vite (frontend UI)

## Extension API

The app exposes a local server at `http://localhost:3030` that the browser extension talks to.

| Endpoint | Method | Description |
|---|---|---|
| `/api/status` | GET | Returns `{ volume, is_playing, current_media }` — polled by the extension |
| `/api/media` | POST | Receives current track title, artist, and playback state from the extension |
| `/api/volume` | POST | Receives volume changes from the extension popup |

**GET /api/status response**
```json
{
  "volume": 0.75,
  "is_playing": true,
  "current_media": {
    "title": "Song Name",
    "artist": "Artist Name",
    "tab_id": 0,
    "is_playing": true
  }
}
```

## Build

```bash
npm install
npm run tauri dev   # development (hot reload)
npm run tauri build # production
```

Requires Rust, Node.js, and the [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your OS. The HTTP server starts automatically on `localhost:3030` alongside the app window.
