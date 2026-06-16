import { describe, it, expect, beforeEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { usePet } from '../usePet';
import { __setInvokeMock, __emitEvent, __resetMocks } from '../../test-utils/mockTauri';

describe('usePet', () => {
  beforeEach(() => {
    __resetMocks();
  });

  it('加载初始情绪状态', async () => {
    __setInvokeMock('get_pet_mood', () => 'happy');

    const { result } = renderHook(() => usePet());

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(result.current.mood).toBe('happy');
    expect(result.current.petName).toBe('番茄猫');
    expect(result.current.error).toBeNull();
  });

  it('名称始终为「番茄猫」', async () => {
    __setInvokeMock('get_pet_mood', () => 'focused');

    const { result } = renderHook(() => usePet());

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(result.current.petName).toBe('番茄猫');
  });

  it('情绪值映射正确', async () => {
    const moods = ['happy', 'focused', 'tired', 'sleeping', 'listening', 'thinking'];

    for (const mood of moods) {
      __setInvokeMock('get_pet_mood', () => mood);

      const { result } = renderHook(() => usePet());
      await waitFor(() => expect(result.current.isLoading).toBe(false));
      expect(result.current.mood).toBe(mood);
    }
  });

  it('invoke 失败时设置 error 状态', async () => {
    __setInvokeMock('get_pet_mood', () => { throw new Error('Rust 错误'); });

    const { result } = renderHook(() => usePet());

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(result.current.error).toBeTruthy();
    // 默认情绪为 happy（fallback）
    expect(result.current.mood).toBe('happy');
  });

  it('refresh 重新加载情绪', async () => {
    __setInvokeMock('get_pet_mood', () => 'happy');

    const { result } = renderHook(() => usePet());

    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(result.current.mood).toBe('happy');

    // 修改 mock 返回值并刷新
    __setInvokeMock('get_pet_mood', () => 'tired');
    result.current.refresh();

    await waitFor(() => {
      expect(result.current.mood).toBe('tired');
    });
  });

  it('监听 pet-mood-changed 事件实时更新情绪', async () => {
    __setInvokeMock('get_pet_mood', () => 'happy');

    const { result } = renderHook(() => usePet());

    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(result.current.mood).toBe('happy');

    // 模拟后端 emit pet-mood-changed 事件 → Thinking
    __emitEvent('pet-mood-changed', 'thinking');

    await waitFor(() => {
      expect(result.current.mood).toBe('thinking');
    });

    // 模拟后端 emit pet-mood-changed 事件 → Happy（AI 回复完成）
    __emitEvent('pet-mood-changed', 'happy');

    await waitFor(() => {
      expect(result.current.mood).toBe('happy');
    });
  });

  it('监听 pet-mood-changed 事件：AI 失败变为 tired', async () => {
    __setInvokeMock('get_pet_mood', () => 'happy');

    const { result } = renderHook(() => usePet());

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    // 模拟 AI 失败 → Tired
    __emitEvent('pet-mood-changed', 'tired');

    await waitFor(() => {
      expect(result.current.mood).toBe('tired');
    });
  });
});
