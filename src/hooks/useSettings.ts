import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

/** 应用设置默认值 */
const DEFAULT_SETTINGS: Record<string, string> = {
  language: 'zh',
  launchOnStartup: 'false',
  windowOpacity: '1.0',
  focusDuration: '25',
  shortBreakDuration: '5',
  longBreakDuration: '15',
  monitorEnabled: 'true',
  monitorCooldown: '30',
  maxTurns: '3',
  maxBudgetUsd: '0.1',
};

export interface AppSettings {
  language: 'zh' | 'en';
  launchOnStartup: boolean;
  windowOpacity: number;
  focusDuration: number;
  shortBreakDuration: number;
  longBreakDuration: number;
  monitorEnabled: boolean;
  monitorCooldown: number;
  maxTurns: number;
  maxBudgetUsd: number | null;
}

function parseSettings(raw: Record<string, string>): AppSettings {
  const s = { ...DEFAULT_SETTINGS, ...raw };
  return {
    language: (s.language === 'en' ? 'en' : 'zh') as 'zh' | 'en',
    launchOnStartup: s.launchOnStartup === 'true',
    windowOpacity: Math.min(1, Math.max(0.3, parseFloat(s.windowOpacity) || 1)),
    focusDuration: Math.max(1, parseInt(s.focusDuration) || 25),
    shortBreakDuration: Math.max(1, parseInt(s.shortBreakDuration) || 5),
    longBreakDuration: Math.max(1, parseInt(s.longBreakDuration) || 15),
    monitorEnabled: s.monitorEnabled !== 'false',
    monitorCooldown: Math.max(5, parseInt(s.monitorCooldown) || 30),
    maxTurns: Math.max(1, Math.min(10, parseInt(s.maxTurns) || 3)),
    maxBudgetUsd: s.maxBudgetUsd ? Math.max(0, parseFloat(s.maxBudgetUsd)) : null,
  };
}

function serializeSettings(settings: Partial<AppSettings>): Record<string, string> {
  const result: Record<string, string> = {};
  for (const [key, value] of Object.entries(settings)) {
    if (value !== undefined && value !== null) {
      result[key] = String(value);
    }
  }
  return result;
}

export function useSettings() {
  const [settings, setSettings] = useState<AppSettings>(parseSettings({}));
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  /** 加载所有设置 */
  const loadSettings = useCallback(async () => {
    setIsLoading(true);
    try {
      const rawList = await invoke<{ key: string; value: string }[]>('get_all_settings');
      const rawMap: Record<string, string> = {};
      for (const item of rawList) {
        rawMap[item.key] = item.value;
      }
      setSettings(parseSettings(rawMap));
      setError(null);
    } catch (err) {
      console.error('加载设置失败:', err);
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  }, []);

  /** 保存部分设置 */
  const saveSettings = useCallback(async (partial: Partial<AppSettings>) => {
    setIsSaving(true);
    try {
      const serialized = serializeSettings(partial);
      for (const [key, value] of Object.entries(serialized)) {
        await invoke('set_setting', { key, value });
      }
      // 加载最新状态
      await loadSettings();
      setError(null);
    } catch (err) {
      console.error('保存设置失败:', err);
      setError(String(err));
    } finally {
      setIsSaving(false);
    }
  }, [loadSettings]);

  return {
    settings,
    isLoading,
    isSaving,
    error,
    loadSettings,
    saveSettings,
  };
}
