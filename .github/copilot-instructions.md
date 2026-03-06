# Copilot Instructions - Rust & Tauri 2.0 Development

## Overview
This project uses Rust for the Tauri backend. The developer is learning Rust, so code should be clear, well-commented, and follow best practices.

---

## Rust Fundamentals

### Variable Declaration
```rust
// Immutable by default (preferred)
let name = "value";

// Mutable (use sparingly)
let mut counter = 0;
counter += 1;

// Type annotations when needed
let age: i32 = 25;
let price: f32 = 9.99;
```

### Ownership Rules
- Each value has ONE owner
- When owner goes out of scope, value is dropped
- Use references (&) to borrow without taking ownership
```rust
// Taking ownership (value moves)
let s1 = String::from("hello");
let s2 = s1; // s1 is no longer valid

// Borrowing (reference)
let s1 = String::from("hello");
let len = calculate_length(&s1); // s1 still valid
```

### Error Handling
- ALWAYS use `Result<T, E>` for functions that can fail
- Use `?` operator to propagate errors
- Avoid `.unwrap()` in production code
```rust
// Good - returns Result
fn read_file(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))
}

// Bad - panics on error
fn read_file_bad(path: &str) -> String {
    std::fs::read_to_string(path).unwrap() // AVOID THIS
}
```

---

## Tauri 2.0 Specific

### Commands
- Use `#[tauri::command]` attribute
- Return `Result<T, String>` for error handling
- Keep commands simple and focused
```rust
#[tauri::command]
fn set_volume(volume: f32) -> Result<String, String> {
    if volume < 0.0 || volume > 1.0 {
        return Err("Volume must be between 0 and 1".to_string());
    }
    
    println!("Setting volume to: {}", volume);
    Ok(format!("Volume set to {}", volume))
}

#[tauri::command]
async fn async_operation() -> Result<String, String> {
    // Use async for long-running operations
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    Ok("Done".to_string())
}
```

### State Management
- Use `tauri::State` for shared application state
- Wrap in `Mutex` or `RwLock` for thread safety
```rust
use std::sync::Mutex;
use tauri::State;

struct AppState {
    current_volume: Mutex<f32>,
}

#[tauri::command]
fn get_volume(state: State<AppState>) -> Result<f32, String> {
    let volume = state.current_volume.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    Ok(*volume)
}

fn main() {
    let state = AppState {
        current_volume: Mutex::new(1.0),
    };
    
    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![get_volume])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Window Management
```rust
use tauri::Manager;

#[tauri::command]
fn show_window(window: tauri::Window) -> Result<(), String> {
    window.show().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn hide_window(window: tauri::Window) -> Result<(), String> {
    window.hide().map_err(|e| e.to_string())
}
```

---

## HTTP Server (Axum)

### Basic Setup
```rust
use axum::{
    routing::{get, post},
    Router, Json,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

#[derive(Deserialize)]
struct VolumeRequest {
    volume: f32,
    tab_id: Option<i32>,
}

#[derive(Serialize)]
struct ApiResponse {
    success: bool,
    message: String,
}

async fn set_volume_handler(
    Json(payload): Json<VolumeRequest>,
) -> Json<ApiResponse> {
    println!("Volume: {}, Tab: {:?}", payload.volume, payload.tab_id);
    
    Json(ApiResponse {
        success: true,
        message: format!("Volume set to {}", payload.volume),
    })
}

async fn health_check() -> &'static str {
    "OK"
}

pub async fn start_server() {
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/volume", post(set_volume_handler))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3030")
        .await
        .unwrap();
    
    println!("Server running on http://localhost:3030");
    
    axum::serve(listener, app)
        .await
        .unwrap();
}
```

### Shared State with HTTP Server
```rust
use std::sync::Arc;
use axum::extract::State as AxumState;

#[derive(Clone)]
struct ServerState {
    volume: Arc<Mutex<f32>>,
}

async fn set_volume_with_state(
    AxumState(state): AxumState<ServerState>,
    Json(payload): Json<VolumeRequest>,
) -> Json<ApiResponse> {
    let mut volume = state.volume.lock().unwrap();
    *volume = payload.volume;
    
    Json(ApiResponse {
        success: true,
        message: format!("Volume updated to {}", payload.volume),
    })
}

pub async fn start_server_with_state() {
    let state = ServerState {
        volume: Arc::new(Mutex::new(1.0)),
    };
    
    let app = Router::new()
        .route("/api/volume", post(set_volume_with_state))
        .with_state(state)
        .layer(CorsLayer::permissive());
    
    // ... rest of setup
}
```

---

## Common Patterns

### Async/Await
```rust
// Mark function as async
async fn fetch_data() -> Result<String, String> {
    // .await on async operations
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    Ok("Data".to_string())
}

// Spawn background tasks
tokio::spawn(async {
    loop {
        println!("Background task running");
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
});
```

### Pattern Matching
```rust
// Match with Result
match some_result {
    Ok(value) => println!("Success: {}", value),
    Err(e) => eprintln!("Error: {}", e),
}

// Match with Option
match maybe_value {
    Some(v) => println!("Value: {}", v),
    None => println!("No value"),
}

// if let for single pattern
if let Some(v) = maybe_value {
    println!("Value: {}", v);
}
```

### Iterators
```rust
// Collect to Vec
let numbers: Vec<i32> = (0..10).collect();

// Map and filter
let doubled: Vec<i32> = numbers.iter()
    .filter(|&n| n % 2 == 0)
    .map(|&n| n * 2)
    .collect();

// For each
numbers.iter().for_each(|n| println!("{}", n));
```

---

## Cargo.toml Dependencies

### Essential for Tauri HTTP Server
```toml
[dependencies]
tauri = { version = "2.0", features = [] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# HTTP server
axum = "0.7"
tokio = { version = "1", features = ["full"] }
tower-http = { version = "0.5", features = ["cors"] }

# System tray (if needed)
tauri-plugin-notification = "2.0"
```

---

## Code Style Guidelines

### Comments
- Use `//` for single-line comments
- Use `///` for documentation comments (above functions)
- Document WHY, not WHAT (code should be self-documenting)
```rust
/// Sets the application volume to the specified level.
/// 
/// # Arguments
/// * `volume` - Volume level between 0.0 and 1.0
/// 
/// # Returns
/// * `Ok(String)` - Success message
/// * `Err(String)` - Error if volume is out of range
#[tauri::command]
fn set_volume(volume: f32) -> Result<String, String> {
    // Validate range before applying
    if volume < 0.0 || volume > 1.0 {
        return Err("Volume must be between 0 and 1".to_string());
    }
    
    Ok(format!("Volume set to {}", volume))
}
```

### Naming
- snake_case for functions and variables
- PascalCase for types and structs
- SCREAMING_SNAKE_CASE for constants
```rust
const MAX_VOLUME: f32 = 1.0;

struct MediaController {
    current_volume: f32,
}

fn set_media_volume(volume: f32) -> Result<(), String> {
    // ...
}
```

### Error Messages
- Be descriptive
- Include context
- Use proper formatting
```rust
// Good
Err(format!("Failed to set volume {}: invalid range (must be 0-1)", volume))

// Bad
Err("Error".to_string())
```

---

## Common Mistakes to Avoid

### DON'T: Use .unwrap() in production
```rust
// Bad
let value = some_result.unwrap();

// Good
let value = some_result.map_err(|e| format!("Error: {}", e))?;
```

### DON'T: Ignore compiler warnings
- Rust compiler warnings are almost always real issues
- Fix them, don't suppress them

### DON'T: Clone unnecessarily
```rust
// Bad - unnecessary clone
let s1 = String::from("hello");
let s2 = s1.clone();
process(&s2);

// Good - borrow instead
let s1 = String::from("hello");
process(&s1);
```

### DO: Use lifetimes when borrowing
```rust
// Function signature shows lifetime
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

---

## Debugging Tips

### Print debugging
```rust
// Regular print
println!("Value: {}", value);

// Debug print (works with most types)
dbg!(value);

// Pretty print structs (add #[derive(Debug)])
#[derive(Debug)]
struct User {
    name: String,
}
println!("{:?}", user);
```

### Error context
```rust
use std::io::ErrorKind;

match std::fs::read_to_string("file.txt") {
    Ok(contents) => println!("{}", contents),
    Err(e) => match e.kind() {
        ErrorKind::NotFound => println!("File not found"),
        ErrorKind::PermissionDenied => println!("Permission denied"),
        _ => println!("Other error: {}", e),
    }
}
```

---

## Testing

### Unit tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_volume() {
        let result = set_volume(0.5);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_volume() {
        let result = set_volume(2.0);
        assert!(result.is_err());
    }
}
```

---

## Integration with Tauri Main

### Complete main.rs example
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};

mod api; // Your HTTP server module

// Shared state
struct AppState {
    volume: Arc<Mutex<f32>>,
}

#[tauri::command]
fn get_volume(state: tauri::State<AppState>) -> Result<f32, String> {
    let volume = state.volume.lock()
        .map_err(|e| format!("Failed to lock: {}", e))?;
    Ok(*volume)
}

#[tauri::command]
fn set_volume(volume: f32, state: tauri::State<AppState>) -> Result<String, String> {
    let mut vol = state.volume.lock()
        .map_err(|e| format!("Failed to lock: {}", e))?;
    *vol = volume;
    Ok(format!("Volume set to {}", volume))
}

#[tokio::main]
async fn main() {
    let state = AppState {
        volume: Arc::new(Mutex::new(1.0)),
    };
    
    let api_state = state.volume.clone();
    
    // Start HTTP server in background
    tokio::spawn(async move {
        api::start_server(api_state).await;
    });
    
    // Start Tauri
    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_volume,
            set_volume
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

## Quick Reference

### Common Types
- `String` - Owned, growable string
- `&str` - String slice, borrowed
- `Vec<T>` - Dynamic array
- `Option<T>` - May or may not have a value (Some/None)
- `Result<T, E>` - Success (Ok) or error (Err)
- `i32`, `f32`, `u32` - Numbers (signed int, float, unsigned int)
- `bool` - true/false

### Common Methods
- `.unwrap()` - Get value or panic (avoid in production!)
- `.expect("msg")` - Unwrap with custom panic message
- `?` - Propagate error to caller
- `.map()` - Transform value
- `.map_err()` - Transform error
- `.and_then()` - Chain operations
- `.clone()` - Create a copy (expensive!)

### Macros
- `println!()` - Print with newline
- `format!()` - Create formatted string
- `vec![]` - Create Vec
- `panic!()` - Crash with message
- `dbg!()` - Debug print

---

## When to Ask for Help

If you encounter:
- **Lifetime errors** - Complex borrow checker issues
- **Trait bounds** - Type constraints that seem confusing
- **Async/await deadlocks** - Tasks not progressing
- **Segfaults** - Usually from unsafe code (we avoid unsafe)

Always explain the error message and what you're trying to achieve.

---

## Project-Specific Context

This is a media controller app that:
- Receives commands from a Chrome extension via HTTP
- Controls media playback
- Runs in system tray
- Needs to be lightweight and responsive

Keep code:
- Simple and readable
- Well-error-handled
- Documented with comments
- Following Rust best practices