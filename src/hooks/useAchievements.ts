import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export interface Achievement {
  id: number;
  key: string;
  name: string;
  description: string;
  unlockedAt: string | null;
  icon: string;
}

export interface FocusStats {
  totalPomodoros: number;
  totalFocusSeconds: number;
  currentStreak: number;
  longestStreak: number;
  lastFocusDate: string | null;
}

export function useAchievements() {
  const [achievements, setAchievements] = useState<Achievement[]>([]);
  const [focusStats, setFocusStats] = useState<FocusStats | null>(null);
  const [newAchievement, setNewAchievement] = useState<Achievement | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // 加载成就和统计
  const loadData = useCallback(async () => {
    setIsLoading(true);
    try {
      const [achList, stats] = await Promise.all([
        invoke<Achievement[]>('get_achievements'),
        invoke<FocusStats>('get_focus_stats'),
      ]);
      setAchievements(achList);
      setFocusStats(stats);
      setError(null);
    } catch (err) {
      console.error('加载成就数据失败:', err);
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  }, []);

  // 初始化加载
  useEffect(() => {
    loadData();
  }, [loadData]);

  // 监听成就解锁事件
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const setup = async () => {
      unlisten = await listen<Achievement>('achievement-unlocked', (event) => {
        const ach = event.payload;
        setNewAchievement(ach);
        // 更新本地成就列表状态
        setAchievements((prev) =>
          prev.map((a) => (a.key === ach.key ? { ...ach, unlockedAt: ach.unlockedAt } : a))
        );
        // 5 秒后清除提示
        setTimeout(() => setNewAchievement(null), 5000);
      });
    };

    setup();
    return () => {
      unlisten?.();
    };
  }, []);

  // 格式化专注时间
  const formatFocusTime = useCallback((seconds: number): string => {
    const hours = Math.floor(seconds / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    if (hours > 0) {
      return `${hours} 小时 ${mins} 分钟`;
    }
    return `${mins} 分钟`;
  }, []);

  const unlockedCount = achievements.filter((a) => a.unlockedAt).length;
  const totalCount = achievements.length;

  return {
    achievements,
    focusStats,
    newAchievement,
    isLoading,
    error,
    loadData,
    unlockedCount,
    totalCount,
    formatFocusTime,
    clearNewAchievement: () => setNewAchievement(null),
  };
}
