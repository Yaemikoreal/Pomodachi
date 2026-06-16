import { describe, it, expect, beforeEach, vi } from 'vitest';
import { renderHook, waitFor, act } from '@testing-library/react';
import { useAchievements } from '../useAchievements';
import { __setInvokeMock, __emitEvent, __resetMocks } from '../../test-utils/mockTauri';

const MOCK_ACHIEVEMENTS = [
  {
    id: 1,
    key: 'first_pomodoro',
    name: '初次专注',
    description: '完成第一个番茄钟',
    unlockedAt: '2026-06-16T08:00:00Z',
    icon: '🍅',
  },
  {
    id: 2,
    key: 'five_pomodoros',
    name: '专注新手',
    description: '完成 5 个番茄钟',
    unlockedAt: null,
    icon: '⭐',
  },
  {
    id: 3,
    key: 'streak_3',
    name: '三天连续',
    description: '连续 3 天完成番茄钟',
    unlockedAt: null,
    icon: '🔥',
  },
];

const MOCK_FOCUS_STATS = {
  totalPomodoros: 10,
  totalFocusSeconds: 15000,
  currentStreak: 2,
  longestStreak: 5,
  lastFocusDate: '2026-06-16',
};

describe('useAchievements', () => {
  beforeEach(() => {
    __resetMocks();
    __setInvokeMock('get_achievements', () => MOCK_ACHIEVEMENTS);
    __setInvokeMock('get_focus_stats', () => MOCK_FOCUS_STATS);
  });

  it('初始加载数据', async () => {
    const { result } = renderHook(() => useAchievements());

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.achievements).toEqual(MOCK_ACHIEVEMENTS);
    expect(result.current.focusStats).toEqual(MOCK_FOCUS_STATS);
    expect(result.current.error).toBeNull();
  });

  it('unlockedCount 计算正确', async () => {
    const { result } = renderHook(() => useAchievements());

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.unlockedCount).toBe(1); // 只有第一个成就已解锁
    expect(result.current.totalCount).toBe(3);
  });

  it('formatFocusTime 格式化分钟', async () => {
    const { result } = renderHook(() => useAchievements());

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.formatFocusTime(300)).toBe('5 分钟');
    expect(result.current.formatFocusTime(1500)).toBe('25 分钟');
  });

  it('formatFocusTime 格式化小时', async () => {
    const { result } = renderHook(() => useAchievements());

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.formatFocusTime(3600)).toBe('1 小时 0 分钟');
    expect(result.current.formatFocusTime(5400)).toBe('1 小时 30 分钟');
  });

  it('loadData 失败时设置 error', async () => {
    __setInvokeMock('get_achievements', () => {
      throw new Error('数据库错误');
    });

    const { result } = renderHook(() => useAchievements());

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.error).toBeTruthy();
  });

  it('achievement-unlocked 事件更新成就列表', async () => {
    const { result } = renderHook(() => useAchievements());

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    // 模拟成就解锁事件
    const newAch = {
      id: 2,
      key: 'five_pomodoros',
      name: '专注新手',
      description: '完成 5 个番茄钟',
      unlockedAt: '2026-06-16T10:00:00Z',
      icon: '⭐',
    };

    act(() => {
      __emitEvent('achievement-unlocked', newAch);
    });

    await waitFor(() => {
      expect(result.current.newAchievement).toEqual(newAch);
    });

    // 验证成就列表已更新
    const updatedAch = result.current.achievements.find((a) => a.key === 'five_pomodoros');
    expect(updatedAch?.unlockedAt).toBe('2026-06-16T10:00:00Z');
  });

  it('achievement-unlocked 事件 5 秒后清除提示', async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });

    const { result } = renderHook(() => useAchievements());

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    const newAch = {
      id: 2,
      key: 'five_pomodoros',
      name: '专注新手',
      description: '完成 5 个番茄钟',
      unlockedAt: '2026-06-16T10:00:00Z',
      icon: '⭐',
    };

    act(() => {
      __emitEvent('achievement-unlocked', newAch);
    });

    expect(result.current.newAchievement).toEqual(newAch);

    // 快进 5 秒
    act(() => {
      vi.advanceTimersByTime(5000);
    });

    expect(result.current.newAchievement).toBeNull();

    vi.useRealTimers();
  });

  it('clearNewAchievement 手动清除新成就提示', async () => {
    const { result } = renderHook(() => useAchievements());

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    const newAch = {
      id: 2,
      key: 'five_pomodoros',
      name: '专注新手',
      description: '完成 5 个番茄钟',
      unlockedAt: '2026-06-16T10:00:00Z',
      icon: '⭐',
    };

    // 发送事件
    act(() => {
      __emitEvent('achievement-unlocked', newAch);
    });

    // 验证事件已接收
    await waitFor(() => {
      expect(result.current.newAchievement).toEqual(newAch);
    });

    // 手动清除
    act(() => {
      result.current.clearNewAchievement();
    });

    expect(result.current.newAchievement).toBeNull();
  });

  it('loadData 重新加载数据', async () => {
    const { result } = renderHook(() => useAchievements());

    // 等待初始加载完成
    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.focusStats?.totalPomodoros).toBe(10);

    // 修改 mock 数据
    const updatedStats = {
      ...MOCK_FOCUS_STATS,
      totalPomodoros: 15,
    };
    __setInvokeMock('get_focus_stats', () => updatedStats);

    // 重新加载
    await act(async () => {
      await result.current.loadData();
    });

    // 验证数据已更新
    await waitFor(() => {
      expect(result.current.focusStats?.totalPomodoros).toBe(15);
    });
  });
});
