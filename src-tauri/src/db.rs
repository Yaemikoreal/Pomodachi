use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

/// 宠物状态（简化版，无游戏数值）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PetStatus {
    pub id: i32,
    pub name: String,
    pub mood: String,
    pub updated_at: String,
}

/// 专注记录
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PomodoroRecord {
    pub id: i64,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub duration: i32,
    pub completed: bool,
    pub distraction_count: i32,
}

/// 黑名单项
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlocklistItem {
    pub id: i64,
    pub process_name: String,
    pub added_at: String,
}

/// 聊天消息
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub id: i64,
    pub role: String,
    pub content: String,
    pub created_at: String,
}

/// AI 配置（使用 Claude Code CLI 作为后端）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AiConfig {
    pub id: i32,
    pub max_turns: i32,
    pub max_budget_usd: Option<f64>,
    pub updated_at: String,
}

/// 任务项
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub completed: bool,
    pub priority: i32,
    pub due_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// 应用设置（key-value 结构）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppSetting {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

/// 数据库管理器
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// 初始化数据库
    pub fn new(app_handle: &AppHandle) -> Result<Self> {
        let db_path = Self::get_db_path(app_handle)?;

        // 确保目录存在
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                rusqlite::Error::InvalidParameterName(format!("创建目录失败: {}", e))
            })?;
        }

        let conn = Connection::open(db_path)?;

        let db = Database {
            conn: Mutex::new(conn),
        };
        db.init_tables()?;
        Ok(db)
    }

    /// 获取数据库路径
    fn get_db_path(app_handle: &AppHandle) -> Result<PathBuf> {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| rusqlite::Error::InvalidParameterName(format!("获取路径失败: {}", e)))?;

        Ok(app_data_dir.join("toumato.db"))
    }

    /// 初始化表结构
    fn init_tables(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute_batch(
            "
            -- 简化版宠物表（无游戏数值）
            CREATE TABLE IF NOT EXISTS pet (
                id INTEGER PRIMARY KEY DEFAULT 1,
                name TEXT NOT NULL DEFAULT '番茄猫',
                mood TEXT NOT NULL DEFAULT 'happy',
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            -- 专注记录
            CREATE TABLE IF NOT EXISTS pomodoro_record (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                started_at TEXT NOT NULL,
                finished_at TEXT,
                duration INTEGER NOT NULL,
                completed BOOLEAN NOT NULL DEFAULT 0,
                distraction_count INTEGER NOT NULL DEFAULT 0
            );

            -- 进程黑名单
            CREATE TABLE IF NOT EXISTS blocklist (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                process_name TEXT NOT NULL UNIQUE,
                added_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            -- 聊天消息
            CREATE TABLE IF NOT EXISTS chat_message (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            -- AI 配置
            CREATE TABLE IF NOT EXISTS ai_config (
                id INTEGER PRIMARY KEY DEFAULT 1,
                max_turns INTEGER NOT NULL DEFAULT 3,
                max_budget_usd REAL DEFAULT 0.1,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            -- 任务表（待办事项）
            CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                completed INTEGER NOT NULL DEFAULT 0,
                priority INTEGER NOT NULL DEFAULT 0,
                due_date TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            -- 应用设置表（key-value）
            CREATE TABLE IF NOT EXISTS app_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            -- 初始化宠物数据和 AI 配置（如果不存在）
            INSERT OR IGNORE INTO pet (id, name, mood) VALUES (1, '番茄猫', 'happy');
            INSERT OR IGNORE INTO ai_config (id) VALUES (1);
            ",
        )?;

        Ok(())
    }

    // ==================== 宠物方法 ====================

    /// 获取宠物状态
    pub fn get_pet_status(&self) -> Result<PetStatus> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, name, mood, updated_at FROM pet WHERE id = 1",
            [],
            |row| {
                Ok(PetStatus {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    mood: row.get(2)?,
                    updated_at: row.get(3)?,
                })
            },
        )
    }

    /// 更新宠物情绪
    pub fn update_pet_mood(&self, mood: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE pet SET mood = ?1, updated_at = datetime('now') WHERE id = 1",
            rusqlite::params![mood],
        )?;
        Ok(())
    }

    // ==================== 专注记录方法 ====================

    /// 添加专注记录
    pub fn add_pomodoro_record(&self, duration: i32) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO pomodoro_record (started_at, duration) VALUES (datetime('now'), ?1)",
            rusqlite::params![duration],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// 完成专注记录
    pub fn complete_pomodoro_record(&self, id: i64, distraction_count: i32) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE pomodoro_record SET finished_at = datetime('now'), completed = 1, distraction_count = ?1 WHERE id = ?2",
            rusqlite::params![distraction_count, id],
        )?;
        Ok(())
    }

    /// 获取专注历史
    pub fn get_pomodoro_history(&self, limit: i32) -> Result<Vec<PomodoroRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, started_at, finished_at, duration, completed, distraction_count FROM pomodoro_record ORDER BY id DESC LIMIT ?1"
        )?;

        let records = stmt
            .query_map([limit], |row| {
                Ok(PomodoroRecord {
                    id: row.get(0)?,
                    started_at: row.get(1)?,
                    finished_at: row.get(2)?,
                    duration: row.get(3)?,
                    completed: row.get(4)?,
                    distraction_count: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(records)
    }

    /// 获取今日专注次数
    pub fn get_today_pomodoro_count(&self) -> Result<i32> {
        let conn = self.conn.lock().unwrap();
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM pomodoro_record WHERE completed = 1 AND date(started_at) = date('now')",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    // ==================== 黑名单方法 ====================

    /// 获取黑名单
    pub fn get_blocklist(&self) -> Result<Vec<BlocklistItem>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT id, process_name, added_at FROM blocklist ORDER BY id DESC")?;

        let items = stmt
            .query_map([], |row| {
                Ok(BlocklistItem {
                    id: row.get(0)?,
                    process_name: row.get(1)?,
                    added_at: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(items)
    }

    /// 添加到黑名单
    pub fn add_to_blocklist(&self, process_name: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO blocklist (process_name) VALUES (?1)",
            rusqlite::params![process_name],
        )?;
        Ok(())
    }

    /// 从黑名单移除
    pub fn remove_from_blocklist(&self, process_name: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM blocklist WHERE process_name = ?1",
            rusqlite::params![process_name],
        )?;
        Ok(())
    }

    // ==================== 聊天消息方法 ====================

    /// 添加聊天消息
    pub fn add_chat_message(&self, role: &str, content: &str) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO chat_message (role, content) VALUES (?1, ?2)",
            rusqlite::params![role, content],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// 获取聊天历史
    pub fn get_chat_history(&self, limit: i32) -> Result<Vec<ChatMessage>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, role, content, created_at FROM chat_message ORDER BY id DESC LIMIT ?1",
        )?;

        let messages = stmt
            .query_map([limit], |row| {
                Ok(ChatMessage {
                    id: row.get(0)?,
                    role: row.get(1)?,
                    content: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        // 反转为时间正序
        Ok(messages.into_iter().rev().collect())
    }

    /// 清空聊天历史
    pub fn clear_chat_history(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM chat_message", [])?;
        Ok(())
    }

    // ==================== AI 配置方法 ====================

    /// 获取 AI 配置
    pub fn get_ai_config(&self) -> Result<AiConfig> {
        let conn = self.conn.lock().unwrap();

        // 兼容处理：旧表有 endpoint/model/api_key 字段，新表没有
        let result = conn.query_row(
            "SELECT id, max_turns, max_budget_usd, updated_at FROM ai_config WHERE id = 1",
            [],
            |row| {
                Ok(AiConfig {
                    id: row.get(0)?,
                    max_turns: row.get(1)?,
                    max_budget_usd: row.get(2)?,
                    updated_at: row.get(3)?,
                })
            },
        );

        if result.is_err() {
            // 可能是旧表结构，尝试重建
            conn.execute("DROP TABLE IF EXISTS ai_config", [])?;
            conn.execute_batch(
                "CREATE TABLE ai_config (
                    id INTEGER PRIMARY KEY DEFAULT 1,
                    max_turns INTEGER NOT NULL DEFAULT 3,
                    max_budget_usd REAL DEFAULT 0.1,
                    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
                );
                INSERT OR IGNORE INTO ai_config (id) VALUES (1);",
            )?;

            return conn.query_row(
                "SELECT id, max_turns, max_budget_usd, updated_at FROM ai_config WHERE id = 1",
                [],
                |row| {
                    Ok(AiConfig {
                        id: row.get(0)?,
                        max_turns: row.get(1)?,
                        max_budget_usd: row.get(2)?,
                        updated_at: row.get(3)?,
                    })
                },
            );
        }

        result
    }

    /// 更新 AI 配置
    pub fn update_ai_config(&self, max_turns: i32, max_budget_usd: Option<f64>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE ai_config SET max_turns = ?1, max_budget_usd = ?2, updated_at = datetime('now') WHERE id = 1",
            rusqlite::params![max_turns, max_budget_usd],
        )?;
        Ok(())
    }

    // ==================== 任务方法 ====================

    /// 添加任务
    pub fn add_task(&self, title: &str, priority: i32, due_date: Option<&str>) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO tasks (title, priority, due_date) VALUES (?1, ?2, ?3)",
            rusqlite::params![title, priority, due_date],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// 获取所有任务
    pub fn get_tasks(&self) -> Result<Vec<Task>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, completed, priority, due_date, created_at, updated_at \
             FROM tasks ORDER BY completed ASC, priority DESC, created_at DESC",
        )?;

        let tasks = stmt
            .query_map([], |row| {
                Ok(Task {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    completed: row.get::<_, i32>(2)? != 0,
                    priority: row.get(3)?,
                    due_date: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(tasks)
    }

    /// 更新任务
    pub fn update_task(
        &self,
        id: i64,
        title: &str,
        priority: i32,
        due_date: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE tasks SET title = ?1, priority = ?2, due_date = ?3, updated_at = datetime('now') WHERE id = ?4",
            rusqlite::params![title, priority, due_date, id],
        )?;
        Ok(())
    }

    /// 切换任务完成状态
    pub fn complete_task(&self, id: i64, completed: bool) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE tasks SET completed = ?1, updated_at = datetime('now') WHERE id = ?2",
            rusqlite::params![completed as i32, id],
        )?;
        Ok(())
    }

    /// 删除任务
    pub fn delete_task(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM tasks WHERE id = ?1", rusqlite::params![id])?;
        Ok(())
    }

    // ==================== 设置方法 ====================

    /// 获取设置值
    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT value FROM app_settings WHERE key = ?1",
            rusqlite::params![key],
            |row| row.get::<_, String>(0),
        );
        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// 设置值
    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES (?1, ?2, datetime('now'))",
            rusqlite::params![key, value],
        )?;
        Ok(())
    }

    /// 获取所有设置
    pub fn get_all_settings(&self) -> Result<Vec<AppSetting>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT key, value, updated_at FROM app_settings ORDER BY key",
        )?;

        let settings = stmt
            .query_map([], |row| {
                Ok(AppSetting {
                    key: row.get(0)?,
                    value: row.get(1)?,
                    updated_at: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(settings)
    }
}
