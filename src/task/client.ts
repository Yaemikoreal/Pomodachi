import { invoke } from '@tauri-apps/api/core';
import { isTauri } from '../utils/tauri';

export interface Task {
  id: number;
  title: string;
  due_at?: number;
  repeat_rule: string;
  emotion_hint: string;
  is_done: boolean;
  created_at: number;
}

export interface CreateTaskInput {
  title: string;
  due_at?: number;
  repeat_rule?: string;
  emotion_hint?: string;
}

export async function listTasks(includeDone = false): Promise<Task[]> {
  if (!isTauri) return [];
  return await invoke<Task[]>('list_tasks', { include_done: includeDone });
}

export async function createTask(input: CreateTaskInput): Promise<number> {
  if (!isTauri) return -1;
  return await invoke<number>('create_task', { input });
}

export async function updateTask(input: Partial<Task> & { id: number }): Promise<void> {
  if (!isTauri) return;
  return await invoke('update_task', { input });
}

export async function completeTask(id: number): Promise<void> {
  if (!isTauri) return;
  return await invoke('complete_task', { id });
}

export async function deleteTask(id: number): Promise<void> {
  if (!isTauri) return;
  return await invoke('delete_task', { id });
}

export async function postponeTask(id: number, minutes: number): Promise<void> {
  if (!isTauri) return;
  return await invoke('postpone_task', { id, minutes });
}
