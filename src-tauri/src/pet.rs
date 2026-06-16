use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tauri::{AppHandle, Emitter};
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

/// 宠物管理器（情绪状态机 + 皮肤 ID + 事件通知 + 手动覆盖）
pub struct PetManager {
    mood: Arc<Mutex<String>>,
    skin_id: Arc<Mutex<String>>,
    app_handle: Option<AppHandle>,
    /// 手动覆盖到期时间（覆盖期内 ClaudeCodeMonitor 不自动切换情绪）
    manual_override_until: Arc<Mutex<Option<Instant>>>,
}

impl PetManager {
    /// 创建新的宠物管理器（测试用，无 AppHandle，不发事件）
    pub fn new() -> Self {
        Self {
            mood: Arc::new(Mutex::new(PetMood::Happy.to_string())),
            skin_id: Arc::new(Mutex::new("firefly".to_string())),
            app_handle: None,
            manual_override_until: Arc::new(Mutex::new(None)),
        }
    }

    /// 创建宠物管理器并指定皮肤（测试用，无 AppHandle）
    pub fn new_with_skin(skin_id: &str) -> Self {
        Self {
            mood: Arc::new(Mutex::new(PetMood::Happy.to_string())),
            skin_id: Arc::new(Mutex::new(skin_id.to_string())),
            app_handle: None,
            manual_override_until: Arc::new(Mutex::new(None)),
        }
    }

    /// 创建宠物管理器（生产用，持有 AppHandle，情绪变化时自动 emit 事件）
    pub fn new_with_handle(app_handle: AppHandle) -> Self {
        Self {
            mood: Arc::new(Mutex::new(PetMood::Happy.to_string())),
            skin_id: Arc::new(Mutex::new("firefly".to_string())),
            app_handle: Some(app_handle),
            manual_override_until: Arc::new(Mutex::new(None)),
        }
    }

    /// 获取当前情绪
    pub async fn get_mood(&self) -> String {
        self.mood.lock().await.clone()
    }

    /// 设置情绪，如果有 AppHandle 则自动 emit pet-mood-changed 事件
    pub async fn set_mood(&self, mood: &str) {
        let mut current = self.mood.lock().await;
        if *current != mood {
            *current = mood.to_string();
            if let Some(ref handle) = self.app_handle {
                let _ = handle.emit("pet-mood-changed", mood);
            }
        }
    }

    /// 设置情绪并标记手动覆盖（覆盖 duration_secs 秒内不自动切换）
    pub async fn set_mood_with_override(&self, mood: &str, duration_secs: u64) {
        self.set_mood(mood).await;
        let mut override_until = self.manual_override_until.lock().await;
        *override_until = Some(Instant::now() + std::time::Duration::from_secs(duration_secs));
    }

    /// 清除手动覆盖，立即恢复自动模式
    pub async fn clear_override(&self) {
        let mut override_until = self.manual_override_until.lock().await;
        *override_until = None;
    }

    /// 检查是否在手动覆盖期内
    pub async fn is_overridden(&self) -> bool {
        let override_until = self.manual_override_until.lock().await;
        if let Some(until) = *override_until {
            return Instant::now() < until;
        }
        false
    }

    /// 获取当前皮肤 ID
    pub async fn get_skin_id(&self) -> String {
        self.skin_id.lock().await.clone()
    }

    /// 设置皮肤 ID
    pub async fn set_skin_id(&self, skin_id: &str) {
        let mut current = self.skin_id.lock().await;
        *current = skin_id.to_string();
    }

    /// 开始专注 → 情绪变为 Focused
    pub async fn on_focus_start(&self) {
        self.set_mood(&PetMood::Focused.to_string()).await;
    }

    /// 休息 → 情绪变为 Sleeping
    pub async fn on_rest(&self) {
        self.set_mood(&PetMood::Sleeping.to_string()).await;
    }

    /// 专注完成 → 情绪变为 Happy
    pub async fn on_focus_complete(&self) {
        self.set_mood(&PetMood::Happy.to_string()).await;
    }

    /// 用户开始输入 → 情绪变为 Listening
    pub async fn on_chat_start(&self) {
        self.set_mood(&PetMood::Listening.to_string()).await;
    }

    /// AI 正在回复 → 情绪变为 Thinking
    pub async fn on_ai_thinking(&self) {
        self.set_mood(&PetMood::Thinking.to_string()).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_mood() {
        let pet = PetManager::new();
        assert_eq!(pet.get_mood().await, "happy");
    }

    #[tokio::test]
    async fn test_default_skin() {
        let pet = PetManager::new();
        assert_eq!(pet.get_skin_id().await, "firefly");
    }

    #[tokio::test]
    async fn test_new_with_skin() {
        let pet = PetManager::new_with_skin("cat2");
        assert_eq!(pet.get_skin_id().await, "cat2");
    }

    #[tokio::test]
    async fn test_on_focus_start() {
        let pet = PetManager::new();
        pet.on_focus_start().await;
        assert_eq!(pet.get_mood().await, "focused");
    }

    #[tokio::test]
    async fn test_on_rest() {
        let pet = PetManager::new();
        pet.on_rest().await;
        assert_eq!(pet.get_mood().await, "sleeping");
    }

    #[tokio::test]
    async fn test_on_focus_complete() {
        let pet = PetManager::new();
        pet.on_focus_start().await;
        pet.on_focus_complete().await;
        assert_eq!(pet.get_mood().await, "happy");
    }

    #[tokio::test]
    async fn test_on_chat_start() {
        let pet = PetManager::new();
        pet.on_chat_start().await;
        assert_eq!(pet.get_mood().await, "listening");
    }

    #[tokio::test]
    async fn test_on_ai_thinking() {
        let pet = PetManager::new();
        pet.on_ai_thinking().await;
        assert_eq!(pet.get_mood().await, "thinking");
    }

    #[tokio::test]
    async fn test_set_mood_custom() {
        let pet = PetManager::new();
        pet.set_mood("tired").await;
        assert_eq!(pet.get_mood().await, "tired");
    }

    #[tokio::test]
    async fn test_set_skin_id() {
        let pet = PetManager::new();
        pet.set_skin_id("new_skin").await;
        assert_eq!(pet.get_skin_id().await, "new_skin");
    }

    #[tokio::test]
    async fn test_manual_override() {
        let pet = PetManager::new();
        // 初始无覆盖
        assert!(!pet.is_overridden().await);

        // 设置覆盖（1 秒）
        pet.set_mood_with_override("focused", 1).await;
        assert_eq!(pet.get_mood().await, "focused");
        assert!(pet.is_overridden().await);

        // 清除覆盖
        pet.clear_override().await;
        assert!(!pet.is_overridden().await);
    }

    #[tokio::test]
    async fn test_manual_override_expires() {
        let pet = PetManager::new();
        // 设置覆盖（0 秒，立即过期）
        pet.set_mood_with_override("tired", 0).await;
        // 等一小段时间确保过期
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        assert!(!pet.is_overridden().await);
    }
}
