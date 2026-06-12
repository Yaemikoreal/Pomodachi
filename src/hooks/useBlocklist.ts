import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface BlocklistItem {
  id: number;
  processName: string;
  addedAt: string;
}

export function useBlocklist() {
  const [blocklist, setBlocklist] = useState<BlocklistItem[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  // 获取黑名单
  const fetchBlocklist = useCallback(async () => {
    setIsLoading(true);
    try {
      const list = await invoke<BlocklistItem[]>('get_blocklist');
      setBlocklist(list);
    } catch (err) {
      console.error('获取黑名单失败:', err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  // 添加到黑名单
  const addToBlocklist = useCallback(async (processName: string) => {
    try {
      await invoke('add_to_blocklist', { processName });
      // 刷新列表
      await fetchBlocklist();
    } catch (err) {
      console.error('添加到黑名单失败:', err);
      throw err;
    }
  }, [fetchBlocklist]);

  // 从黑名单移除
  const removeFromBlocklist = useCallback(async (processName: string) => {
    try {
      await invoke('remove_from_blocklist', { processName });
      // 刷新列表
      await fetchBlocklist();
    } catch (err) {
      console.error('从黑名单移除失败:', err);
      throw err;
    }
  }, [fetchBlocklist]);

  // 开始监听
  const startMonitoring = useCallback(async () => {
    try {
      await invoke('start_monitoring');
    } catch (err) {
      console.error('启动监听失败:', err);
      throw err;
    }
  }, []);

  // 停止监听
  const stopMonitoring = useCallback(async () => {
    try {
      await invoke('stop_monitoring');
    } catch (err) {
      console.error('停止监听失败:', err);
      throw err;
    }
  }, []);

  // 检查监听状态
  const isMonitoring = useCallback(async () => {
    try {
      return await invoke<boolean>('is_monitoring');
    } catch (err) {
      console.error('检查监听状态失败:', err);
      return false;
    }
  }, []);

  return {
    blocklist,
    isLoading,
    fetchBlocklist,
    addToBlocklist,
    removeFromBlocklist,
    startMonitoring,
    stopMonitoring,
    isMonitoring,
  };
}
