use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use sysinfo::System;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

use crate::pet::PetManager;

/// Claude Code 进程状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ClaudeCodeState {
    /// 未检测到 Claude Code 进程
    NotRunning,
    /// 运行中 + CPU 空闲（≤10%）
    Idle,
    /// 运行中 + CPU 活跃（>10%）
    Active,
    /// 运行中 + 空闲超过阈值
    LongIdle,
    /// 进程异常消失
    Crashed,
}

/// Claude Code 状态详情（发给前端）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCodeStatus {
    pub state: ClaudeCodeState,
    pub cpu_usage: f32,
    pub idle_seconds: u64,
    pub process_name: String,
}

/// Claude Code 进程监控器
pub struct ClaudeCodeMonitor {
    app_handle: AppHandle,
    pet_manager: Arc<PetManager>,
    is_running: Arc<Mutex<bool>>,
    cancel_token: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
}

impl ClaudeCodeMonitor {
    /// 创建新的监控器
    pub fn new(app_handle: AppHandle, pet_manager: Arc<PetManager>) -> Self {
        Self {
            app_handle,
            pet_manager,
            is_running: Arc::new(Mutex::new(false)),
            cancel_token: Arc::new(Mutex::new(None)),
        }
    }

    /// 启动监控（每 5 秒轮询）
    pub async fn start(&self) -> Result<(), String> {
        let mut is_running = self.is_running.lock().await;
        if *is_running {
            return Ok(());
        }
        *is_running = true;
        drop(is_running);

        let pet_manager = self.pet_manager.clone();
        let is_running = self.is_running.clone();
        let app_handle = self.app_handle.clone();
        let cancel_token = self.cancel_token.clone();

        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        *cancel_token.lock().await = Some(tx);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            let mut rx = rx;

            // 上次检测到 Claude Code 还在运行
            let mut was_running = false;
            // 上次 CPU 空闲开始时间
            let mut idle_since: Option<Instant> = None;

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let running = is_running.lock().await;
                        if !*running {
                            break;
                        }
                        drop(running);

                        // 检查手动覆盖期内不自动切换
                        if pet_manager.is_overridden().await {
                            continue;
                        }

                        // 扫描 Claude Code 进程
                        let status = Self::detect_claude_code();

                        match &status.state {
                            ClaudeCodeState::NotRunning => {
                                if was_running {
                                    // 进程异常消失
                                    pet_manager.set_mood("error").await;
                                    was_running = false;
                                    idle_since = None;
                                } else {
                                    // 一直未运行 → Sleeping
                                    pet_manager.set_mood("sleeping").await;
                                }
                            }
                            ClaudeCodeState::Active => {
                                was_running = true;
                                idle_since = None;
                                pet_manager.set_mood("thinking").await;
                            }
                            ClaudeCodeState::Idle => {
                                was_running = true;
                                // 开始空闲计时
                                if idle_since.is_none() {
                                    idle_since = Some(Instant::now());
                                }
                                // 检查空闲是否超过 10 分钟
                                if let Some(since) = idle_since {
                                    if since.elapsed() >= Duration::from_secs(600) {
                                        pet_manager.set_mood("tired").await;
                                    } else {
                                        pet_manager.set_mood("happy").await;
                                    }
                                }
                            }
                            _ => {}
                        }

                        // 发送状态详情给前端
                        let _ = app_handle.emit("claude-code-status-changed", &status);
                    }
                    _ = &mut rx => {
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// 停止监控
    pub async fn stop(&self) -> Result<(), String> {
        let mut is_running = self.is_running.lock().await;
        *is_running = false;

        if let Some(tx) = self.cancel_token.lock().await.take() {
            let _ = tx.send(());
        }

        Ok(())
    }

    /// 获取监控状态
    pub async fn is_running(&self) -> bool {
        *self.is_running.lock().await
    }

    /// 检测本机 Claude Code 进程状态（公开供 IPC 调用）
    pub fn detect_claude_code() -> ClaudeCodeStatus {
        let mut sys = System::new_all();
        sys.refresh_all();

        // 查找 claude / claude.exe 进程
        let mut found_process: Option<(&str, f32)> = None;

        for (_pid, process) in sys.processes() {
            let name = process.name().to_lowercase();
            if name == "claude" || name == "claude.exe" {
                let cpu = process.cpu_usage();
                found_process = Some((process.name(), cpu));
                break;
            }
        }

        match found_process {
            Some((name, cpu)) => {
                if cpu > 10.0 {
                    ClaudeCodeStatus {
                        state: ClaudeCodeState::Active,
                        cpu_usage: cpu,
                        idle_seconds: 0,
                        process_name: name.to_string(),
                    }
                } else {
                    ClaudeCodeStatus {
                        state: ClaudeCodeState::Idle,
                        cpu_usage: cpu,
                        idle_seconds: 0,
                        process_name: name.to_string(),
                    }
                }
            }
            None => ClaudeCodeStatus {
                state: ClaudeCodeState::NotRunning,
                cpu_usage: 0.0,
                idle_seconds: 0,
                process_name: String::new(),
            },
        }
    }
}
