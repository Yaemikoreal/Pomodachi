import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

/** Claude Code 进程状态 */
export type ClaudeCodeState =
  | 'notRunning'  // 未运行
  | 'idle'        // 运行中 + CPU 空闲
  | 'active'      // 运行中 + CPU 活跃
  | 'longIdle'    // 运行中 + 长时空闲
  | 'crashed';    // 进程异常退出

/** Claude Code 状态详情 */
export interface ClaudeCodeStatus {
  state: ClaudeCodeState;
  cpuUsage: number;
  idleSeconds: number;
  processName: string;
}

export function useClaudeCodeStatus() {
  const [status, setStatus] = useState<ClaudeCodeStatus>({
    state: 'notRunning',
    cpuUsage: 0,
    idleSeconds: 0,
    processName: '',
  });
  const [isLoading, setIsLoading] = useState(true);

  // 初始拉取状态
  const fetchStatus = useCallback(async () => {
    try {
      const s = await invoke<ClaudeCodeStatus>('get_claude_code_status');
      setStatus(s);
    } catch (err) {
      console.error('获取 Claude Code 状态失败:', err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  // 初始化 + 监听状态变化事件
  useEffect(() => {
    fetchStatus();

    const unlisten = listen<ClaudeCodeStatus>('claude-code-status-changed', (event) => {
      setStatus(event.payload);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [fetchStatus]);

  return {
    status,
    isLoading,
    refresh: fetchStatus,
  };
}
