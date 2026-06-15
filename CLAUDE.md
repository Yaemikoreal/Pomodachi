# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

**Pomodoro Pet (番茄宠物)** — 基于 Tauri v2 的本地优先 AI 番茄钟桌面宠物/个人助理。

核心理念：把专注时间具象化为宠物的情绪反馈。专注时宠物开心陪伴，摸鱼时宠物发出提醒。AI 作为"宠物灵魂"提供拟人化交互和任务管理帮助。

## 技术栈

| 层 | 技术 |
|---|---|
| 桌面框架 | Tauri v2 (Rust) |
| 前端 | React 19 + TypeScript + TailwindCSS + Framer Motion |
| 后端核心 | Rust |
| 本地存储 | SQLite (rusqlite) |
| AI 接入 | Claude Code CLI 子进程通信 |
| 进程监听 | sysinfo + Windows API |
| 动画 | PixiJS + Spine |

## 开发命令

```bash
# 启动开发服务器（前端 + 后端）
pnpm tauri dev

# 仅前端开发
pnpm dev

# 构建生产版本
pnpm tauri build

# 运行 Rust 测试
cd src-tauri && cargo test

# 检查 Rust 代码
cd src-tauri && cargo check

# 格式化 Rust 代码
cd src-tauri && cargo fmt
```

## 架构

```
Frontend (React/TS)  ←IPC→  Backend (Rust)
  - SpinePet 宠物动画         - timer.rs 番茄钟引擎 (防休眠被杀)
  - StatusPanel 状态面板       - monitor.rs 进程监听 (Windows API)
  - Sidebar (聊天/任务/设置)   - db.rs SQLite 持久化 (7 表)
  - ChatPanel/ChatBubble 聊天  - pet.rs 宠物情绪状态机
  - TaskPanel 任务管理         - ai.rs Claude CLI 子进程通信
  - Hooks (7 个 IPC 封装)     - task.rs 任务管理器
                              - notification.rs Windows 通知
```

前后端通过 Tauri IPC 通信。计时逻辑在 Rust 后端运行，前端仅负责渲染和交互。

## Rust 后端模块

### db.rs - 数据库
- SQLite 数据库，存储于 Tauri app data 目录
- 七张表：`pet`（宠物状态）、`pomodoro_record`（专注记录）、`blocklist`（分心黑名单）、`chat_message`（聊天记录）、`ai_config`（AI 配置）、`tasks`（待办任务）、`app_settings`（应用设置）
- 所有数据库操作通过 `Database` 结构体封装

### timer.rs - 番茄钟引擎
- `PomodoroTimer` 结构体管理计时状态
- 三种模式：Focus(25分钟)、ShortBreak(5分钟)、LongBreak(15分钟)
- 每 4 个番茄自动进入长休息
- 通过 Tokio 异步任务驱动计时，发送 `timer-tick` 和 `timer-complete` 事件

### monitor.rs - 进程监听
- `ProcessMonitor` 结构体管理监听状态
- Windows 平台使用 `GetForegroundWindow` + `GetWindowThreadProcessId` 获取前台进程
- 每 3 秒检测一次，匹配黑名单后发送 `distraction-detected` 事件

### pet.rs - 宠物状态机
- `PetManager` 结构体管理宠物情绪状态
- 情绪状态：Happy, Focused, Tired, Sleeping, Listening, Thinking
- 事件驱动：专注开始→Focused、休息→Sleeping、专注完成→Happy、聊天→Listening、AI 思考→Thinking

## 前端 Hooks

| Hook | 用途 |
|------|------|
| `useTimer` | 计时器状态管理，调用 Rust IPC |
| `usePet` | 宠物情绪状态管理 |
| `usePomodoro` | 专注记录管理 |
| `useBlocklist` | 黑名单管理 |
| `useChat` | AI 聊天 + 历史 + 配置管理 |
| `useTasks` | 任务 CRUD |
| `useSettings` | 应用设置读写（10 项设置） |

## Tauri IPC 命令

```rust
// 计时器
get_timer_state, start_timer, pause_timer, reset_timer, skip_timer, set_timer_mode

// 宠物
get_pet_mood, set_pet_mood

// 专注记录
add_pomodoro_record, complete_pomodoro_record, get_pomodoro_history, get_today_pomodoro_count

// 黑名单
get_blocklist, add_to_blocklist, remove_from_blocklist

// 监听器
start_monitoring, stop_monitoring, is_monitoring

// AI 聊天
send_chat_message, get_chat_history, clear_chat_history, get_ai_config, update_ai_config, clear_ai_session

// 任务
get_tasks, add_task, update_task, complete_task, delete_task

// 设置
get_setting, set_setting, get_all_settings

// 应用控制
exit_app, set_pet_window_size
```

## 开发路线

1. **Phase 1 (MVP)** ✅: Tauri + React 骨架、透明悬浮窗、Spine 动画、拖动/缩放
2. **Phase 2 (游戏化)** ✅: Rust 计时器、SQLite 存储、进程监听、宠物状态机
3. **Phase 3 (AI)** ✅: 侧边栏面板、Claude Code CLI 集成、情绪感知聊天、任务管理
4. **Phase 4 (打磨)** ✅: 系统托盘、设置面板、个人助理转型、消息通知

## 开发约定

- Tauri v2 项目，前端 pnpm，后端 cargo
- Rust 后端遵循模块化划分，每个核心功能独立文件
- 前端组件按功能划分，Hooks 封装 Tauri IPC 调用
- SQLite 数据库文件存于 Tauri app data 目录
- 像素动画素材放 `public/spines/`，按皮肤分子目录
- Commit message 使用中文，格式：`[类型]：精炼概要`
