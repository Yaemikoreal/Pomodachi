use std::sync::Arc;

use crate::db::Database;

/// 任务管理器
pub struct TaskManager {
    db: Arc<Database>,
}

impl TaskManager {
    /// 创建新的任务管理器
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// 获取所有任务
    pub async fn get_tasks(&self) -> Result<Vec<crate::db::Task>, String> {
        self.db.get_tasks().map_err(|e| e.to_string())
    }

    /// 添加任务
    pub async fn add_task(
        &self,
        title: &str,
        priority: i32,
        due_date: Option<&str>,
    ) -> Result<crate::db::Task, String> {
        let id = self
            .db
            .add_task(title, priority, due_date)
            .map_err(|e| e.to_string())?;

        // 重新读取以获取完整数据（含 created_at/updated_at）
        let tasks = self.db.get_tasks().map_err(|e| e.to_string())?;
        tasks
            .into_iter()
            .find(|t| t.id == id)
            .ok_or_else(|| "添加任务后未找到".to_string())
    }

    /// 更新任务
    pub async fn update_task(
        &self,
        id: i64,
        title: &str,
        priority: i32,
        due_date: Option<&str>,
    ) -> Result<(), String> {
        self.db
            .update_task(id, title, priority, due_date)
            .map_err(|e| e.to_string())
    }

    /// 切换任务完成状态
    pub async fn complete_task(&self, id: i64, completed: bool) -> Result<(), String> {
        self.db
            .complete_task(id, completed)
            .map_err(|e| e.to_string())
    }

    /// 删除任务
    pub async fn delete_task(&self, id: i64) -> Result<(), String> {
        self.db.delete_task(id).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    fn create_test_manager() -> TaskManager {
        let db = Database::new_in_memory().expect("创建内存数据库失败");
        TaskManager::new(Arc::new(db))
    }

    #[tokio::test]
    async fn test_add_task() {
        let manager = create_test_manager();
        let task = manager.add_task("测试任务", 1, Some("2026-12-31")).await.unwrap();
        assert!(task.id > 0);
        assert_eq!(task.title, "测试任务");
        assert_eq!(task.priority, 1);
        assert!(!task.completed);
    }

    #[tokio::test]
    async fn test_complete_task() {
        let manager = create_test_manager();
        let task = manager.add_task("测试任务", 0, None).await.unwrap();

        manager.complete_task(task.id, true).await.unwrap();
        let tasks = manager.get_tasks().await.unwrap();
        assert!(tasks.iter().any(|t| t.id == task.id && t.completed));
    }

    #[tokio::test]
    async fn test_delete_task() {
        let manager = create_test_manager();
        let task = manager.add_task("待删除", 0, None).await.unwrap();

        manager.delete_task(task.id).await.unwrap();
        let tasks = manager.get_tasks().await.unwrap();
        assert!(!tasks.iter().any(|t| t.id == task.id));
    }

    #[tokio::test]
    async fn test_get_tasks_ordering() {
        let manager = create_test_manager();
        manager.add_task("普通任务", 0, None).await.unwrap();
        manager.add_task("紧急任务", 2, None).await.unwrap();

        let tasks = manager.get_tasks().await.unwrap();
        // 优先级高的排在前面
        assert_eq!(tasks[0].priority, 2);
        assert_eq!(tasks[1].priority, 0);
    }
}
