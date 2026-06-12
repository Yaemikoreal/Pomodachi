import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export type TimerMode = 'focus' | 'shortBreak' | 'longBreak';

export interface TimerState {
  mode: TimerMode;
  duration: number;
  remaining: number;
  isRunning: boolean;
  completedPomodoros: number;
}

export function useTimer() {
  const [timerState, setTimerState] = useState<TimerState>({
    mode: 'focus',
    duration: 25 * 60,
    remaining: 25 * 60,
    isRunning: false,
    completedPomodoros: 0,
  });

  // 获取初始状态
  useEffect(() => {
    const fetchState = async () => {
      try {
        const state = await invoke<TimerState>('get_timer_state');
        setTimerState(state);
      } catch (err) {
        console.error('获取计时器状态失败:', err);
      }
    };
    fetchState();
  }, []);

  // 监听计时器事件
  useEffect(() => {
    const unlistenTick = listen<TimerState>('timer-tick', (event) => {
      setTimerState(event.payload);
    });

    const unlistenComplete = listen<TimerState>('timer-complete', (event) => {
      setTimerState(event.payload);
    });

    return () => {
      unlistenTick.then((fn) => fn());
      unlistenComplete.then((fn) => fn());
    };
  }, []);

  // 开始计时
  const start = useCallback(async () => {
    try {
      const state = await invoke<TimerState>('start_timer');
      setTimerState(state);
    } catch (err) {
      console.error('启动计时器失败:', err);
    }
  }, []);

  // 暂停计时
  const pause = useCallback(async () => {
    try {
      const state = await invoke<TimerState>('pause_timer');
      setTimerState(state);
    } catch (err) {
      console.error('暂停计时器失败:', err);
    }
  }, []);

  // 重置计时
  const reset = useCallback(async () => {
    try {
      const state = await invoke<TimerState>('reset_timer');
      setTimerState(state);
    } catch (err) {
      console.error('重置计时器失败:', err);
    }
  }, []);

  // 跳过当前阶段
  const skip = useCallback(async () => {
    try {
      const state = await invoke<TimerState>('skip_timer');
      setTimerState(state);
    } catch (err) {
      console.error('跳过计时器失败:', err);
    }
  }, []);

  // 设置模式
  const setMode = useCallback(async (mode: TimerMode) => {
    try {
      const state = await invoke<TimerState>('set_timer_mode', { mode });
      setTimerState(state);
    } catch (err) {
      console.error('设置计时器模式失败:', err);
    }
  }, []);

  // 格式化时间
  const formatTime = useCallback((seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  }, []);

  // 计算进度百分比
  const progress = timerState.duration > 0
    ? ((timerState.duration - timerState.remaining) / timerState.duration) * 100
    : 0;

  return {
    ...timerState,
    start,
    pause,
    reset,
    skip,
    setMode,
    formatTime,
    progress,
    formattedTime: formatTime(timerState.remaining),
  };
}
