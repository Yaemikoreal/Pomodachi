import { describe, it, expect, beforeEach } from 'vitest';
import { renderHook, waitFor, act } from '@testing-library/react';
import { useTasks } from '../useTasks';
import { __setInvokeMock, __resetMocks } from '../../test-utils/mockTauri';

const MOCK_TASKS = [
  { id: 1, title: '完成任务 A', completed: false, priority: 1, dueDate: null, createdAt: '2026-01-01T00:00:00Z', updatedAt: '2026-01-01T00:00:00Z' },
  { id: 2, title: '完成任务 B', completed: true, priority: 0, dueDate: null, createdAt: '2026-01-01T00:00:00Z', updatedAt: '2026-01-01T00:00:00Z' },
];

describe('useTasks', () => {
  beforeEach(() => {
    __resetMocks();
  });

  it('加载任务列表', async () => {
    __setInvokeMock('get_tasks', () => MOCK_TASKS);

    const { result } = renderHook(() => useTasks());

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(result.current.tasks.length).toBe(2);
    expect(result.current.pendingCount).toBe(1);
    expect(result.current.error).toBeNull();
  });

  it('空任务列表', async () => {
    __setInvokeMock('get_tasks', () => []);

    const { result } = renderHook(() => useTasks());

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(result.current.tasks).toEqual([]);
    expect(result.current.pendingCount).toBe(0);
  });

  it('添加任务', async () => {
    const newTask = { id: 3, title: '新任务', completed: false, priority: 2, dueDate: null, createdAt: '2026-01-01T00:00:00Z', updatedAt: '2026-01-01T00:00:00Z' };
    __setInvokeMock('get_tasks', () => []);
    __setInvokeMock('add_task', () => newTask);

    const { result } = renderHook(() => useTasks());

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await act(async () => {
      await result.current.addTask('新任务', 2);
    });

    expect(result.current.tasks.length).toBe(1);
    expect(result.current.tasks[0].title).toBe('新任务');
    expect(result.current.tasks[0].priority).toBe(2);
  });

  it('完成任务', async () => {
    __setInvokeMock('get_tasks', () => MOCK_TASKS);
    __setInvokeMock('complete_task', () => {});

    const { result } = renderHook(() => useTasks());

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await act(async () => {
      await result.current.completeTask(1, true);
    });

    const task = result.current.tasks.find((t) => t.id === 1);
    expect(task?.completed).toBe(true);
    expect(result.current.pendingCount).toBe(0);
  });

  it('删除任务', async () => {
    __setInvokeMock('get_tasks', () => MOCK_TASKS);
    __setInvokeMock('delete_task', () => {});

    const { result } = renderHook(() => useTasks());

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    await act(async () => {
      await result.current.deleteTask(1);
    });

    expect(result.current.tasks.length).toBe(1);
    expect(result.current.tasks[0].id).toBe(2);
  });

  it('加载失败时设置 error', async () => {
    __setInvokeMock('get_tasks', () => { throw new Error('数据库错误'); });

    const { result } = renderHook(() => useTasks());

    await waitFor(() => expect(result.current.isLoading).toBe(false));

    expect(result.current.error).toBeTruthy();
    expect(result.current.tasks).toEqual([]);
  });
});
