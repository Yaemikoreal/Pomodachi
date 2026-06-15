use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

/// 计时器模式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TimerMode {
    Focus,
    ShortBreak,
    LongBreak,
}

/// 计时器状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerState {
    pub mode: TimerMode,
    pub duration: u32,
    pub remaining: u32,
    pub is_running: bool,
    pub completed_pomodoros: u32,
}

impl Default for TimerState {
    fn default() -> Self {
        Self {
            mode: TimerMode::Focus,
            duration: 25 * 60,
            remaining: 25 * 60,
            is_running: false,
            completed_pomodoros: 0,
        }
    }
}

/// 番茄钟引擎
pub struct PomodoroTimer {
    state: Arc<Mutex<TimerState>>,
    app_handle: AppHandle,
    cancel_token: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
}

impl PomodoroTimer {
    /// 创建新的计时器实例
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            state: Arc::new(Mutex::new(TimerState::default())),
            app_handle,
            cancel_token: Arc::new(Mutex::new(None)),
        }
    }

    /// 获取当前状态
    pub async fn get_state(&self) -> TimerState {
        self.state.lock().await.clone()
    }

    /// 开始计时
    pub async fn start(&self) -> Result<TimerState, String> {
        let mut state = self.state.lock().await;

        if state.is_running {
            return Ok(state.clone());
        }

        state.is_running = true;
        let state_clone = state.clone();
        drop(state);

        // 启动计时任务
        let state_ref = self.state.clone();
        let app_handle = self.app_handle.clone();
        let cancel_token = self.cancel_token.clone();

        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        *cancel_token.lock().await = Some(tx);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            let mut rx = rx;

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let mut state = state_ref.lock().await;
                        if !state.is_running {
                            break;
                        }

                        if state.remaining > 0 {
                            state.remaining -= 1;
                            let state_clone = state.clone();
                            let _ = app_handle.emit("timer-tick", &state_clone);
                        } else {
                            // 计时完成
                            state.is_running = false;
                            state.completed_pomodoros += 1;
                            let state_clone = state.clone();
                            let _ = app_handle.emit("timer-complete", &state_clone);
                            break;
                        }
                    }
                    _ = &mut rx => {
                        // 取消信号
                        break;
                    }
                }
            }
        });

        Ok(state_clone)
    }

    /// 暂停计时
    pub async fn pause(&self) -> Result<TimerState, String> {
        let mut state = self.state.lock().await;
        state.is_running = false;

        // 发送取消信号
        if let Some(tx) = self.cancel_token.lock().await.take() {
            let _ = tx.send(());
        }

        Ok(state.clone())
    }

    /// 重置当前阶段
    pub async fn reset(&self) -> Result<TimerState, String> {
        let mut state = self.state.lock().await;
        state.is_running = false;
        state.remaining = state.duration;

        // 发送取消信号
        if let Some(tx) = self.cancel_token.lock().await.take() {
            let _ = tx.send(());
        }

        let state_clone = state.clone();
        let _ = self.app_handle.emit("timer-tick", &state_clone);
        Ok(state_clone)
    }

    /// 跳过当前阶段
    pub async fn skip(&self) -> Result<TimerState, String> {
        let mut state = self.state.lock().await;
        state.is_running = false;

        // 发送取消信号
        if let Some(tx) = self.cancel_token.lock().await.take() {
            let _ = tx.send(());
        }

        // 切换到下一阶段
        Self::next_mode(&mut state);

        let state_clone = state.clone();
        let _ = self.app_handle.emit("timer-tick", &state_clone);
        Ok(state_clone)
    }

    /// 设置计时器模式
    pub async fn set_mode(&self, mode: TimerMode) -> Result<TimerState, String> {
        let mut state = self.state.lock().await;
        state.is_running = false;

        // 发送取消信号
        if let Some(tx) = self.cancel_token.lock().await.take() {
            let _ = tx.send(());
        }

        state.mode = mode.clone();
        state.duration = match mode {
            TimerMode::Focus => 25 * 60,
            TimerMode::ShortBreak => 5 * 60,
            TimerMode::LongBreak => 15 * 60,
        };
        state.remaining = state.duration;

        let state_clone = state.clone();
        let _ = self.app_handle.emit("timer-tick", &state_clone);
        Ok(state_clone)
    }

    /// 切换到下一阶段
    pub fn next_mode(state: &mut TimerState) {
        state.mode = match state.mode {
            TimerMode::Focus => {
                if state.completed_pomodoros % 4 == 0 {
                    TimerMode::LongBreak
                } else {
                    TimerMode::ShortBreak
                }
            }
            TimerMode::ShortBreak | TimerMode::LongBreak => TimerMode::Focus,
        };

        state.duration = match state.mode {
            TimerMode::Focus => 25 * 60,
            TimerMode::ShortBreak => 5 * 60,
            TimerMode::LongBreak => 15 * 60,
        };
        state.remaining = state.duration;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let state = TimerState::default();
        assert_eq!(state.mode, TimerMode::Focus);
        assert_eq!(state.duration, 25 * 60);
        assert_eq!(state.remaining, 25 * 60);
        assert!(!state.is_running);
        assert_eq!(state.completed_pomodoros, 0);
    }

    #[test]
    fn test_next_mode_focus_to_short_break() {
        let mut state = TimerState {
            mode: TimerMode::Focus,
            completed_pomodoros: 1,
            ..Default::default()
        };
        PomodoroTimer::next_mode(&mut state);
        assert_eq!(state.mode, TimerMode::ShortBreak);
        assert_eq!(state.duration, 5 * 60);
        assert_eq!(state.remaining, 5 * 60);
    }

    #[test]
    fn test_next_mode_focus_to_long_break() {
        let mut state = TimerState {
            mode: TimerMode::Focus,
            completed_pomodoros: 4,
            ..Default::default()
        };
        PomodoroTimer::next_mode(&mut state);
        assert_eq!(state.mode, TimerMode::LongBreak);
        assert_eq!(state.duration, 15 * 60);
        assert_eq!(state.remaining, 15 * 60);
    }

    #[test]
    fn test_next_mode_focus_to_long_break_multiple_of_4() {
        let mut state = TimerState {
            mode: TimerMode::Focus,
            completed_pomodoros: 8,
            ..Default::default()
        };
        PomodoroTimer::next_mode(&mut state);
        assert_eq!(state.mode, TimerMode::LongBreak);
    }

    #[test]
    fn test_next_mode_short_break_to_focus() {
        let mut state = TimerState {
            mode: TimerMode::ShortBreak,
            ..Default::default()
        };
        PomodoroTimer::next_mode(&mut state);
        assert_eq!(state.mode, TimerMode::Focus);
        assert_eq!(state.duration, 25 * 60);
    }

    #[test]
    fn test_next_mode_long_break_to_focus() {
        let mut state = TimerState {
            mode: TimerMode::LongBreak,
            ..Default::default()
        };
        PomodoroTimer::next_mode(&mut state);
        assert_eq!(state.mode, TimerMode::Focus);
    }

    #[test]
    fn test_set_mode_durations() {
        let test_cases = vec![
            (TimerMode::Focus, 25 * 60),
            (TimerMode::ShortBreak, 5 * 60),
            (TimerMode::LongBreak, 15 * 60),
        ];

        for (mode, expected_duration) in test_cases {
            let mut state = TimerState::default();
            state.mode = mode;
            state.duration = expected_duration;
            state.remaining = expected_duration;
            assert_eq!(state.duration, expected_duration);
            assert_eq!(state.remaining, expected_duration);
        }
    }
}
