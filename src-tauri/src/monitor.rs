use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use sysinfo::System;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

/// 分心检测事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DistractionEvent {
    pub process_name: String,
    pub timestamp: String,
}

/// 进程监听器
pub struct ProcessMonitor {
    blocklist: Arc<Mutex<Vec<String>>>,
    is_monitoring: Arc<Mutex<bool>>,
    app_handle: AppHandle,
    cancel_token: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
    /// 上次分心检测时间（用于冷却）
    last_distraction_time: Arc<Mutex<Option<Instant>>>,
    /// 分心冷却秒数
    cooldown_secs: Arc<Mutex<u64>>,
}

impl ProcessMonitor {
    /// 创建新的监听器
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            blocklist: Arc::new(Mutex::new(Vec::new())),
            is_monitoring: Arc::new(Mutex::new(false)),
            app_handle,
            cancel_token: Arc::new(Mutex::new(None)),
            last_distraction_time: Arc::new(Mutex::new(None)),
            cooldown_secs: Arc::new(Mutex::new(30)),
        }
    }

    /// 设置分心冷却时间（秒）
    pub async fn set_cooldown(&self, seconds: u64) {
        *self.cooldown_secs.lock().await = seconds;
    }

    /// 检查是否在冷却期内
    async fn is_in_cooldown(&self) -> bool {
        let cd = *self.cooldown_secs.lock().await;
        if let Some(last_time) = *self.last_distraction_time.lock().await {
            return last_time.elapsed().as_secs() < cd;
        }
        false
    }

    /// 更新黑名单
    pub async fn update_blocklist(&self, new_blocklist: Vec<String>) {
        let mut blocklist = self.blocklist.lock().await;
        *blocklist = new_blocklist;
    }

    /// 获取当前前台窗口进程名（Windows）
    #[cfg(target_os = "windows")]
    pub fn get_foreground_process() -> Option<String> {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd == HWND(0) {
                return None;
            }

            let mut process_id: u32 = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut process_id));

            if process_id == 0 {
                return None;
            }

            let mut sys = System::new();
            // 只刷新目标 PID，避免每次遍历所有进程
            sys.refresh_process(sysinfo::Pid::from_u32(process_id));

            if let Some(process) = sys.process(sysinfo::Pid::from_u32(process_id)) {
                return Some(process.name().to_string());
            }
        }

        None
    }

    /// 获取当前前台窗口进程名（非 Windows 平台）
    #[cfg(not(target_os = "windows"))]
    pub fn get_foreground_process() -> Option<String> {
        // TODO: 实现 macOS/Linux 前台窗口检测
        None
    }

    /// 检查当前进程是否在黑名单中
    pub async fn is_distracted(&self) -> Option<String> {
        if let Some(process_name) = Self::get_foreground_process() {
            let blocklist = self.blocklist.lock().await;
            if blocklist.iter().any(|p| p.to_lowercase() == process_name.to_lowercase()) {
                return Some(process_name);
            }
        }
        None
    }

    /// 开始监听
    pub async fn start_monitoring(&self) -> Result<(), String> {
        let mut is_monitoring = self.is_monitoring.lock().await;
        if *is_monitoring {
            return Ok(());
        }
        *is_monitoring = true;
        drop(is_monitoring);

        let blocklist = self.blocklist.clone();
        let is_monitoring = self.is_monitoring.clone();
        let app_handle = self.app_handle.clone();
        let cancel_token = self.cancel_token.clone();
        let last_distraction_time = self.last_distraction_time.clone();
        let cooldown_secs = self.cooldown_secs.clone();

        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        *cancel_token.lock().await = Some(tx);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(3));
            let mut rx = rx;

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let monitoring = is_monitoring.lock().await;
                        if !*monitoring {
                            break;
                        }
                        drop(monitoring);

                        if let Some(process_name) = Self::get_foreground_process() {
                            let bl = blocklist.lock().await;
                            let is_blocked = bl.iter().any(|p| p.to_lowercase() == process_name.to_lowercase());
                            drop(bl);

                            if is_blocked {
                                // 检查冷却期：冷却期内不重复触发分心事件
                                let cd = *cooldown_secs.lock().await;
                                let in_cooldown = if let Some(last_time) = *last_distraction_time.lock().await {
                                    last_time.elapsed().as_secs() < cd
                                } else {
                                    false
                                };

                                if !in_cooldown {
                                    // 更新最后一次分心时间
                                    *last_distraction_time.lock().await = Some(std::time::Instant::now());

                                    let event = DistractionEvent {
                                        process_name,
                                        timestamp: chrono::Utc::now().to_rfc3339(),
                                    };
                                    let _ = app_handle.emit("distraction-detected", &event);
                                }
                            }
                        }
                    }
                    _ = &mut rx => {
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// 停止监听
    pub async fn stop_monitoring(&self) -> Result<(), String> {
        let mut is_monitoring = self.is_monitoring.lock().await;
        *is_monitoring = false;

        if let Some(tx) = self.cancel_token.lock().await.take() {
            let _ = tx.send(());
        }

        Ok(())
    }

    /// 获取监听状态
    pub async fn is_running(&self) -> bool {
        *self.is_monitoring.lock().await
    }
}
