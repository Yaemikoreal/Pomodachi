use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

/// 宠物情绪状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum PetMood {
    Happy,
    Focused,
    Tired,
    Sleeping,
    Listening,
    Thinking,
}

impl PetMood {
    pub fn to_string(&self) -> String {
        match self {
            PetMood::Happy => "happy".to_string(),
            PetMood::Focused => "focused".to_string(),
            PetMood::Tired => "tired".to_string(),
            PetMood::Sleeping => "sleeping".to_string(),
            PetMood::Listening => "listening".to_string(),
            PetMood::Thinking => "thinking".to_string(),
        }
    }

}

/// 宠物管理器（纯情绪状态机，无需数据库依赖）
pub struct PetManager {
    mood: Arc<Mutex<String>>,
}

impl PetManager {
    /// 创建新的宠物管理器，默认情绪为 Happy
    pub fn new() -> Self {
        Self {
            mood: Arc::new(Mutex::new(PetMood::Happy.to_string())),
        }
    }

    /// 获取当前情绪
    pub async fn get_mood(&self) -> String {
        self.mood.lock().await.clone()
    }

    /// 设置情绪
    pub async fn set_mood(&self, mood: &str) {
        let mut current = self.mood.lock().await;
        *current = mood.to_string();
    }

    /// 开始专注 → 情绪变为 Focused
    pub async fn on_focus_start(&self) {
        let mut mood = self.mood.lock().await;
        *mood = PetMood::Focused.to_string();
    }

    /// 休息 → 情绪变为 Sleeping
    pub async fn on_rest(&self) {
        let mut mood = self.mood.lock().await;
        *mood = PetMood::Sleeping.to_string();
    }

    /// 专注完成 → 情绪变为 Happy
    pub async fn on_focus_complete(&self) {
        let mut mood = self.mood.lock().await;
        *mood = PetMood::Happy.to_string();
    }

    /// 用户开始输入 → 情绪变为 Listening
    pub async fn on_chat_start(&self) {
        let mut mood = self.mood.lock().await;
        *mood = PetMood::Listening.to_string();
    }

    /// AI 正在回复 → 情绪变为 Thinking
    pub async fn on_ai_thinking(&self) {
        let mut mood = self.mood.lock().await;
        *mood = PetMood::Thinking.to_string();
    }
}
