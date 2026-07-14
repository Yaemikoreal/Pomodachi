use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub due_at: Option<i64>,
    pub repeat_rule: String,
    pub emotion_hint: String,
    pub is_done: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskInput {
    pub title: String,
    pub due_at: Option<i64>,
    pub repeat_rule: Option<String>,
    pub emotion_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskInput {
    pub id: i64,
    pub title: Option<String>,
    pub due_at: Option<i64>,
    pub repeat_rule: Option<String>,
    pub emotion_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub content: String,
    pub emotion_tag: Option<String>,
    pub intent: Option<IntentResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "intent")]
pub enum IntentResult {
    #[serde(rename = "create_task")]
    CreateTask { title: String, due_at: Option<i64>, emotion_hint: Option<String> },
    #[serde(rename = "chat")]
    Chat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsState {
    pub provider: String,
    pub api_base: String,
    pub api_key: Option<String>,
    pub model: String,
    pub system_prompt: String,
    pub scale_factor: f64,
    pub avatar_pack_path: String,
    pub render_fix: bool,
    pub auto_start: bool,
    pub always_on_top: bool,
    pub click_through: bool,
    pub shortcut_show_hide: String,
}
