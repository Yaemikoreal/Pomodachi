import { invoke } from '@tauri-apps/api/core';
import { isTauri } from '../utils/tauri';

export interface LlmResponse {
  content: string;
  emotion_tag?: string;
  intent?: {
    intent: 'create_task' | 'chat';
    title?: string;
    due_at?: number | null;
    emotion_hint?: string;
  };
}

export async function sendMessage(text: string): Promise<LlmResponse> {
  if (!isTauri) {
    return { content: '请在 Tauri 桌面环境中使用 LLM 功能。' };
  }
  return await invoke<LlmResponse>('send_message', { text });
}
