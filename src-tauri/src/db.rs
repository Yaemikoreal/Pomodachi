use crate::models::{CreateTaskInput, Task};
use rusqlite::{params, Connection, Result};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

pub fn db_path(app_handle: &tauri::AppHandle) -> PathBuf {
    let mut path = app_handle
        .path()
        .app_data_dir()
        .expect("failed to get app data dir");
    fs::create_dir_all(&path).ok();
    path.push("pomodachi.db");
    path
}

pub fn init_db(app_handle: &tauri::AppHandle) -> Result<Connection> {
    let path = db_path(app_handle);
    let conn = Connection::open(path)?;

    conn.execute_batch(
        "DROP TABLE IF EXISTS tasks;
        DROP TABLE IF EXISTS messages;
        DROP TABLE IF EXISTS settings;

        CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            due_at INTEGER,
            repeat_rule TEXT DEFAULT 'none',
            emotion_hint TEXT DEFAULT '认真',
            is_done INTEGER DEFAULT 0,
            created_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            emotion_tag TEXT,
            created_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );",
    )?;

    Ok(conn)
}

pub fn create_task(conn: &Connection, input: &CreateTaskInput) -> Result<i64> {
    let now = chrono::Utc::now().timestamp_millis();
    conn.execute(
        "INSERT INTO tasks (title, due_at, repeat_rule, emotion_hint, is_done, created_at)
         VALUES (?1, ?2, ?3, ?4, 0, ?5)",
        params![
            input.title,
            input.due_at,
            input.repeat_rule.as_deref().unwrap_or("none"),
            input.emotion_hint.as_deref().unwrap_or("认真"),
            now
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn list_tasks(conn: &Connection, include_done: bool) -> Result<Vec<Task>> {
    let sql = if include_done {
        "SELECT id, title, due_at, repeat_rule, emotion_hint, is_done, created_at
         FROM tasks ORDER BY is_done ASC, due_at ASC"
    } else {
        "SELECT id, title, due_at, repeat_rule, emotion_hint, is_done, created_at
         FROM tasks WHERE is_done = 0 ORDER BY due_at ASC"
    };
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], |row| {
        Ok(Task {
            id: row.get(0)?,
            title: row.get(1)?,
            due_at: row.get(2)?,
            repeat_rule: row.get(3)?,
            emotion_hint: row.get(4)?,
            is_done: row.get::<_, i32>(5)? != 0,
            created_at: row.get(6)?,
        })
    })?;
    rows.collect()
}

pub fn update_task(conn: &Connection, input: &crate::models::UpdateTaskInput) -> Result<usize> {
    let current = conn.query_row(
        "SELECT id, title, due_at, repeat_rule, emotion_hint, is_done, created_at FROM tasks WHERE id = ?1",
        params![input.id],
        |row| {
            Ok(crate::models::Task {
                id: row.get(0)?,
                title: row.get(1)?,
                due_at: row.get(2)?,
                repeat_rule: row.get(3)?,
                emotion_hint: row.get(4)?,
                is_done: row.get::<_, i32>(5)? != 0,
                created_at: row.get(6)?,
            })
        },
    )?;

    let title = input.title.as_ref().unwrap_or(&current.title);
    let due_at = input.due_at.or(current.due_at);
    let repeat_rule = input.repeat_rule.as_ref().unwrap_or(&current.repeat_rule);
    let emotion_hint = input.emotion_hint.as_ref().unwrap_or(&current.emotion_hint);

    conn.execute(
        "UPDATE tasks SET title = ?1, due_at = ?2, repeat_rule = ?3, emotion_hint = ?4 WHERE id = ?5",
        params![title, due_at, repeat_rule, emotion_hint, input.id],
    )
}

pub fn complete_task(conn: &Connection, id: i64) -> Result<usize> {
    conn.execute("UPDATE tasks SET is_done = 1 WHERE id = ?1", params![id])
}

pub fn delete_task(conn: &Connection, id: i64) -> Result<usize> {
    conn.execute("DELETE FROM tasks WHERE id = ?1", params![id])
}

pub fn postpone_task(conn: &Connection, id: i64, minutes: i64) -> Result<usize> {
    let now = chrono::Utc::now().timestamp_millis();
    let new_due = now + minutes * 60 * 1000;
    conn.execute(
        "UPDATE tasks SET due_at = ?1 WHERE id = ?2",
        params![new_due, id],
    )
}

pub fn get_due_tasks(conn: &Connection, before: i64) -> Result<Vec<Task>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, due_at, repeat_rule, emotion_hint, is_done, created_at
         FROM tasks WHERE is_done = 0 AND due_at IS NOT NULL AND due_at <= ?1
         ORDER BY due_at ASC",
    )?;
    let rows = stmt.query_map(params![before], |row| {
        Ok(Task {
            id: row.get(0)?,
            title: row.get(1)?,
            due_at: row.get(2)?,
            repeat_rule: row.get(3)?,
            emotion_hint: row.get(4)?,
            is_done: row.get::<_, i32>(5)? != 0,
            created_at: row.get(6)?,
        })
    })?;
    rows.collect()
}

pub fn save_message(conn: &Connection, role: &str, content: &str, emotion_tag: Option<&str>) -> Result<()> {
    let now = chrono::Utc::now().timestamp_millis();
    conn.execute(
        "INSERT INTO messages (role, content, emotion_tag, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![role, content, emotion_tag, now],
    )?;
    Ok(())
}

pub fn recent_messages(conn: &Connection, limit: usize) -> Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT role, content FROM messages ORDER BY created_at DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit as i64], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    rows.collect()
}

pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
    let mut rows = stmt.query(params![key])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}
