use crate::crypto;
use crate::db;
use crate::models::SettingsState;
use rusqlite::Connection;
use std::sync::Mutex;
use tauri::State;

pub struct DbState(pub Mutex<Connection>);

fn get_or(conn: &Connection, key: &str, default: &str) -> Result<String, String> {
    db::get_setting(conn, key)
        .map_err(|e| e.to_string())?
        .map(Ok)
        .unwrap_or_else(|| Ok(default.to_string()))
}

fn get_bool(conn: &Connection, key: &str, default: bool) -> Result<bool, String> {
    db::get_setting(conn, key)
        .map_err(|e| e.to_string())?
        .map(|v| Ok(v == "1"))
        .unwrap_or(Ok(default))
}

fn get_f64(conn: &Connection, key: &str, default: f64) -> Result<f64, String> {
    db::get_setting(conn, key)
        .map_err(|e| e.to_string())?
        .map(|v| v.parse().map_err(|_| format!("invalid f64 for {}", key)))
        .unwrap_or(Ok(default))
}

#[tauri::command]
pub fn get_settings(state: State<DbState>) -> Result<SettingsState, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let api_key = crypto::get_api_key()?;
    Ok(SettingsState {
        provider: get_or(&conn, "provider", "openai")?,
        api_base: get_or(&conn, "api_base", "https://api.openai.com/v1")?,
        api_key,
        model: get_or(&conn, "model", "gpt-4o-mini")?,
        system_prompt: get_or(&conn, "system_prompt", "")?,
        scale_factor: get_f64(&conn, "scale_factor", 1.0)?,
        avatar_pack_path: get_or(&conn, "avatar_pack_path", "")?,
        render_fix: get_bool(&conn, "render_fix", false)?,
        auto_start: get_bool(&conn, "auto_start", false)?,
        always_on_top: get_bool(&conn, "always_on_top", true)?,
        click_through: get_bool(&conn, "click_through", true)?,
        shortcut_show_hide: get_or(&conn, "shortcut_show_hide", "")?,
    })
}

#[tauri::command]
pub fn save_settings(state: State<DbState>, settings: SettingsState) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::set_setting(&conn, "provider", &settings.provider).map_err(|e| e.to_string())?;
    db::set_setting(&conn, "api_base", &settings.api_base).map_err(|e| e.to_string())?;
    db::set_setting(&conn, "model", &settings.model).map_err(|e| e.to_string())?;
    db::set_setting(&conn, "system_prompt", &settings.system_prompt).map_err(|e| e.to_string())?;
    db::set_setting(&conn, "scale_factor", &settings.scale_factor.to_string()).map_err(|e| e.to_string())?;
    db::set_setting(&conn, "avatar_pack_path", &settings.avatar_pack_path).map_err(|e| e.to_string())?;
    db::set_setting(&conn, "render_fix", if settings.render_fix { "1" } else { "0" }).map_err(|e| e.to_string())?;
    db::set_setting(&conn, "auto_start", if settings.auto_start { "1" } else { "0" }).map_err(|e| e.to_string())?;
    db::set_setting(&conn, "always_on_top", if settings.always_on_top { "1" } else { "0" }).map_err(|e| e.to_string())?;
    db::set_setting(&conn, "click_through", if settings.click_through { "1" } else { "0" }).map_err(|e| e.to_string())?;
    db::set_setting(&conn, "shortcut_show_hide", &settings.shortcut_show_hide).map_err(|e| e.to_string())?;
    match settings.api_key {
        Some(key) if !key.is_empty() => crypto::store_api_key(&key)?,
        _ => crypto::delete_api_key()?,
    }
    Ok(())
}
