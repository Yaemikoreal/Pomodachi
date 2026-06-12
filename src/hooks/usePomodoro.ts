import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface PomodoroRecord {
  id: number;
  startedAt: string;
  finishedAt: string | null;
  duration: number;
  completed: boolean;
  distractionCount: number;
}

export function usePomodoro() {
  const [records, setRecords] = useState<PomodoroRecord[]>([]);
  const [todayCount, setTodayCount] = useState(0);
  const [isLoading, setIsLoading] = useState(false);

  // 添加专注记录
  const addRecord = useCallback(async (duration: number) => {
    try {
      const id = await invoke<number>('add_pomodoro_record', { duration });
      return id;
    } catch (err) {
      console.error('添加专注记录失败:', err);
      throw err;
    }
  }, []);

  // 完成专注记录
  const completeRecord = useCallback(async (id: number, distractionCount: number) => {
    try {
      await invoke('complete_pomodoro_record', { id, distractionCount });
    } catch (err) {
      console.error('完成专注记录失败:', err);
      throw err;
    }
  }, []);

  // 获取专注历史
  const fetchHistory = useCallback(async (limit: number = 10) => {
    setIsLoading(true);
    try {
      const history = await invoke<PomodoroRecord[]>('get_pomodoro_history', { limit });
      setRecords(history);
    } catch (err) {
      console.error('获取专注历史失败:', err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  // 获取今日专注次数
  const fetchTodayCount = useCallback(async () => {
    try {
      const count = await invoke<number>('get_today_pomodoro_count');
      setTodayCount(count);
    } catch (err) {
      console.error('获取今日专注次数失败:', err);
    }
  }, []);

  return {
    records,
    todayCount,
    isLoading,
    addRecord,
    completeRecord,
    fetchHistory,
    fetchTodayCount,
  };
}
