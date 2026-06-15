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

/// 宠物管理器（纯情绪状态机 + 皮肤 ID）
pub struct PetManager {
    mood: Arc<Mutex<String>>,
    skin_id: Arc<Mutex<String>>,
}

impl PetManager {
    /// 创建新的宠物管理器，默认情绪为 Happy，默认皮肤为 firefly
    pub fn new() -> Self {
        Self {
            mood: Arc::new(Mutex::new(PetMood::Happy.to_string())),
            skin_id: Arc::new(Mutex::new("firefly".to_string())),
        }
    }

    /// 创建宠物管理器并指定皮肤
    pub fn new_with_skin(skin_id: &str) -> Self {
        Self {
            mood: Arc::new(Mutex::new(PetMood::Happy.to_string())),
            skin_id: Arc::new(Mutex::new(skin_id.to_string())),
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
}
