use crate::db;
use crate::settings::DbState;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

pub fn start_scheduler(app_handle: AppHandle) {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(5));
            if let Some(state) = app_handle.try_state::<DbState>() {
                let conn = match state.0.lock() {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                let now = chrono::Utc::now().timestamp_millis();
                match db::get_due_tasks(&conn, now) {
                    Ok(tasks) => {
                        for task in tasks {
                            let _ = app_handle.emit("reminder:trigger", task);
                        }
                    }
                    Err(e) => eprintln!("Scheduler error: {}", e),
                }
            }
        }
    });
}
