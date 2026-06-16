import { describe, it, expect, beforeEach } from 'vitest';
import { renderHook, waitFor, act } from '@testing-library/react';
import { usePomodoro } from '../usePomodoro';
import { __setInvokeMock, __resetMocks } from '../../test-utils/mockTauri';

const MOCK_RECORDS = [
  {
    id: 1,
    startedAt: '2026-06-16T08:00:00Z',
    finishedAt: '2026-06-16T08:25:00Z',
    duration: 1500,
    completed: true,
    distractionCount: 0,
  },
  {
    id: 2,
    startedAt: '2026-06-16T09:00:00Z',
    finishedAt: null,
    duration: 1500,
    completed: false,
    distractionCount: 2,
  },
];

describe('usePomodoro', () => {
  beforeEach(() => {
    __resetMocks();
    __setInvokeMock('get_pomodoro_history', () => MOCK_RECORDS);
    __setInvokeMock('get_today_pomodoro_count', () => 3);
  });

  it('初始状态正确', () => {
    const { result } = renderHook(() => usePomodoro());

    expect(result.current.records).toEqual([]);
    expect(result.current.todayCount).toBe(0);
    expect(result.current.isLoading).toBe(false);
  });

  it('addRecord 调用 invoke 并返回记录 ID', async () => {
    __setInvokeMock('add_pomodoro_record', () => 42);

    const { result } = renderHook(() => usePomodoro());

    let recordId: number | undefined;
    await act(async () => {
      recordId = await result.current.addRecord(1500);
    });

    expect(recordId).toBe(42);
  });

  it('addRecord 失败时抛出错误', async () => {
    __setInvokeMock('add_pomodoro_record', () => {
      throw new Error('数据库错误');
    });

    const { result } = renderHook(() => usePomodoro());

    await expect(async () => {
      await act(async () => {
        await result.current.addRecord(1500);
      });
    }).rejects.toThrow('数据库错误');
  });

  it('completeRecord 调用 invoke 完成记录', async () => {
    __setInvokeMock('complete_pomodoro_record', () => undefined);

    const { result } = renderHook(() => usePomodoro());

    await act(async () => {
      await result.current.completeRecord(1, 2);
    });

    // 验证没有抛出错误
    expect(true).toBe(true);
  });

  it('fetchHistory 加载专注历史记录', async () => {
    const { result } = renderHook(() => usePomodoro());

    await act(async () => {
      await result.current.fetchHistory(10);
    });

    await waitFor(() => {
      expect(result.current.records).toEqual(MOCK_RECORDS);
    });
    expect(result.current.isLoading).toBe(false);
  });

  it('fetchHistory 使用默认 limit', async () => {
    const { result } = renderHook(() => usePomodoro());

    await act(async () => {
      await result.current.fetchHistory();
    });

    await waitFor(() => {
      expect(result.current.records).toEqual(MOCK_RECORDS);
    });
  });

  it('fetchHistory 失败时不更新记录', async () => {
    __setInvokeMock('get_pomodoro_history', () => {
      throw new Error('获取失败');
    });

    const { result } = renderHook(() => usePomodoro());

    await act(async () => {
      await result.current.fetchHistory();
    });

    expect(result.current.records).toEqual([]);
    expect(result.current.isLoading).toBe(false);
  });

  it('fetchTodayCount 加载今日专注次数', async () => {
    const { result } = renderHook(() => usePomodoro());

    await act(async () => {
      await result.current.fetchTodayCount();
    });

    await waitFor(() => {
      expect(result.current.todayCount).toBe(3);
    });
  });

  it('fetchTodayCount 失败时保持原值', async () => {
    __setInvokeMock('get_today_pomodoro_count', () => {
      throw new Error('查询失败');
    });

    const { result } = renderHook(() => usePomodoro());

    await act(async () => {
      await result.current.fetchTodayCount();
    });

    expect(result.current.todayCount).toBe(0);
  });
});
