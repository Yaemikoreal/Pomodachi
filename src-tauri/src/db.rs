use rusqlite::{Connection, Result, params};
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

/// 成就定义
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Achievement {
    pub id: i64,
    pub key: String,
    pub name: String,
    pub description: String,
    pub unlocked_at: Option<String>,
    pub icon: String,
}

/// 专注统计
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FocusStats {
    pub total_pomodoros: i32,
    pub total_focus_seconds: i64,
    pub current_streak: i32,
    pub longest_streak: i32,
    pub last_focus_date: Option<String>,
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

    /// 创建内存数据库（用于测试）
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
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
        // 第一段：创建表（单独作用域以释放锁）
        {
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

                -- 成就表
                CREATE TABLE IF NOT EXISTS achievements (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    key TEXT NOT NULL UNIQUE,
                    name TEXT NOT NULL,
                    description TEXT NOT NULL,
                    unlocked_at TEXT,
                    icon TEXT NOT NULL DEFAULT '🏆'
                );

                -- 专注统计表（单行累计）
                CREATE TABLE IF NOT EXISTS focus_stats (
                    id INTEGER PRIMARY KEY DEFAULT 1,
                    total_pomodoros INTEGER NOT NULL DEFAULT 0,
                    total_focus_seconds INTEGER NOT NULL DEFAULT 0,
                    current_streak INTEGER NOT NULL DEFAULT 0,
                    longest_streak INTEGER NOT NULL DEFAULT 0,
                    last_focus_date TEXT
                );

                -- 初始化宠物数据和 AI 配置（如果不存在）
                INSERT OR IGNORE INTO pet (id, name, mood) VALUES (1, '番茄猫', 'happy');
                INSERT OR IGNORE INTO ai_config (id) VALUES (1);
                ",
            )?;
        } // conn 在此释放，锁被释放

        // 运行数据库迁移（获取自己的锁）
        self.run_migrations()?;

        // 初始化默认成就（获取自己的锁）
        self.init_default_achievements()?;

        Ok(())
    }

    /// 运行数据库迁移（列添加等）
    fn run_migrations(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // 迁移：pet 表添加 skin_id 列（如果不存在）
        let has_skin_id: bool = conn
            .prepare("PRAGMA table_info(pet)")?
            .query_map([], |row| row.get::<_, String>(1))?
            .filter_map(|r| r.ok())
            .any(|name| name == "skin_id");

        if !has_skin_id {
            conn.execute(
                "ALTER TABLE pet ADD COLUMN skin_id TEXT NOT NULL DEFAULT 'firefly'",
                [],
            )?;
        }

        Ok(())
    }

    /// 初始化默认成就定义
    fn init_default_achievements(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let achievements = vec![
            ("first_pomodoro", "专注新手", "完成第一个番茄钟 🍅", "🌱"),
            ("tenth_pomodoro", "专注达人", "累计完成 10 个番茄钟 🔥", "🔥"),
            ("fifty_pomodoro", "专注大师", "累计完成 50 个番茄钟 💪", "💪"),
            ("hundred_pomodoro", "专注传说", "累计完成 100 个番茄钟 👑", "👑"),
            ("total_5_hours", "时光积累·初", "累计专注 5 小时 ⏳", "⏳"),
            ("total_20_hours", "时光积累·中", "累计专注 20 小时 ⌛", "⌛"),
            ("streak_3", "三日坚持", "连续 3 天完成专注 📆", "📆"),
            ("streak_7", "七日如一日", "连续 7 天完成专注 📅", "📅"),
            ("streak_30", "月度守护", "连续 30 天完成专注 🗓️", "🗓️"),
        ];
        for (key, name, description, icon) in achievements {
            conn.execute(
                "INSERT OR IGNORE INTO achievements (key, name, description, icon) VALUES (?1, ?2, ?3, ?4)",
                params![key, name, description, icon],
            )?;
        }
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

    /// 获取宠物皮肤 ID
    pub fn get_skin_id(&self) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT skin_id FROM pet WHERE id = 1",
            [],
            |row| row.get(0),
        )
    }

    /// 设置宠物皮肤 ID
    pub fn set_skin_id(&self, skin_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE pet SET skin_id = ?1, updated_at = datetime('now') WHERE id = 1",
            rusqlite::params![skin_id],
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

    // ==================== 成就/统计方法 ====================

    /// 获取专注统计
    pub fn get_focus_stats(&self) -> Result<FocusStats> {
        let conn = self.conn.lock().unwrap();
        // 确保统计行存在
        conn.execute(
            "INSERT OR IGNORE INTO focus_stats (id) VALUES (1)",
            [],
        )?;

        conn.query_row(
            "SELECT total_pomodoros, total_focus_seconds, current_streak, longest_streak, last_focus_date \
             FROM focus_stats WHERE id = 1",
            [],
            |row| {
                Ok(FocusStats {
                    total_pomodoros: row.get(0)?,
                    total_focus_seconds: row.get(1)?,
                    current_streak: row.get(2)?,
                    longest_streak: row.get(3)?,
                    last_focus_date: row.get(4)?,
                })
            },
        )
    }

    /// 更新专注统计（番茄钟完成后调用）
    pub fn update_focus_stats(&self, duration_seconds: i32) -> Result<FocusStats> {
        let conn = self.conn.lock().unwrap();

        // 确保行存在
        conn.execute("INSERT OR IGNORE INTO focus_stats (id) VALUES (1)", [])?;

        // 获取当前日期
        let today: String = conn.query_row(
            "SELECT date('now')",
            [],
            |row| row.get(0),
        )?;

        // 获取上次专注日期
        let last_date: Option<String> = conn.query_row(
            "SELECT last_focus_date FROM focus_stats WHERE id = 1",
            [],
            |row| row.get(0),
        ).ok();

        // 计算连续天数
        let new_streak = match last_date {
            Some(ref d) if d == &today => {
                // 今天已专注过，保持连续
                None // 用 SQL 保持原值
            }
            Some(ref d) => {
                // 检查昨天
                let yesterday: String = conn.query_row(
                    "SELECT date('now', '-1 day')",
                    [],
                    |row| row.get(0),
                ).unwrap_or_default();
                if d == &yesterday {
                    Some(1) // 连续 +1，由 SQL 累加
                } else {
                    Some(1) // 重置为 1（断签）
                }
            }
            None => Some(1), // 首次专注
        };

        // 更新统计
        if new_streak == Some(1) {
            // 检查昨天是否专注过
            let is_yesterday = match last_date {
                Some(ref d) => {
                    let yesterday: String = conn.query_row(
                        "SELECT date('now', '-1 day')",
                        [],
                        |row| row.get(0),
                    ).unwrap_or_default();
                    d == &yesterday
                }
                None => false,
            };

            if is_yesterday {
                // 连续 +1
                conn.execute(
                    "UPDATE focus_stats SET
                        total_pomodoros = total_pomodoros + 1,
                        total_focus_seconds = total_focus_seconds + ?1,
                        current_streak = current_streak + 1,
                        longest_streak = MAX(longest_streak, current_streak + 1),
                        last_focus_date = date('now')
                    WHERE id = 1",
                    params![duration_seconds],
                )?;
            } else {
                // 重置连续为 1
                conn.execute(
                    "UPDATE focus_stats SET
                        total_pomodoros = total_pomodoros + 1,
                        total_focus_seconds = total_focus_seconds + ?1,
                        current_streak = 1,
                        last_focus_date = date('now')
                    WHERE id = 1",
                    params![duration_seconds],
                )?;
            }
        } else {
            // 今天已专注过，只累加数据，不改变连续值
            conn.execute(
                "UPDATE focus_stats SET
                    total_pomodoros = total_pomodoros + 1,
                    total_focus_seconds = total_focus_seconds + ?1
                WHERE id = 1",
                params![duration_seconds],
            )?;
        }

        // 更新 longest_streak（确保不遗漏）
        conn.execute(
            "UPDATE focus_stats SET longest_streak = MAX(longest_streak, current_streak) WHERE id = 1",
            [],
        )?;

        drop(conn);
        self.get_focus_stats()
    }

    /// 获取所有成就列表
    pub fn get_achievements(&self) -> Result<Vec<Achievement>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, key, name, description, unlocked_at, icon \
             FROM achievements ORDER BY id",
        )?;

        let achievements = stmt
            .query_map([], |row| {
                Ok(Achievement {
                    id: row.get(0)?,
                    key: row.get(1)?,
                    name: row.get(2)?,
                    description: row.get(3)?,
                    unlocked_at: row.get(4)?,
                    icon: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;

        Ok(achievements)
    }

    /// 检查是否有新解锁的成就
    pub fn check_achievements(&self, stats: &FocusStats) -> Result<Vec<Achievement>> {
        let mut new_achievements = Vec::new();

        // 检查每个成就条件
        let checks: Vec<(&str, bool)> = vec![
            ("first_pomodoro", stats.total_pomodoros >= 1),
            ("tenth_pomodoro", stats.total_pomodoros >= 10),
            ("fifty_pomodoro", stats.total_pomodoros >= 50),
            ("hundred_pomodoro", stats.total_pomodoros >= 100),
            ("total_5_hours", stats.total_focus_seconds >= 5 * 3600),
            ("total_20_hours", stats.total_focus_seconds >= 20 * 3600),
            ("streak_3", stats.current_streak >= 3),
            ("streak_7", stats.current_streak >= 7),
            ("streak_30", stats.current_streak >= 30),
        ];

        let conn = self.conn.lock().unwrap();
        for (key, met) in checks {
            if met {
                // 尝试解锁（只有在未解锁时才返回行）
                let affected = conn.execute(
                    "UPDATE achievements SET unlocked_at = datetime('now') \
                     WHERE key = ?1 AND unlocked_at IS NULL",
                    params![key],
                )?;
                if affected > 0 {
                    // 读取完整成就数据
                    if let Ok(ach) = conn.query_row(
                        "SELECT id, key, name, description, unlocked_at, icon \
                         FROM achievements WHERE key = ?1",
                        params![key],
                        |row| {
                            Ok(Achievement {
                                id: row.get(0)?,
                                key: row.get(1)?,
                                name: row.get(2)?,
                                description: row.get(3)?,
                                unlocked_at: row.get(4)?,
                                icon: row.get(5)?,
                            })
                        },
                    ) {
                        new_achievements.push(ach);
                    }
                }
            }
        }

        Ok(new_achievements)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_db() -> Database {
        Database::new_in_memory().expect("创建内存数据库失败")
    }

    #[test]
    fn test_init_tables_creates_default_pet() {
        let db = create_test_db();
        let pet = db.get_pet_status().unwrap();
        assert_eq!(pet.name, "番茄猫");
        assert_eq!(pet.mood, "happy");
        assert_eq!(pet.id, 1);
    }

    #[test]
    fn test_pomodoro_crud() {
        let db = create_test_db();
        let id = db.add_pomodoro_record(25 * 60).unwrap();
        assert!(id > 0);

        // 完成记录
        db.complete_pomodoro_record(id, 2).unwrap();

        // 查询历史
        let history = db.get_pomodoro_history(10).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].id, id);
        assert!(history[0].completed);
        assert_eq!(history[0].distraction_count, 2);
    }

    #[test]
    fn test_today_pomodoro_count() {
        let db = create_test_db();
        assert_eq!(db.get_today_pomodoro_count().unwrap(), 0);

        db.add_pomodoro_record(25 * 60).unwrap();
        // 刚添加未完成的记录不计入（因为 completed = 0）
        assert_eq!(db.get_today_pomodoro_count().unwrap(), 0);

        let last_id = db.add_pomodoro_record(25 * 60).unwrap();
        db.complete_pomodoro_record(last_id, 0).unwrap();
        // completed = true 的记录会计入
        assert_eq!(db.get_today_pomodoro_count().unwrap(), 1);
    }

    #[test]
    fn test_blocklist_crud() {
        let db = create_test_db();
        let list = db.get_blocklist().unwrap();
        assert!(list.is_empty());

        db.add_to_blocklist("chrome.exe").unwrap();
        db.add_to_blocklist("wechat.exe").unwrap();

        let list = db.get_blocklist().unwrap();
        assert_eq!(list.len(), 2);

        // 重复添加不报错
        db.add_to_blocklist("chrome.exe").unwrap();
        let list = db.get_blocklist().unwrap();
        assert_eq!(list.len(), 2);

        // 移除
        db.remove_from_blocklist("chrome.exe").unwrap();
        let list = db.get_blocklist().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].process_name, "wechat.exe");
    }

    #[test]
    fn test_chat_message_crud() {
        let db = create_test_db();
        let history = db.get_chat_history(10).unwrap();
        assert!(history.is_empty());

        db.add_chat_message("user", "你好").unwrap();
        db.add_chat_message("assistant", "喵~").unwrap();

        let history = db.get_chat_history(10).unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].role, "user");
        assert_eq!(history[0].content, "你好");
        assert_eq!(history[1].role, "assistant");
        assert_eq!(history[1].content, "喵~");

        // 清空
        db.clear_chat_history().unwrap();
        let history = db.get_chat_history(10).unwrap();
        assert!(history.is_empty());
    }

    #[test]
    fn test_task_crud() {
        let db = create_test_db();
        let tasks = db.get_tasks().unwrap();
        assert!(tasks.is_empty());

        // 添加
        let id = db.add_task("测试任务", 1, Some("2026-12-31")).unwrap();
        assert!(id > 0);

        let tasks = db.get_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "测试任务");
        assert_eq!(tasks[0].priority, 1);

        // 切换完成
        db.complete_task(id, true).unwrap();
        let tasks = db.get_tasks().unwrap();
        assert!(tasks[0].completed);

        // 删除
        db.delete_task(id).unwrap();
        let tasks = db.get_tasks().unwrap();
        assert!(tasks.is_empty());
    }

    #[test]
    fn test_settings_crud() {
        let db = create_test_db();

        // 不存在的键返回 None
        assert!(db.get_setting("nonexistent").unwrap().is_none());

        db.set_setting("language", "zh").unwrap();
        assert_eq!(db.get_setting("language").unwrap().unwrap(), "zh");

        // 覆盖
        db.set_setting("language", "en").unwrap();
        assert_eq!(db.get_setting("language").unwrap().unwrap(), "en");
    }

    #[test]
    fn test_ai_config_default() {
        let db = create_test_db();
        let config = db.get_ai_config().unwrap();
        assert_eq!(config.max_turns, 3);
        assert_eq!(config.max_budget_usd, Some(0.1));
    }

    #[test]
    fn test_skin_id() {
        let db = create_test_db();
        // 默认值
        let skin = db.get_skin_id().unwrap();
        assert_eq!(skin, "firefly");

        db.set_skin_id("cat2").unwrap();
        let skin = db.get_skin_id().unwrap();
        assert_eq!(skin, "cat2");
    }

    #[test]
    fn test_focus_stats_initial() {
        let db = create_test_db();
        let stats = db.get_focus_stats().unwrap();
        assert_eq!(stats.total_pomodoros, 0);
        assert_eq!(stats.total_focus_seconds, 0);
        assert_eq!(stats.current_streak, 0);
        assert_eq!(stats.longest_streak, 0);
    }

    #[test]
    fn test_focus_stats_update() {
        let db = create_test_db();
        let stats = db.update_focus_stats(25 * 60).unwrap();
        assert_eq!(stats.total_pomodoros, 1);
        assert_eq!(stats.total_focus_seconds, 25 * 60);
        assert_eq!(stats.current_streak, 1);
        assert_eq!(stats.longest_streak, 1);
    }

    #[test]
    fn test_achievements_init() {
        let db = create_test_db();
        let achievements = db.get_achievements().unwrap();
        assert_eq!(achievements.len(), 9);
        // 全部未解锁
        assert!(achievements.iter().all(|a| a.unlocked_at.is_none()));
    }

    #[test]
    fn test_check_achievements_first_pomodoro() {
        let db = create_test_db();
        let stats = FocusStats {
            total_pomodoros: 1,
            total_focus_seconds: 1500,
            current_streak: 1,
            longest_streak: 1,
            last_focus_date: None,
        };
        let new = db.check_achievements(&stats).unwrap();
        assert!(!new.is_empty());
        assert!(new.iter().any(|a| a.key == "first_pomodoro"));
    }

    #[test]
    fn test_check_achievements_no_duplicate() {
        let db = create_test_db();
        let stats = FocusStats {
            total_pomodoros: 1,
            total_focus_seconds: 1500,
            current_streak: 1,
            longest_streak: 1,
            last_focus_date: None,
        };
        // 第一次应该解锁
        let first = db.check_achievements(&stats).unwrap();
        assert_eq!(first.len(), 1);
        // 第二次不应该再次解锁
        let second = db.check_achievements(&stats).unwrap();
        assert!(second.is_empty());
    }

    #[test]
    fn test_get_focus_stats_persistence() {
        let db = create_test_db();
        // 首次获取时自动创建
        let stats = db.get_focus_stats().unwrap();
        assert_eq!(stats.total_pomodoros, 0);
    }
}
