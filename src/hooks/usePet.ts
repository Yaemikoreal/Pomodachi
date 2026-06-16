import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

/** 宠物情绪状态，参考 agent-pet 的 messageMap 设计 */
export type PetMood =
  | 'happy'      // 开心 - success/idle
  | 'focused'    // 专注 - processing
  | 'tired'      // 疲惫 - waiting
  | 'sleeping'   // 睡眠 - idle
  | 'listening'  // 聆听 - new_message
  | 'thinking'   // 思考 - review_required
  | 'error'      // 错误/分心 - error
  | 'waving';    // 挥手打招呼 - new_message

export function usePet() {
  const [mood, setMood] = useState<PetMood>('happy');
  const petName = '番茄猫';
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // 获取宠物情绪（初始拉取）
  const fetchMood = useCallback(async () => {
    try {
      const currentMood = await invoke<string>('get_pet_mood');
      setMood(currentMood as PetMood);
      setError(null);
    } catch (err) {
      console.error('获取宠物情绪失败:', err);
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  }, []);

  // 初始化 + 监听后端情绪变化事件
  useEffect(() => {
    fetchMood();

    // 监听 Rust 后端 emit 的 pet-mood-changed 事件
    const unlisten = listen<string>('pet-mood-changed', (event) => {
      setMood(event.payload as PetMood);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [fetchMood]);

  return {
    mood,
    petName,
    isLoading,
    error,
    refresh: fetchMood,
  };
}
