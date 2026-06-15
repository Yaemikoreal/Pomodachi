import { vi } from 'vitest';

/** Mock invoke 注册表：命令名 → 实现函数 */
type InvokeMocks = Record<string, (...args: any[]) => any>;

let invokeMocks: InvokeMocks = {};
/** listen 回调注册表：事件名 → 回调函数 */
const listenCallbacks: Record<string, (payload: any) => void> = {};

// Mock @tauri-apps/api/core
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn((cmd: string, args?: Record<string, unknown>) => {
    const mock = invokeMocks[cmd];
    if (!mock) {
      throw new Error(`[mockTauri] 未注册 invoke mock: ${cmd}`);
    }
    const result = mock(args);
    return Promise.resolve(result);
  }),
}));

// Mock @tauri-apps/api/event
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn((event: string, callback: (payload: any) => void) => {
    listenCallbacks[event] = callback;
    return Promise.resolve(() => {}); // 返回取消监听函数
  }),
}));

/**
 * 注册 invoke mock
 * @param cmd 命令名
 * @param fn 实现函数（接收 args，返回 mock 数据）
 */
export function __setInvokeMock(cmd: string, fn: (...args: any[]) => any) {
  invokeMocks[cmd] = fn;
}

/**
 * 模拟 Tauri 事件发射，触发 listen 注册的回调
 */
export function __emitEvent(event: string, payload: any) {
  const cb = listenCallbacks[event];
  if (cb) {
    cb({ payload });
  }
}

/** 重置所有 mock */
export function __resetMocks() {
  invokeMocks = {};
  Object.keys(listenCallbacks).forEach((key) => delete listenCallbacks[key]);
}
