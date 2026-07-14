use crate::db;
use crate::models::{CreateTaskInput, Task, UpdateTaskInput};
use crate::settings::DbState;
use tauri::State;

#[tauri::command]
pub fn create_task(state: State<DbState>, input: CreateTaskInput) -> Result<i64, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::create_task(&conn, &input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_tasks(state: State<DbState>, include_done: bool) -> Result<Vec<Task>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::list_tasks(&conn, include_done).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_task(state: State<DbState>, input: UpdateTaskInput) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::update_task(&conn, &input).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn complete_task(state: State<DbState>, id: i64) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::complete_task(&conn, id).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_task(state: State<DbState>, id: i64) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::delete_task(&conn, id).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn postpone_task(state: State<DbState>, id: i64, minutes: i64) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::postpone_task(&conn, id, minutes).map_err(|e| e.to_string())?;
    Ok(())
}


