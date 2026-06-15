import { describe, it, expect, beforeEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { useTimer } from '../useTimer';
import { __setInvokeMock, __emitEvent, __resetMocks } from '../../test-utils/mockTauri';

const MOCK_INITIAL_STATE = {
  mode: 'focus' as const,
  duration: 25 * 60,
  remaining: 25 * 60,
  isRunning: false,
  completedPomodoros: 0,
};

const MOCK_RUNNING_STATE = {
  mode: 'focus' as const,
  duration: 25 * 60,
  remaining: 20 * 60,
  isRunning: true,
  completedPomodoros: 0,
};

describe('useTimer', () => {
  beforeEach(() => {
    __resetMocks();
    __setInvokeMock('get_timer_state', () => MOCK_INITIAL_STATE);
  });

  it('加载初始计时器状态', async () => {
    const { result } = renderHook(() => useTimer());

    await waitFor(() => {
      expect(result.current.isRunning).toBe(false);
    });

    expect(result.current.mode).toBe('focus');
    expect(result.current.remaining).toBe(25 * 60);
    expect(result.current.duration).toBe(25 * 60);
    expect(result.current.formattedTime).toBe('25:00');
    expect(result.current.progress).toBe(0);
  });

  it('start 调用 invoke start_timer 并更新状态', async () => {
    __setInvokeMock('start_timer', () => MOCK_RUNNING_STATE);

    const { result } = renderHook(() => useTimer());

    await waitFor(() => expect(result.current.isRunning).toBe(false));
    result.current.start();

    await waitFor(() => {
      expect(result.current.isRunning).toBe(true);
    });
    expect(result.current.remaining).toBe(20 * 60);
    expect(result.current.formattedTime).toBe('20:00');
  });

  it('pause 调用 invoke pause_timer 并更新状态', async () => {
    __setInvokeMock('start_timer', () => MOCK_RUNNING_STATE);
    __setInvokeMock('pause_timer', () => ({
      ...MOCK_RUNNING_STATE,
      isRunning: false,
    }));

    const { result } = renderHook(() => useTimer());

    await waitFor(() => expect(result.current.isRunning).toBe(false));
    result.current.start();
    await waitFor(() => expect(result.current.isRunning).toBe(true));
    result.current.pause();

    await waitFor(() => {
      expect(result.current.isRunning).toBe(false);
    });
  });

  it('reset 调用 invoke reset_timer 并重置状态', async () => {
    __setInvokeMock('reset_timer', () => MOCK_INITIAL_STATE);

    const { result } = renderHook(() => useTimer());

    await waitFor(() => expect(result.current.isRunning).toBe(false));
    await result.current.reset();

    expect(result.current.isRunning).toBe(false);
    expect(result.current.remaining).toBe(25 * 60);
  });

  it('skip 调用 invoke skip_timer 并切换到下一阶段', async () => {
    const skippedState = {
      mode: 'shortBreak' as const,
      duration: 5 * 60,
      remaining: 5 * 60,
      isRunning: false,
      completedPomodoros: 0,
    };
    __setInvokeMock('skip_timer', () => skippedState);

    const { result } = renderHook(() => useTimer());

    await waitFor(() => expect(result.current.isRunning).toBe(false));
    result.current.skip();

    await waitFor(() => {
      expect(result.current.mode).toBe('shortBreak');
    });
    expect(result.current.duration).toBe(5 * 60);
  });

  it('timer-tick 事件更新状态', async () => {
    const { result } = renderHook(() => useTimer());

    await waitFor(() => expect(result.current.isRunning).toBe(false));

    // 模拟 tick 事件
    __emitEvent('timer-tick', {
      ...MOCK_RUNNING_STATE,
      remaining: 19 * 60 + 30,
    });

    await waitFor(() => {
      expect(result.current.remaining).toBe(19 * 60 + 30);
    });
    expect(result.current.formattedTime).toBe('19:30');
  });

  it('timer-complete 事件更新状态', async () => {
    const { result } = renderHook(() => useTimer());

    await waitFor(() => expect(result.current.isRunning).toBe(false));

    // 模拟完成事件
    __emitEvent('timer-complete', {
      ...MOCK_INITIAL_STATE,
      completedPomodoros: 1,
    });

    await waitFor(() => {
      expect(result.current.completedPomodoros).toBe(1);
    });
  });

  it('格式化时间显示', () => {
    const { result } = renderHook(() => useTimer());
    expect(result.current.formatTime(0)).toBe('00:00');
    expect(result.current.formatTime(61)).toBe('01:01');
    expect(result.current.formatTime(150)).toBe('02:30');
    expect(result.current.formatTime(3600)).toBe('60:00');
  });
});
