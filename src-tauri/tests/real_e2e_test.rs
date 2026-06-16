//! 真正的端到端测试 - 直接调用后端真实逻辑
//!
//! 这些测试实例化真实的后端组件，执行真实操作，验证数据持久化

use std::sync::Arc;
use tokio::sync::Mutex;
use toumato_app_lib::db::Database;
use toumato_app_lib::pet::PetManager;
use toumato_app_lib::task::TaskManager;

/// 测试任务管理完整流程 - 真实调用
#[tokio::test]
async fn test_real_task_management() {
    println!("📋 开始真实任务管理测试...");

    let db = Arc::new(Database::new_in_memory().expect("创建数据库失败"));
    let task_manager = TaskManager::new(db);

    // 1. 添加任务
    let task = task_manager.add_task("完成项目报告", 2, None).await.expect("添加任务失败");
    assert_eq!(task.title, "完成项目报告");
    assert_eq!(task.priority, 2);
    assert!(!task.completed);
    println!("  ✓ 添加任务: id={}, title={}", task.id, task.title);

    // 2. 获取任务列表
    let tasks = task_manager.get_tasks().await.expect("获取任务列表失败");
    assert!(tasks.len() > 0);
    let found = tasks.iter().find(|t| t.id == task.id);
    assert!(found.is_some());
    println!("  ✓ 任务列表包含新任务: {} 个任务", tasks.len());

    // 3. 完成任务
    task_manager.complete_task(task.id, true).await.expect("完成任务失败");
    let updated_tasks = task_manager.get_tasks().await.expect("获取更新后的任务失败");
    let completed = updated_tasks.iter().find(|t| t.id == task.id).unwrap();
    assert!(completed.completed);
    println!("  ✓ 任务已标记完成");

    // 4. 删除任务
    task_manager.delete_task(task.id).await.expect("删除任务失败");
    let final_tasks = task_manager.get_tasks().await.expect("获取最终任务失败");
    let deleted = final_tasks.iter().find(|t| t.id == task.id);
    assert!(deleted.is_none());
    println!("  ✓ 任务已删除");

    println!("✅ 任务管理测试通过");
}

/// 测试宠物状态管理 - 真实调用
#[tokio::test]
async fn test_real_pet_state_management() {
    println!("🐱 开始真实宠物状态测试...");

    let pet_manager = PetManager::new();

    // 1. 获取初始状态
    let initial_mood = pet_manager.get_mood().await;
    assert_eq!(initial_mood, "happy");
    println!("  ✓ 初始情绪: {}", initial_mood);

    // 2. 模拟专注开始
    pet_manager.on_focus_start().await;
    let focused_mood = pet_manager.get_mood().await;
    assert_eq!(focused_mood, "focused");
    println!("  ✓ 专注开始后情绪: {}", focused_mood);

    // 3. 模拟休息
    pet_manager.on_rest().await;
    let resting_mood = pet_manager.get_mood().await;
    assert_eq!(resting_mood, "sleeping");
    println!("  ✓ 休息时情绪: {}", resting_mood);

    // 4. 模拟专注完成
    pet_manager.on_focus_complete().await;
    let happy_mood = pet_manager.get_mood().await;
    assert_eq!(happy_mood, "happy");
    println!("  ✓ 专注完成后情绪: {}", happy_mood);

    // 5. 模拟聊天开始
    pet_manager.on_chat_start().await;
    let listening_mood = pet_manager.get_mood().await;
    assert_eq!(listening_mood, "listening");
    println!("  ✓ 聊天时情绪: {}", listening_mood);

    // 6. 模拟 AI 思考
    pet_manager.on_ai_thinking().await;
    let thinking_mood = pet_manager.get_mood().await;
    assert_eq!(thinking_mood, "thinking");
    println!("  ✓ AI 思考时情绪: {}", thinking_mood);

    // 7. 测试皮肤切换
    pet_manager.set_skin_id("skin_02").await;
    let skin_id = pet_manager.get_skin_id().await;
    assert_eq!(skin_id, "skin_02");
    println!("  ✓ 皮肤已切换: {}", skin_id);

    println!("✅ 宠物状态测试通过");
}

/// 测试成就系统 - 真实调用
#[tokio::test]
async fn test_real_achievement_system() {
    println!("🏆 开始真实成就系统测试...");

    let db = Database::new_in_memory().expect("创建数据库失败");

    // 1. 获取成就列表（已在 init_tables 中初始化）
    let achievements = db.get_achievements().expect("获取成就列表失败");
    assert!(achievements.len() > 0);
    println!("  ✓ 成就数量: {}", achievements.len());

    // 3. 检查初始状态（所有成就未解锁）
    let unlocked = achievements.iter().filter(|a| a.unlocked_at.is_some()).count();
    assert_eq!(unlocked, 0);
    println!("  ✓ 初始状态: {} 个已解锁", unlocked);

    // 4. 更新专注统计
    let stats = db.update_focus_stats(1500).expect("更新统计失败");
    println!("  ✓ 专注统计已更新");

    // 5. 检查成就解锁
    let new_achievements = db.check_achievements(&stats).expect("检查成就失败");
    println!("  ✓ 成就检查完成: {} 个新成就解锁", new_achievements.len());

    // 6. 获取专注统计
    let focus_stats = db.get_focus_stats().expect("获取统计失败");
    assert!(focus_stats.total_pomodoros >= 0);
    assert!(focus_stats.total_focus_seconds >= 0);
    println!("  ✓ 统计数据: {} 个番茄, {} 秒专注", focus_stats.total_pomodoros, focus_stats.total_focus_seconds);

    println!("✅ 成就系统测试通过");
}

/// 测试设置管理 - 真实调用
#[tokio::test]
async fn test_real_settings_management() {
    println!("⚙️ 开始真实设置管理测试...");

    let db = Database::new_in_memory().expect("创建数据库失败");

    // 1. 初始设置列表为空
    let initial_settings = db.get_all_settings().expect("获取设置失败");
    assert_eq!(initial_settings.len(), 0);
    println!("  ✓ 初始设置为空");

    // 2. 添加设置
    db.set_setting("focusDuration", "25").expect("添加设置失败");
    db.set_setting("language", "zh").expect("添加语言失败");
    db.set_setting("monitorEnabled", "true").expect("添加监听设置失败");
    println!("  ✓ 已添加 3 个设置");

    // 3. 获取设置列表
    let settings = db.get_all_settings().expect("获取设置失败");
    assert_eq!(settings.len(), 3);
    println!("  ✓ 设置数量: {}", settings.len());

    // 4. 修改设置
    db.set_setting("focusDuration", "30").expect("修改设置失败");
    db.set_setting("language", "en").expect("修改语言失败");
    println!("  ✓ 设置已修改");

    // 5. 验证设置生效
    let updated_settings = db.get_all_settings().expect("获取更新后的设置失败");
    let focus_setting = updated_settings.iter().find(|s| s.key == "focusDuration");
    assert!(focus_setting.is_some());
    assert_eq!(focus_setting.unwrap().value, "30");

    let lang_setting = updated_settings.iter().find(|s| s.key == "language");
    assert!(lang_setting.is_some());
    assert_eq!(lang_setting.unwrap().value, "en");
    println!("  ✓ 设置验证通过");

    println!("✅ 设置管理测试通过");
}

/// 测试聊天消息管理 - 真实调用
#[tokio::test]
async fn test_real_chat_management() {
    println!("💬 开始真实聊天管理测试...");

    let db = Database::new_in_memory().expect("创建数据库失败");

    // 1. 添加用户消息
    db.add_chat_message("user", "你好，番茄猫！").expect("添加用户消息失败");
    println!("  ✓ 用户消息已添加");

    // 2. 添加助手回复
    db.add_chat_message("assistant", "你好！我是番茄猫，很高兴见到你！🐱").expect("添加助手消息失败");
    println!("  ✓ 助手消息已添加");

    // 3. 获取聊天历史
    let history = db.get_chat_history(10).expect("获取聊天历史失败");
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].role, "user");
    assert_eq!(history[1].role, "assistant");
    println!("  ✓ 聊天历史: {} 条消息", history.len());

    // 4. 清空聊天历史
    db.clear_chat_history().expect("清空聊天历史失败");
    let empty_history = db.get_chat_history(10).expect("获取清空后的历史失败");
    assert_eq!(empty_history.len(), 0);
    println!("  ✓ 聊天历史已清空");

    println!("✅ 聊天管理测试通过");
}

/// 测试黑名单管理 - 真实调用
#[tokio::test]
async fn test_real_blocklist_management() {
    println!("🚫 开始真实黑名单管理测试...");

    let db = Database::new_in_memory().expect("创建数据库失败");

    // 1. 获取初始黑名单
    let initial_list = db.get_blocklist().expect("获取黑名单失败");
    assert_eq!(initial_list.len(), 0);
    println!("  ✓ 初始黑名单为空");

    // 2. 添加到黑名单
    db.add_to_blocklist("WeChat.exe").expect("添加到黑名单失败");
    db.add_to_blocklist("QQ.exe").expect("添加到黑名单失败");
    println!("  ✓ 已添加 2 个应用到黑名单");

    // 3. 获取更新后的黑名单
    let updated_list = db.get_blocklist().expect("获取更新后的黑名单失败");
    assert_eq!(updated_list.len(), 2);
    assert!(updated_list.iter().any(|item| item.process_name == "WeChat.exe"));
    assert!(updated_list.iter().any(|item| item.process_name == "QQ.exe"));
    println!("  ✓ 黑名单验证通过: {} 个应用", updated_list.len());

    // 4. 从黑名单移除
    db.remove_from_blocklist("WeChat.exe").expect("从黑名单移除失败");
    let final_list = db.get_blocklist().expect("获取最终黑名单失败");
    assert_eq!(final_list.len(), 1);
    assert!(!final_list.iter().any(|item| item.process_name == "WeChat.exe"));
    println!("  ✓ WeChat.exe 已从黑名单移除");

    println!("✅ 黑名单管理测试通过");
}

/// 测试专注记录管理 - 真实调用
#[tokio::test]
async fn test_real_pomodoro_record_management() {
    println!("🍅 开始真实专注记录管理测试...");

    let db = Database::new_in_memory().expect("创建数据库失败");

    // 1. 添加专注记录
    let record_id = db.add_pomodoro_record(1500).expect("添加记录失败");
    assert!(record_id > 0);
    println!("  ✓ 添加专注记录: id={}", record_id);

    // 2. 完成专注记录
    db.complete_pomodoro_record(record_id, 0).expect("完成记录失败");
    println!("  ✓ 专注记录已完成");

    // 3. 获取今日专注次数
    let today_count = db.get_today_pomodoro_count().expect("获取今日次数失败");
    assert!(today_count >= 0);
    println!("  ✓ 今日专注次数: {}", today_count);

    // 4. 获取专注历史
    let history = db.get_pomodoro_history(10).expect("获取历史失败");
    assert!(history.len() > 0);
    let record = history.iter().find(|r| r.id == record_id);
    assert!(record.is_some());
    println!("  ✓ 专注历史: {} 条记录", history.len());

    println!("✅ 专注记录管理测试通过");
}

/// 测试完整用户会话 - 真实调用
#[tokio::test]
async fn test_real_full_user_session() {
    println!("🎯 开始真实完整用户会话测试...");

    // 初始化组件
    let db = Arc::new(Database::new_in_memory().expect("创建数据库失败"));
    let pet_manager = PetManager::new();
    let task_manager = TaskManager::new(db.clone());

    println!("  ✓ 所有组件初始化成功");

    // 1. 检查宠物状态
    let mood = pet_manager.get_mood().await;
    assert_eq!(mood, "happy");
    println!("  ✓ 宠物情绪: {}", mood);

    // 2. 添加任务
    let task = task_manager.add_task("完成端到端测试", 1, None).await.expect("添加任务失败");
    println!("  ✓ 添加任务: {}", task.title);

    // 3. 模拟专注开始
    pet_manager.on_focus_start().await;
    println!("  ✓ 开始专注，宠物情绪: {}", pet_manager.get_mood().await);

    // 4. 记录专注会话
    let record_id = db.add_pomodoro_record(1500).expect("添加记录失败");
    db.complete_pomodoro_record(record_id, 0).expect("完成记录失败");
    println!("  ✓ 专注记录已保存");

    // 5. 模拟专注完成
    pet_manager.on_focus_complete().await;
    println!("  ✓ 专注完成，宠物情绪: {}", pet_manager.get_mood().await);

    // 6. 检查成就
    let stats = db.update_focus_stats(1500).expect("更新统计失败");
    let new_achievements = db.check_achievements(&stats).expect("检查成就失败");
    println!("  ✓ 已解锁成就: {} 个", new_achievements.len());

    // 7. 完成任务
    task_manager.complete_task(task.id, true).await.expect("完成任务失败");
    println!("  ✓ 任务已完成: {}", task.title);

    // 8. 查看统计
    let focus_stats = db.get_focus_stats().expect("获取统计失败");
    println!("  ✓ 总番茄数: {}", focus_stats.total_pomodoros);

    // 9. 清理
    task_manager.delete_task(task.id).await.expect("删除任务失败");
    println!("  ✓ 任务已清理");

    println!("✅ 完整用户会话测试通过");
}

/// 测试数据持久化 - 真实调用
#[tokio::test]
async fn test_real_data_persistence() {
    println!("💾 开始真实数据持久化测试...");

    let db = Database::new_in_memory().expect("创建数据库失败");

    // 1. 测试任务持久化
    let task_id = db.add_task("持久化测试任务", 1, None).expect("添加任务失败");
    let tasks = db.get_tasks().expect("获取任务失败");
    assert!(tasks.iter().any(|t| t.id == task_id));
    println!("  ✓ 任务数据持久化成功");

    // 2. 测试设置持久化
    db.set_setting("testKey", "testValue").expect("保存设置失败");
    let settings = db.get_all_settings().expect("获取设置失败");
    assert!(settings.iter().any(|s| s.key == "testKey" && s.value == "testValue"));
    println!("  ✓ 设置数据持久化成功");

    // 3. 测试聊天记录持久化
    db.add_chat_message("user", "测试消息").expect("添加消息失败");
    let history = db.get_chat_history(10).expect("获取历史失败");
    assert!(history.iter().any(|m| m.content == "测试消息"));
    println!("  ✓ 聊天记录持久化成功");

    // 4. 测试专注记录持久化
    let record_id = db.add_pomodoro_record(1500).expect("添加记录失败");
    let records = db.get_pomodoro_history(10).expect("获取记录失败");
    assert!(records.iter().any(|r| r.id == record_id));
    println!("  ✓ 专注记录持久化成功");

    // 5. 测试黑名单持久化
    db.add_to_blocklist("TestApp.exe").expect("添加黑名单失败");
    let blocklist = db.get_blocklist().expect("获取黑名单失败");
    assert!(blocklist.iter().any(|b| b.process_name == "TestApp.exe"));
    println!("  ✓ 黑名单数据持久化成功");

    println!("✅ 数据持久化测试通过");
}
