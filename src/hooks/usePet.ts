import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export type PetMood = 'happy' | 'focused' | 'tired' | 'sleeping' | 'listening' | 'thinking';

export function usePet() {
  const [mood, setMood] = useState<PetMood>('happy');
  const petName = '番茄猫';
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // 获取宠物情绪
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

  // 初始化
  useEffect(() => {
    fetchMood();
  }, [fetchMood]);

  return {
    mood,
    petName,
    isLoading,
    error,
    refresh: fetchMood,
  };
}
