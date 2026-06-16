import { describe, it, expect, beforeEach } from 'vitest';
import { renderHook, waitFor, act } from '@testing-library/react';
import { useBlocklist } from '../useBlocklist';
import { __setInvokeMock, __resetMocks } from '../../test-utils/mockTauri';

const MOCK_BLOCKLIST = [
  { id: 1, processName: 'WeChat.exe', addedAt: '2026-06-16T08:00:00Z' },
  { id: 2, processName: 'QQ.exe', addedAt: '2026-06-16T09:00:00Z' },
];

describe('useBlocklist', () => {
  beforeEach(() => {
    __resetMocks();
    __setInvokeMock('get_blocklist', () => MOCK_BLOCKLIST);
  });

  it('初始状态正确', () => {
    const { result } = renderHook(() => useBlocklist());

    expect(result.current.blocklist).toEqual([]);
    expect(result.current.isLoading).toBe(false);
  });

  it('fetchBlocklist 加载黑名单列表', async () => {
    const { result } = renderHook(() => useBlocklist());

    await act(async () => {
      await result.current.fetchBlocklist();
    });

    await waitFor(() => {
      expect(result.current.blocklist).toEqual(MOCK_BLOCKLIST);
    });
    expect(result.current.isLoading).toBe(false);
  });

  it('fetchBlocklist 失败时保持空列表', async () => {
    __setInvokeMock('get_blocklist', () => {
      throw new Error('获取失败');
    });

    const { result } = renderHook(() => useBlocklist());

    await act(async () => {
      await result.current.fetchBlocklist();
    });

    expect(result.current.blocklist).toEqual([]);
    expect(result.current.isLoading).toBe(false);
  });

  it('addToBlocklist 添加应用到黑名单', async () => {
    __setInvokeMock('add_to_blocklist', () => undefined);
    const updatedList = [
      ...MOCK_BLOCKLIST,
      { id: 3, processName: 'Douyin.exe', addedAt: '2026-06-16T10:00:00Z' },
    ];
    __setInvokeMock('get_blocklist', () => updatedList);

    const { result } = renderHook(() => useBlocklist());

    await act(async () => {
      await result.current.addToBlocklist('Douyin.exe');
    });

    await waitFor(() => {
      expect(result.current.blocklist).toEqual(updatedList);
    });
  });

  it('addToBlocklist 失败时抛出错误', async () => {
    __setInvokeMock('add_to_blocklist', () => {
      throw new Error('添加失败');
    });

    const { result } = renderHook(() => useBlocklist());

    await expect(async () => {
      await act(async () => {
        await result.current.addToBlocklist('Douyin.exe');
      });
    }).rejects.toThrow('添加失败');
  });

  it('removeFromBlocklist 从黑名单移除应用', async () => {
    __setInvokeMock('remove_from_blocklist', () => undefined);
    const updatedList = MOCK_BLOCKLIST.filter((item) => item.processName !== 'WeChat.exe');
    __setInvokeMock('get_blocklist', () => updatedList);

    const { result } = renderHook(() => useBlocklist());

    await act(async () => {
      await result.current.removeFromBlocklist('WeChat.exe');
    });

    await waitFor(() => {
      expect(result.current.blocklist).toEqual(updatedList);
    });
  });

  it('removeFromBlocklist 失败时抛出错误', async () => {
    __setInvokeMock('remove_from_blocklist', () => {
      throw new Error('移除失败');
    });

    const { result } = renderHook(() => useBlocklist());

    await expect(async () => {
      await act(async () => {
        await result.current.removeFromBlocklist('WeChat.exe');
      });
    }).rejects.toThrow('移除失败');
  });

  it('startMonitoring 启动监听', async () => {
    __setInvokeMock('start_monitoring', () => undefined);

    const { result } = renderHook(() => useBlocklist());

    await act(async () => {
      await result.current.startMonitoring();
    });

    // 验证没有抛出错误
    expect(true).toBe(true);
  });

  it('startMonitoring 失败时抛出错误', async () => {
    __setInvokeMock('start_monitoring', () => {
      throw new Error('启动失败');
    });

    const { result } = renderHook(() => useBlocklist());

    await expect(async () => {
      await act(async () => {
        await result.current.startMonitoring();
      });
    }).rejects.toThrow('启动失败');
  });

  it('stopMonitoring 停止监听', async () => {
    __setInvokeMock('stop_monitoring', () => undefined);

    const { result } = renderHook(() => useBlocklist());

    await act(async () => {
      await result.current.stopMonitoring();
    });

    // 验证没有抛出错误
    expect(true).toBe(true);
  });

  it('isMonitoring 返回监听状态', async () => {
    __setInvokeMock('is_monitoring', () => true);

    const { result } = renderHook(() => useBlocklist());

    let monitoring = false;
    await act(async () => {
      monitoring = await result.current.isMonitoring();
    });

    expect(monitoring).toBe(true);
  });

  it('isMonitoring 失败时返回 false', async () => {
    __setInvokeMock('is_monitoring', () => {
      throw new Error('查询失败');
    });

    const { result } = renderHook(() => useBlocklist());

    let monitoring = true;
    await act(async () => {
      monitoring = await result.current.isMonitoring();
    });

    expect(monitoring).toBe(false);
  });
});
