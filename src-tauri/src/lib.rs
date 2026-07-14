use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, Manager, Window, WindowEvent};

mod crypto;
mod db;
mod llm;
mod models;
mod scheduler;
mod settings;
mod task;

pub use settings::DbState;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn start_dragging(window: Window) -> Result<(), String> {
    window
        .start_dragging()
        .map_err(|e| format!("Failed to start dragging: {}", e))
}

#[tauri::command]
fn show_window(window: Window) -> Result<(), String> {
    window
        .show()
        .map_err(|e| format!("Failed to show window: {}", e))?;
    window
        .set_focus()
        .map_err(|e| format!("Failed to focus window: {}", e))
}

#[tauri::command]
fn hide_window(window: Window) -> Result<(), String> {
    window
        .hide()
        .map_err(|e| format!("Failed to hide window: {}", e))
}

#[tauri::command]
fn set_ignore_cursor_events(window: Window, ignore: bool) -> Result<(), String> {
    window
        .set_ignore_cursor_events(ignore)
        .map_err(|e| format!("Failed to set ignore cursor events: {}", e))
}

#[tauri::command]
fn set_window_size(window: Window, width: f64, height: f64) -> Result<(), String> {
    use tauri::LogicalSize;
    window
        .set_size(tauri::Size::Logical(LogicalSize { width, height }))
        .map_err(|e| format!("Failed to set window size: {}", e))
}

#[tauri::command]
fn set_always_on_top(window: Window, always_on_top: bool) -> Result<(), String> {
    window
        .set_always_on_top(always_on_top)
        .map_err(|e| format!("Failed to set always on top: {}", e))
}

#[tauri::command]
fn set_auto_start(app_handle: tauri::AppHandle, auto_start: bool) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;
    let manager = app_handle.autolaunch();
    if auto_start {
        manager.enable().map_err(|e| e.to_string())
    } else {
        manager.disable().map_err(|e| e.to_string())
    }
}

#[tauri::command]
fn quit_app(app_handle: tauri::AppHandle) {
    app_handle.exit(0);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .invoke_handler(tauri::generate_handler![
            greet,
            start_dragging,
            show_window,
            hide_window,
            quit_app,
            llm::send_message,
            task::create_task,
            task::list_tasks,
            task::update_task,
            task::complete_task,
            task::delete_task,
            task::postpone_task,
            settings::get_settings,
            settings::save_settings,
            set_ignore_cursor_events,
            set_window_size,
            set_always_on_top,
            set_auto_start,
        ])
        .setup(|app| {
            // Initialize database
            let conn = db::init_db(&app.handle())?;
            app.manage(DbState(std::sync::Mutex::new(conn)));

            // Start reminder scheduler
            scheduler::start_scheduler(app.handle().clone());

            // Build tray menu
            let toggle_i = MenuItemBuilder::with_id("toggle", "显示/隐藏").build(app)?;
            let settings_i = MenuItemBuilder::with_id("settings", "设置").build(app)?;
            let quit_i = MenuItemBuilder::with_id("quit", "退出").build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&toggle_i)
                .item(&settings_i)
                .separator()
                .item(&quit_i)
                .build()?;

            let tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event({
                    move |app, event| match event.id.as_ref() {
                        "toggle" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.is_visible().map(|visible| {
                                    if visible {
                                        let _ = window.hide();
                                    } else {
                                        let _ = window.show();
                                        let _ = window.set_focus();
                                    }
                                });
                            }
                        }
                        "settings" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                                let _ = window.emit("open-settings", ());
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.is_visible().map(|visible| {
                                if visible {
                                    let _ = window.hide();
                                } else {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            });
                        }
                    }
                })
                .build(app)?;

            // Keep tray alive for the lifetime of the app
            let _ = tray;

            // Ensure closing the window hides it instead of quitting
            if let Some(window) = app.get_webview_window("main") {
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = window_clone.hide();
                    }
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
