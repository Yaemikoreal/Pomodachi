pub mod ai;
pub mod db;
mod monitor;
mod notification;
pub mod pet;
pub mod task;
pub mod timer;

use ai::AiClient;
use db::{Achievement, AiConfig, BlocklistItem, ChatMessage, Database, FocusStats, PomodoroRecord, Task};
use monitor::{DistractionEvent, ProcessMonitor};
use pet::PetManager;
use std::sync::Arc;
use tauri::{Emitter, Listener, Manager};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use task::TaskManager;
use timer::{PomodoroTimer, TimerMode, TimerState};

/// 应用状态
pub struct AppState {
    pub db: Arc<Database>,
    pub timer: PomodoroTimer,
    pub pet_manager: PetManager,
    pub task_manager: TaskManager,
    pub process_monitor: ProcessMonitor,
    pub ai_client: AiClient,
}

// ==================== 计时器命令 ====================

#[tauri::command]
async fn get_timer_state(state: tauri::State<'_, AppState>) -> Result<TimerState, String> {
    Ok(state.timer.get_state().await)
}

#[tauri::command]
async fn start_timer(state: tauri::State<'_, AppState>) -> Result<TimerState, String> {
    let timer_state = state.timer.start().await?;

    // 自动更新宠物情绪（根据计时模式）
    match timer_state.mode {
        TimerMode::Focus => {
            state.pet_manager.on_focus_start().await;
        }
        TimerMode::ShortBreak | TimerMode::LongBreak => {
            state.pet_manager.on_rest().await;
        }
    }

    Ok(timer_state)
}

#[tauri::command]
async fn pause_timer(state: tauri::State<'_, AppState>) -> Result<TimerState, String> {
    state.timer.pause().await
}

#[tauri::command]
async fn reset_timer(state: tauri::State<'_, AppState>) -> Result<TimerState, String> {
    state.timer.reset().await
}

#[tauri::command]
async fn skip_timer(state: tauri::State<'_, AppState>) -> Result<TimerState, String> {
    let timer_state = state.timer.skip().await?;

    // 跳过后进入新模式，自动更新宠物情绪
    match timer_state.mode {
        TimerMode::Focus => {
            state.pet_manager.on_focus_start().await;
        }
        TimerMode::ShortBreak | TimerMode::LongBreak => {
            state.pet_manager.on_rest().await;
        }
    }

    Ok(timer_state)
}

#[tauri::command]
async fn set_timer_mode(
    state: tauri::State<'_, AppState>,
    mode: TimerMode,
) -> Result<TimerState, String> {
    let timer_state = state.timer.set_mode(mode).await?;

    // 切换模式后更新宠物情绪
    match timer_state.mode {
        TimerMode::Focus => {
            state.pet_manager.on_focus_start().await;
        }
        TimerMode::ShortBreak | TimerMode::LongBreak => {
            state.pet_manager.on_rest().await;
        }
    }

    Ok(timer_state)
}

// ==================== 宠物命令 ====================

#[tauri::command]
async fn get_pet_mood(state: tauri::State<'_, AppState>) -> Result<String, String> {
    Ok(state.pet_manager.get_mood().await)
}

#[tauri::command]
async fn set_pet_mood(
    state: tauri::State<'_, AppState>,
    mood: String,
) -> Result<(), String> {
    state.pet_manager.set_mood(&mood).await;
    Ok(())
}

#[tauri::command]
async fn get_pet_skin(state: tauri::State<'_, AppState>) -> Result<String, String> {
    Ok(state.pet_manager.get_skin_id().await)
}

#[tauri::command]
async fn set_pet_skin(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    skin_id: String,
) -> Result<(), String> {
    // 更新内存
    state.pet_manager.set_skin_id(&skin_id).await;
    // 持久化到数据库
    state.db.set_skin_id(&skin_id).map_err(|e| e.to_string())?;
    // 通知前端皮肤已变更
    let _ = app.emit("pet-skin-changed", &skin_id);
    Ok(())
}

// ==================== 专注记录命令 ====================

#[tauri::command]
async fn add_pomodoro_record(
    state: tauri::State<'_, AppState>,
    duration: i32,
) -> Result<i64, String> {
    state.db.add_pomodoro_record(duration).map_err(|e| e.to_string())
}

#[tauri::command]
async fn complete_pomodoro_record(
    state: tauri::State<'_, AppState>,
    id: i64,
    distraction_count: i32,
) -> Result<(), String> {
    state.db.complete_pomodoro_record(id, distraction_count).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_pomodoro_history(
    state: tauri::State<'_, AppState>,
    limit: i32,
) -> Result<Vec<PomodoroRecord>, String> {
    state.db.get_pomodoro_history(limit).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_today_pomodoro_count(state: tauri::State<'_, AppState>) -> Result<i32, String> {
    state.db.get_today_pomodoro_count().map_err(|e| e.to_string())
}

// ==================== 黑名单命令 ====================

#[tauri::command]
async fn get_blocklist(state: tauri::State<'_, AppState>) -> Result<Vec<BlocklistItem>, String> {
    state.db.get_blocklist().map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_to_blocklist(
    state: tauri::State<'_, AppState>,
    process_name: String,
) -> Result<(), String> {
    state.db.add_to_blocklist(&process_name).map_err(|e| e.to_string())?;

    // 更新监听器黑名单
    let blocklist = state.db.get_blocklist().map_err(|e| e.to_string())?;
    let process_names: Vec<String> = blocklist.into_iter().map(|item| item.process_name).collect();
    state.process_monitor.update_blocklist(process_names).await;

    Ok(())
}

#[tauri::command]
async fn remove_from_blocklist(
    state: tauri::State<'_, AppState>,
    process_name: String,
) -> Result<(), String> {
    state.db.remove_from_blocklist(&process_name).map_err(|e| e.to_string())?;

    // 更新监听器黑名单
    let blocklist = state.db.get_blocklist().map_err(|e| e.to_string())?;
    let process_names: Vec<String> = blocklist.into_iter().map(|item| item.process_name).collect();
    state.process_monitor.update_blocklist(process_names).await;

    Ok(())
}

// ==================== 监听器命令 ====================

#[tauri::command]
async fn start_monitoring(state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.process_monitor.start_monitoring().await
}

#[tauri::command]
async fn stop_monitoring(state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.process_monitor.stop_monitoring().await
}

#[tauri::command]
async fn is_monitoring(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    Ok(state.process_monitor.is_running().await)
}

// ==================== AI 聊天命令 ====================

#[tauri::command]
async fn send_chat_message(
    state: tauri::State<'_, AppState>,
    message: String,
) -> Result<String, String> {
    // 保存用户消息
    state
        .db
        .add_chat_message("user", &message)
        .map_err(|e| e.to_string())?;

    // 获取宠物情绪和今日专注数
    let mood = state.pet_manager.get_mood().await;
    let today_count = state.db.get_today_pomodoro_count().map_err(|e| e.to_string())?;

    // 生成系统提示词
    let system_prompt = AiClient::generate_system_prompt(&mood, today_count);

    // 获取聊天历史（最近 20 条）
    let history = state.db.get_chat_history(20).map_err(|e| e.to_string())?;

    // 构建消息列表（包含系统提示）
    let mut messages = vec![ChatMessage {
        id: 0,
        role: "system".to_string(),
        content: system_prompt,
        created_at: String::new(),
    }];
    messages.extend(history);

    // 调用 AI 获取回复
    let response = state.ai_client.chat(messages).await?;

    // 保存助手回复
    state
        .db
        .add_chat_message("assistant", &response)
        .map_err(|e| e.to_string())?;

    Ok(response)
}

#[tauri::command]
async fn get_chat_history(
    state: tauri::State<'_, AppState>,
    limit: Option<i32>,
) -> Result<Vec<ChatMessage>, String> {
    state
        .db
        .get_chat_history(limit.unwrap_or(50))
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn clear_chat_history(state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.db.clear_chat_history().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_ai_config(state: tauri::State<'_, AppState>) -> Result<AiConfig, String> {
    state.db.get_ai_config().map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_ai_config(
    state: tauri::State<'_, AppState>,
    max_turns: i32,
    max_budget_usd: Option<f64>,
) -> Result<(), String> {
    state
        .db
        .update_ai_config(max_turns, max_budget_usd)
        .map_err(|e| e.to_string())?;

    // 重新加载配置到 AI 客户端
    let new_config = state.db.get_ai_config().map_err(|e| e.to_string())?;
    state.ai_client.update_config(new_config).await;

    Ok(())
}

/// 清空 AI 对话会话
#[tauri::command]
async fn clear_ai_session(state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.ai_client.clear_session().await;
    Ok(())
}

// ==================== 任务命令 ====================

#[tauri::command]
async fn get_tasks(state: tauri::State<'_, AppState>) -> Result<Vec<Task>, String> {
    state.task_manager.get_tasks().await
}

#[tauri::command]
async fn add_task(
    state: tauri::State<'_, AppState>,
    title: String,
    priority: i32,
    due_date: Option<String>,
) -> Result<Task, String> {
    state
        .task_manager
        .add_task(&title, priority, due_date.as_deref())
        .await
}

#[tauri::command]
async fn update_task(
    state: tauri::State<'_, AppState>,
    id: i64,
    title: String,
    priority: i32,
    due_date: Option<String>,
) -> Result<(), String> {
    state
        .task_manager
        .update_task(id, &title, priority, due_date.as_deref())
        .await
}

#[tauri::command]
async fn complete_task(
    state: tauri::State<'_, AppState>,
    id: i64,
    completed: bool,
) -> Result<(), String> {
    state.task_manager.complete_task(id, completed).await
}

#[tauri::command]
async fn delete_task(state: tauri::State<'_, AppState>, id: i64) -> Result<(), String> {
    state.task_manager.delete_task(id).await
}

// ==================== 设置命令 ====================

#[tauri::command]
async fn get_setting(
    state: tauri::State<'_, AppState>,
    key: String,
) -> Result<Option<String>, String> {
    state.db.get_setting(&key).map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_setting(
    state: tauri::State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), String> {
    state.db.set_setting(&key, &value).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_all_settings(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<db::AppSetting>, String> {
    state.db.get_all_settings().map_err(|e| e.to_string())
}

// ==================== 成就/统计命令 ====================

#[tauri::command]
async fn get_focus_stats(state: tauri::State<'_, AppState>) -> Result<FocusStats, String> {
    state.db.get_focus_stats().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_achievements(state: tauri::State<'_, AppState>) -> Result<Vec<Achievement>, String> {
    state.db.get_achievements().map_err(|e| e.to_string())
}

// ==================== 应用控制命令 ====================

#[tauri::command]
async fn exit_app(app: tauri::AppHandle) -> Result<(), String> {
    app.exit(0);
    Ok(())
}

#[tauri::command]
async fn set_pet_window_size(
    app: tauri::AppHandle,
    width: f64,
    height: f64,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("pet") {
        window
            .set_size(tauri::LogicalSize::new(width, height))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // 初始化数据库
            let db = Arc::new(Database::new(app.handle()).expect("数据库初始化失败"));

            // 初始化计时器
            let timer = PomodoroTimer::new(app.handle().clone());

            // 初始化宠物管理器
            // 从数据库加载皮肤 ID
            let skin_id = db.get_skin_id().unwrap_or_else(|_| "firefly".to_string());
            let pet_manager = PetManager::new_with_skin(&skin_id);

            // 初始化任务管理器
            let task_manager = TaskManager::new(db.clone());

            // 初始化进程监听器
            let process_monitor = ProcessMonitor::new(app.handle().clone());

            // 初始化 AI 客户端
            let ai_config = db.get_ai_config().expect("获取 AI 配置失败");
            let ai_client = AiClient::new(ai_config);

            // 注册应用状态
            app.manage(AppState {
                db: db.clone(),
                timer,
                pet_manager,
                task_manager,
                process_monitor,
                ai_client,
            });

            // ==================== 事件监听器 ====================
            // 获取应用句柄（需要多个克隆用于多个监听器）
            let app_handle = app.handle().clone();
            let app_handle_2 = app_handle.clone();

            // 监听计时完成事件 → 记录专注 + 更新统计 + 检查成就
            let handle_1 = app_handle.clone();
            app_handle.listen("timer-complete", move |event| {
                if let Ok(timer_state) = serde_json::from_str::<TimerState>(event.payload()) {
                    if timer_state.mode == TimerMode::Focus {
                        // 发送 Windows 通知
                        notification::show_notification(
                            "🍅 专注完成",
                            "太棒了！继续保持！ 🐱",
                        )
                        .ok();

                        let handle = handle_1.clone();
                        tauri::async_runtime::spawn(async move {
                            let state = handle.state::<AppState>();
                            // 宠物情绪变 Happy
                            state.pet_manager.on_focus_complete().await;
                            // 记录专注完成
                            let _ = state
                                .db
                                .add_pomodoro_record(timer_state.duration as i32);
                            // 更新专注统计
                            if let Ok(stats) = state
                                .db
                                .update_focus_stats(timer_state.duration as i32)
                            {
                                // 检查是否有新成就解锁
                                if let Ok(new_achievements) = state
                                    .db
                                    .check_achievements(&stats)
                                {
                                    for ach in &new_achievements {
                                        let _ = handle.emit("achievement-unlocked", ach);
                                    }
                                }
                            }
                        });
                    }
                }
            });

            // 监听分心检测事件 → 仅记录日志
            app_handle_2.listen("distraction-detected", move |event| {
                if let Ok(distraction) =
                    serde_json::from_str::<DistractionEvent>(event.payload())
                {
                    println!(
                        "[分心检测] 检测到分心: {} ({})",
                        distraction.process_name, distraction.timestamp
                    );
                }
            });

            // ==================== 系统托盘 ====================
            // 创建托盘菜单
            let show_item = MenuItem::with_id(app, "show", "显示宠物", true, None::<&str>)?;
            let today_item = MenuItem::with_id(app, "today", "今日专注：0 个", false, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &today_item, &quit_item])?;

            // 构建托盘图标
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("pet") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("pet") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 计时器
            get_timer_state,
            start_timer,
            pause_timer,
            reset_timer,
            skip_timer,
            set_timer_mode,
            // 宠物
            get_pet_mood,
            set_pet_mood,
            get_pet_skin,
            set_pet_skin,
            // 专注记录
            add_pomodoro_record,
            complete_pomodoro_record,
            get_pomodoro_history,
            get_today_pomodoro_count,
            // 黑名单
            get_blocklist,
            add_to_blocklist,
            remove_from_blocklist,
            // 监听器
            start_monitoring,
            stop_monitoring,
            is_monitoring,
            // AI 聊天
            send_chat_message,
            get_chat_history,
            clear_chat_history,
            get_ai_config,
            update_ai_config,
            clear_ai_session,
            // 任务
            get_tasks,
            add_task,
            update_task,
            complete_task,
            delete_task,
            // 设置
            get_setting,
            set_setting,
            get_all_settings,
            // 成就/统计
            get_focus_stats,
            get_achievements,
            // 应用控制
            exit_app,
            set_pet_window_size,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
