/**
 * 端到端 IPC 测试 - 通过 Tauri invoke 调用真实后端
 *
 * 这些测试验证前后端完整交互，模拟用户实际工作流
 */

import { describe, it, expect, beforeAll, afterAll } from 'vitest';

// 注意：这些测试需要 Tauri 应用运行
// 在 CI 环境中可能需要跳过或使用 mock

const isTauriAvailable = typeof window !== 'undefined' && window.__TAURI__;

// 动态导入 Tauri API
let invoke: any;
let listen: any;

beforeAll(async () => {
  if (isTauriAvailable) {
    try {
      const core = await import('@tauri-apps/api/core');
      const event = await import('@tauri-apps/api/event');
      invoke = core.invoke;
      listen = event.listen;
    } catch (e) {
      console.warn('Tauri API 不可用，跳过 IPC 测试');
    }
  }
});

describe.skipIf(!isTauriAvailable)('端到端 IPC 测试', () => {
  describe('计时器功能', () => {
    it('获取计时器初始状态', async () => {
      const state = await invoke('get_timer_state');

      expect(state).toHaveProperty('mode');
      expect(state).toHaveProperty('duration');
      expect(state).toHaveProperty('remaining');
      expect(state).toHaveProperty('isRunning');
      expect(state).toHaveProperty('completedPomodoros');

      expect(state.mode).toBe('focus');
      expect(state.isRunning).toBe(false);
      expect(state.duration).toBeGreaterThan(0);
    });

    it('启动和暂停计时器', async () => {
      // 启动计时器
      const runningState = await invoke('start_timer');
      expect(runningState.isRunning).toBe(true);

      // 暂停计时器
      const pausedState = await invoke('pause_timer');
      expect(pausedState.isRunning).toBe(false);

      // 重置计时器
      await invoke('reset_timer');
    });

    it('跳过到下一阶段', async () => {
      const skippedState = await invoke('skip_timer');
      expect(skippedState.mode).toBe('shortBreak');
    });
  });

  describe('任务管理', () => {
    let taskId: number;

    it('添加任务', async () => {
      const task = await invoke('add_task', {
        title: 'E2E 测试任务',
        priority: 2,
      });

      expect(task).toHaveProperty('id');
      expect(task.title).toBe('E2E 测试任务');
      expect(task.priority).toBe(2);
      expect(task.completed).toBe(false);

      taskId = task.id;
    });

    it('获取任务列表', async () => {
      const tasks = await invoke('get_tasks');
      expect(Array.isArray(tasks)).toBe(true);

      const addedTask = tasks.find((t: any) => t.id === taskId);
      expect(addedTask).toBeDefined();
    });

    it('完成任务', async () => {
      await invoke('complete_task', { id: taskId, completed: true });

      const tasks = await invoke('get_tasks');
      const completedTask = tasks.find((t: any) => t.id === taskId);
      expect(completedTask.completed).toBe(true);
    });

    it('删除任务', async () => {
      await invoke('delete_task', { id: taskId });

      const tasks = await invoke('get_tasks');
      const deletedTask = tasks.find((t: any) => t.id === taskId);
      expect(deletedTask).toBeUndefined();
    });
  });

  describe('AI 聊天', () => {
    it('发送消息并接收回复', async () => {
      const response = await invoke('send_chat_message', {
        content: '你好，番茄猫！',
      });

      expect(typeof response).toBe('string');
      expect(response.length).toBeGreaterThan(0);
    });

    it('获取聊天历史', async () => {
      const history = await invoke('get_chat_history');
      expect(Array.isArray(history)).toBe(true);
    });

    it('清空聊天历史', async () => {
      await invoke('clear_chat_history');

      const history = await invoke('get_chat_history');
      expect(history.length).toBe(0);
    });
  });

  describe('成就系统', () => {
    it('获取成就列表', async () => {
      const achievements = await invoke('get_achievements');

      expect(Array.isArray(achievements)).toBe(true);
      expect(achievements.length).toBeGreaterThan(0);

      const firstAch = achievements[0];
      expect(firstAch).toHaveProperty('id');
      expect(firstAch).toHaveProperty('key');
      expect(firstAch).toHaveProperty('name');
      expect(firstAch).toHaveProperty('description');
    });

    it('获取专注统计', async () => {
      const stats = await invoke('get_focus_stats');

      expect(stats).toHaveProperty('totalPomodoros');
      expect(stats).toHaveProperty('totalFocusSeconds');
      expect(stats).toHaveProperty('currentStreak');
      expect(stats).toHaveProperty('longestStreak');
    });
  });

  describe('设置管理', () => {
    it('获取所有设置', async () => {
      const settings = await invoke('get_all_settings');

      expect(Array.isArray(settings)).toBe(true);
      expect(settings.length).toBeGreaterThan(0);

      const languageSetting = settings.find((s: any) => s.key === 'language');
      expect(languageSetting).toBeDefined();
    });

    it('修改设置', async () => {
      await invoke('set_setting', { key: 'testKey', value: 'testValue' });

      const settings = await invoke('get_all_settings');
      const testSetting = settings.find((s: any) => s.key === 'testKey');
      expect(testSetting.value).toBe('testValue');
    });
  });

  describe('分心检测', () => {
    it('获取黑名单', async () => {
      const blocklist = await invoke('get_blocklist');
      expect(Array.isArray(blocklist)).toBe(true);
    });

    it('添加到黑名单', async () => {
      await invoke('add_to_blocklist', { processName: 'TestApp.exe' });

      const blocklist = await invoke('get_blocklist');
      const addedItem = blocklist.find((item: any) => item.processName === 'TestApp.exe');
      expect(addedItem).toBeDefined();
    });

    it('从黑名单移除', async () => {
      await invoke('remove_from_blocklist', { processName: 'TestApp.exe' });

      const blocklist = await invoke('get_blocklist');
      const removedItem = blocklist.find((item: any) => item.processName === 'TestApp.exe');
      expect(removedItem).toBeUndefined();
    });

    it('启停监听', async () => {
      await invoke('start_monitoring');

      const isMonitoring = await invoke('is_monitoring');
      expect(isMonitoring).toBe(true);

      await invoke('stop_monitoring');

      const isStopped = await invoke('is_monitoring');
      expect(isStopped).toBe(false);
    });
  });

  describe('宠物状态', () => {
    it('获取宠物情绪', async () => {
      const mood = await invoke('get_pet_mood');

      expect(typeof mood).toBe('string');
      expect(['happy', 'focused', 'tired', 'sleeping', 'listening', 'thinking']).toContain(mood);
    });
  });

  describe('完整用户会话', () => {
    it('模拟完整使用流程', async () => {
      // 1. 获取宠物状态
      const mood = await invoke('get_pet_mood');
      expect(mood).toBeDefined();

      // 2. 添加任务
      const task = await invoke('add_task', {
        title: '完成 E2E 测试',
        priority: 1,
      });
      expect(task.id).toBeDefined();

      // 3. 开始专注
      await invoke('start_timer');
      await invoke('pause_timer');
      await invoke('reset_timer');

      // 4. 发送聊天消息
      const chatResponse = await invoke('send_chat_message', {
        content: '测试消息',
      });
      expect(chatResponse).toBeDefined();

      // 5. 完成任务
      await invoke('complete_task', { id: task.id, completed: true });

      // 6. 清理
      await invoke('delete_task', { id: task.id });
      await invoke('clear_chat_history');
    });
  });
});

describe('事件监听测试', () => {
  it.skipIf(!isTauriAvailable)('监听计时器事件', async () => {
    const events: any[] = [];

    const unlisten = await listen('timer-tick', (event: any) => {
      events.push(event.payload);
    });

    // 启动计时器
    await invoke('start_timer');

    // 等待一些事件
    await new Promise((resolve) => setTimeout(resolve, 2000));

    // 暂停计时器
    await invoke('pause_timer');
    await invoke('reset_timer');

    // 取消监听
    unlisten();

    // 验证收到了事件
    expect(events.length).toBeGreaterThan(0);
  });
});
