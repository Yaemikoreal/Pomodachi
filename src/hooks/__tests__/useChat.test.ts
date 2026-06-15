import { describe, it, expect, beforeEach } from 'vitest';
import { renderHook, waitFor, act } from '@testing-library/react';
import { useChat } from '../useChat';
import { __setInvokeMock, __resetMocks } from '../../test-utils/mockTauri';

describe('useChat', () => {
  beforeEach(() => {
    __resetMocks();
    __setInvokeMock('get_chat_history', () => []);
  });

  it('初始状态为空消息列表', async () => {
    const { result } = renderHook(() => useChat());

    await waitFor(() => {
      expect(result.current.messages).toEqual([]);
    });
    expect(result.current.isLoading).toBe(false);
    expect(result.current.error).toBeNull();
  });

  it('加载聊天历史', async () => {
    const history = [
      { id: 1, role: 'user', content: '你好', createdAt: '2026-01-01T00:00:00Z' },
      { id: 2, role: 'assistant', content: '喵~', createdAt: '2026-01-01T00:00:01Z' },
    ];
    __setInvokeMock('get_chat_history', () => history);

    const { result } = renderHook(() => useChat());

    await waitFor(() => {
      expect(result.current.messages.length).toBe(2);
    });
    expect(result.current.messages[0].content).toBe('你好');
    expect(result.current.messages[1].content).toBe('喵~');
  });

  it('发送消息成功后显示用户消息和回复', async () => {
    __setInvokeMock('get_chat_history', () => []);
    __setInvokeMock('send_chat_message', () => '你好呀！今天也要加油哦！ 🐱');

    const { result } = renderHook(() => useChat());

    // 等待初始加载完成
    await waitFor(() => expect(result.current.isLoading).toBe(false));

    // 发送消息
    await act(async () => {
      await result.current.sendMessage('你好');
    });

    // 应包含用户消息和助手回复
    expect(result.current.messages.length).toBe(2);
    expect(result.current.messages[0].role).toBe('user');
    expect(result.current.messages[0].content).toBe('你好');
    expect(result.current.messages[1].role).toBe('assistant');
    expect(result.current.messages[1].content).toBe('你好呀！今天也要加油哦！ 🐱');
  });

  it('发送空消息不触发调用', async () => {
    const sendSpy = __setInvokeMock('send_chat_message', () => '');
    let callCount = 0;
    __setInvokeMock('send_chat_message', () => {
      callCount++;
      return '';
    });

    const { result } = renderHook(() => useChat());

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await act(async () => {
      await result.current.sendMessage('  ');
    });

    // 空消息不应发送
    expect(callCount).toBe(0);
  });

  it('发送失败时显示回退错误消息', async () => {
    __setInvokeMock('get_chat_history', () => []);
    __setInvokeMock('send_chat_message', () => { throw new Error('API 错误'); });

    const { result } = renderHook(() => useChat());

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await act(async () => {
      await result.current.sendMessage('测试');
    });

    // 应包含用户消息和回退消息
    expect(result.current.messages.length).toBe(2);
    expect(result.current.messages[0].role).toBe('user');
    expect(result.current.messages[1].role).toBe('assistant');
    expect(result.current.messages[1].content).toContain('不舒服');
    expect(result.current.error).toBeTruthy();
  });

  it('清空聊天历史', async () => {
    __setInvokeMock('get_chat_history', () => []);
    __setInvokeMock('clear_chat_history', () => {});

    const { result } = renderHook(() => useChat());

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await act(async () => {
      await result.current.clearHistory();
    });

    expect(result.current.messages).toEqual([]);
    expect(result.current.error).toBeNull();
  });
});
