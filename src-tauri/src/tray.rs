// tray.rs — System tray icon and menu.
//
// Tauri 2.0 provides tray support via the `tray-icon` feature flag.
// We build a menu with two items:
//   • Show / Hide — toggles the main window visibility
//   • Quit        — exits the application
//
// Clicking the tray icon itself (not the menu) also shows the window.

use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

/// Set up the system tray for the given app handle.
/// Call this inside the `.setup()` callback in lib.rs.
pub fn setup_tray(app: &AppHandle) -> Result<(), tauri::Error> {
    // Build the context-menu items.
    // MenuItem::with_id(app, id, label, enabled, accelerator)
    let show_item = MenuItem::with_id(app, "show", "Show / Hide", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    // Assemble the menu from the items
    let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

    // Build the tray icon and attach both a menu-event handler and a click handler.
    // The leading `_tray` keeps the value alive for the lifetime of the program.
    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        // Called when the user clicks a menu item
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "show" => toggle_window_visibility(app),
                "quit" => {
                    println!("[tray] Quit requested");
                    app.exit(0);
                }
                unknown => {
                    println!("[tray] Unknown menu event: {}", unknown);
                }
            }
        })
        // Called when the user clicks the tray icon itself (not a menu item)
        .on_tray_icon_event(|tray, event| {
            // Only react to left-button clicks; ignore hover, right-click, etc.
            if let TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                button_state: tauri::tray::MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}

/// Toggle the main window between visible and hidden.
fn toggle_window_visibility(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        // is_visible() returns Result<bool>, so we unwrap_or to a safe default
        let visible = window.is_visible().unwrap_or(false);

        if visible {
            let _ = window.hide();
            println!("[tray] Window hidden");
        } else {
            let _ = window.show();
            let _ = window.set_focus();
            println!("[tray] Window shown");
        }
    }
}
