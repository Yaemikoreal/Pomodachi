import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface Task {
  id: number;
  title: string;
  completed: boolean;
  priority: number;
  dueDate: string | null;
  createdAt: string;
  updatedAt: string;
}

export function useTasks() {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchTasks = useCallback(async () => {
    setIsLoading(true);
    try {
      const result = await invoke<Task[]>('get_tasks');
      setTasks(result);
      setError(null);
    } catch (err) {
      console.error('获取任务失败:', err);
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchTasks();
  }, [fetchTasks]);

  const addTask = useCallback(async (title: string, priority: number = 0, dueDate?: string) => {
    try {
      const newTask = await invoke<Task>('add_task', {
        title,
        priority,
        dueDate: dueDate || null,
      });
      setTasks((prev) => [newTask, ...prev]);
      setError(null);
    } catch (err) {
      console.error('添加任务失败:', err);
      setError(String(err));
    }
  }, []);

  const completeTask = useCallback(async (id: number, completed: boolean) => {
    try {
      await invoke('complete_task', { id, completed });
      setTasks((prev) =>
        prev.map((t) => (t.id === id ? { ...t, completed } : t))
      );
      setError(null);
    } catch (err) {
      console.error('更新任务失败:', err);
      setError(String(err));
    }
  }, []);

  const deleteTask = useCallback(async (id: number) => {
    try {
      await invoke('delete_task', { id });
      setTasks((prev) => prev.filter((t) => t.id !== id));
      setError(null);
    } catch (err) {
      console.error('删除任务失败:', err);
      setError(String(err));
    }
  }, []);

  const pendingCount = tasks.filter((t) => !t.completed).length;

  return {
    tasks,
    isLoading,
    error,
    pendingCount,
    fetchTasks,
    addTask,
    completeTask,
    deleteTask,
  };
}
