import { describe, it, expect, beforeEach } from 'vitest';
import { renderHook, waitFor, act } from '@testing-library/react';
import { useSettings } from '../useSettings';
import { __setInvokeMock, __resetMocks } from '../../test-utils/mockTauri';

const MOCK_SETTINGS = [
  { key: 'language', value: 'zh' },
  { key: 'launchOnStartup', value: 'false' },
  { key: 'windowOpacity', value: '0.8' },
  { key: 'focusDuration', value: '30' },
  { key: 'shortBreakDuration', value: '5' },
  { key: 'longBreakDuration', value: '15' },
  { key: 'monitorEnabled', value: 'true' },
  { key: 'monitorCooldown', value: '30' },
  { key: 'maxTurns', value: '5' },
  { key: 'maxBudgetUsd', value: '0.2' },
];

const EXPECTED_PARSED = {
  language: 'zh',
  launchOnStartup: false,
  windowOpacity: 0.8,
  focusDuration: 30,
  shortBreakDuration: 5,
  longBreakDuration: 15,
  monitorEnabled: true,
  monitorCooldown: 30,
  maxTurns: 5,
  maxBudgetUsd: 0.2,
};

describe('useSettings', () => {
  beforeEach(() => {
    __resetMocks();
    __setInvokeMock('get_all_settings', () => MOCK_SETTINGS);
  });

  it('初始状态使用默认设置', () => {
    const { result } = renderHook(() => useSettings());

    expect(result.current.settings.language).toBe('zh');
    expect(result.current.settings.launchOnStartup).toBe(false);
    expect(result.current.settings.focusDuration).toBe(25);
    expect(result.current.isLoading).toBe(true);
  });

  it('loadSettings 加载并解析设置', async () => {
    const { result } = renderHook(() => useSettings());

    await act(async () => {
      await result.current.loadSettings();
    });

    await waitFor(() => {
      expect(result.current.settings).toEqual(EXPECTED_PARSED);
    });
    expect(result.current.isLoading).toBe(false);
    expect(result.current.error).toBeNull();
  });

  it('loadSettings 失败时设置 error', async () => {
    __setInvokeMock('get_all_settings', () => {
      throw new Error('数据库错误');
    });

    const { result } = renderHook(() => useSettings());

    await act(async () => {
      await result.current.loadSettings();
    });

    expect(result.current.error).toBeTruthy();
    expect(result.current.isLoading).toBe(false);
  });

  it('parseSettings 处理边界值 - windowOpacity', async () => {
    __setInvokeMock('get_all_settings', () => [
      { key: 'windowOpacity', value: '0.1' }, // 低于最小值 0.3
    ]);

    const { result } = renderHook(() => useSettings());

    await act(async () => {
      await result.current.loadSettings();
    });

    await waitFor(() => {
      expect(result.current.settings.windowOpacity).toBe(0.3);
    });
  });

  it('parseSettings 处理边界值 - focusDuration', async () => {
    // parseInt('0') 返回 0，0 || 25 返回 25（因为 0 是 falsy）
    // 所以 focusDuration='0' 实际会使用默认值 25
    __setInvokeMock('get_all_settings', () => [
      { key: 'focusDuration', value: '0' },
    ]);

    const { result } = renderHook(() => useSettings());

    await act(async () => {
      await result.current.loadSettings();
    });

    await waitFor(() => {
      expect(result.current.settings.focusDuration).toBe(25); // 0 是 falsy，使用默认值
    });
  });

  it('parseSettings 处理边界值 - focusDuration 负数', async () => {
    // 负数会被 Math.max(1, ...) 限制为 1
    __setInvokeMock('get_all_settings', () => [
      { key: 'focusDuration', value: '-5' },
    ]);

    const { result } = renderHook(() => useSettings());

    await act(async () => {
      await result.current.loadSettings();
    });

    await waitFor(() => {
      expect(result.current.settings.focusDuration).toBe(1);
    });
  });

  it('parseSettings 处理边界值 - maxTurns', async () => {
    __setInvokeMock('get_all_settings', () => [
      { key: 'maxTurns', value: '15' }, // 超过最大值 10
    ]);

    const { result } = renderHook(() => useSettings());

    await act(async () => {
      await result.current.loadSettings();
    });

    await waitFor(() => {
      expect(result.current.settings.maxTurns).toBe(10);
    });
  });

  it('parseSettings 处理无效值', async () => {
    __setInvokeMock('get_all_settings', () => [
      { key: 'focusDuration', value: 'abc' }, // 无效数字
    ]);

    const { result } = renderHook(() => useSettings());

    await act(async () => {
      await result.current.loadSettings();
    });

    await waitFor(() => {
      expect(result.current.settings.focusDuration).toBe(25); // 使用默认值
    });
  });

  it('saveSettings 保存设置并重新加载', async () => {
    __setInvokeMock('set_setting', () => undefined);
    const updatedSettings = [
      ...MOCK_SETTINGS,
      { key: 'focusDuration', value: '45' },
    ];
    __setInvokeMock('get_all_settings', () => updatedSettings);

    const { result } = renderHook(() => useSettings());

    // 先加载初始设置
    await act(async () => {
      await result.current.loadSettings();
    });

    // 保存新设置
    await act(async () => {
      await result.current.saveSettings({ focusDuration: 45 });
    });

    await waitFor(() => {
      expect(result.current.settings.focusDuration).toBe(45);
    });
    expect(result.current.isSaving).toBe(false);
    expect(result.current.error).toBeNull();
  });

  it('saveSettings 失败时设置 error', async () => {
    __setInvokeMock('set_setting', () => {
      throw new Error('保存失败');
    });

    const { result } = renderHook(() => useSettings());

    await act(async () => {
      await result.current.saveSettings({ focusDuration: 45 });
    });

    expect(result.current.error).toBeTruthy();
    expect(result.current.isSaving).toBe(false);
  });

  it('parseSettings 处理 language 为 en', async () => {
    __setInvokeMock('get_all_settings', () => [
      { key: 'language', value: 'en' },
    ]);

    const { result } = renderHook(() => useSettings());

    await act(async () => {
      await result.current.loadSettings();
    });

    await waitFor(() => {
      expect(result.current.settings.language).toBe('en');
    });
  });

  it('parseSettings 处理 launchOnStartup 为 true', async () => {
    __setInvokeMock('get_all_settings', () => [
      { key: 'launchOnStartup', value: 'true' },
    ]);

    const { result } = renderHook(() => useSettings());

    await act(async () => {
      await result.current.loadSettings();
    });

    await waitFor(() => {
      expect(result.current.settings.launchOnStartup).toBe(true);
    });
  });

  it('parseSettings 处理 monitorEnabled 为 false', async () => {
    __setInvokeMock('get_all_settings', () => [
      { key: 'monitorEnabled', value: 'false' },
    ]);

    const { result } = renderHook(() => useSettings());

    await act(async () => {
      await result.current.loadSettings();
    });

    await waitFor(() => {
      expect(result.current.settings.monitorEnabled).toBe(false);
    });
  });

  it('parseSettings 处理 maxBudgetUsd 为 null', async () => {
    __setInvokeMock('get_all_settings', () => [
      { key: 'maxBudgetUsd', value: '' },
    ]);

    const { result } = renderHook(() => useSettings());

    await act(async () => {
      await result.current.loadSettings();
    });

    await waitFor(() => {
      expect(result.current.settings.maxBudgetUsd).toBeNull();
    });
  });
});
