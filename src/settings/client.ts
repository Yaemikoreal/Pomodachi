import { invoke } from '@tauri-apps/api/core';
import { isTauri } from '../utils/tauri';

export interface Settings {
  provider: string;
  api_base: string;
  api_key?: string;
  model: string;
  system_prompt: string;
  scale_factor: number;
  avatar_pack_path: string;
  render_fix: boolean;
  auto_start: boolean;
  always_on_top: boolean;
  click_through: boolean;
  shortcut_show_hide: string;
}

export const DEFAULT_SETTINGS: Settings = {
  provider: 'openai',
  api_base: 'https://api.openai.com/v1',
  model: 'gpt-4o-mini',
  system_prompt: '',
  scale_factor: 1.0,
  avatar_pack_path: '',
  render_fix: false,
  auto_start: false,
  always_on_top: true,
  click_through: true,
  shortcut_show_hide: '',
};

export async function getSettings(): Promise<Settings> {
  if (!isTauri) {
    return { ...DEFAULT_SETTINGS };
  }
  const saved = await invoke<Settings>('get_settings');
  return { ...DEFAULT_SETTINGS, ...saved };
}

export async function saveSettings(settings: Settings): Promise<void> {
  if (!isTauri) return;
  return await invoke('save_settings', { settings });
}
