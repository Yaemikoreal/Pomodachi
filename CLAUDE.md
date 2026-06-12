# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

**Pomodoro Pet (番茄宠物)** — 基于 Tauri v2 的本地优先、游戏化 AI 番茄钟桌面宠物。

核心理念：把专注时间具象化为宠物的生命值和成长。专注时宠物开心成长，摸鱼时宠物受伤生气。AI 作为"宠物灵魂"提供拟人化情绪反馈。

## 技术栈

| 层 | 技术 |
|---|---|
| 桌面框架 | Tauri v2 (Rust) |
| 前端 | React 19 + TypeScript + TailwindCSS + Framer Motion |
| 后端核心 | Rust |
| 本地存储 | SQLite (rusqlite) |
| AI 接入 | OpenAI 兼容格式 (本地 Ollama / 云端 API) |
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
  - StatusPanel 状态面板       - monitor.rs 进程监听 (sysinfo)
  - Hooks (useTimer/usePet)   - db.rs SQLite 持久化
                              - pet.rs 宠物状态机
```

前后端通过 Tauri IPC 通信。计时逻辑在 Rust 后端运行，前端仅负责渲染和交互。

## Rust 后端模块

### db.rs - 数据库
- SQLite 数据库，存储于 Tauri app data 目录
- 三张表：`pet`（宠物状态）、`pomodoro_record`（专注记录）、`blocklist`（分心黑名单）
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
- `PetManager` 结构体管理宠物状态
- 情绪状态：Happy, Focused, Tired, Angry, Sad, Sleeping
- 游戏逻辑：完成专注 +50 EXP +10 金币，分心 -5 HP，每 100 EXP 升一级

## 前端 Hooks

| Hook | 用途 |
|------|------|
| `useTimer` | 计时器状态管理，调用 Rust IPC |
| `usePet` | 宠物状态管理，监听分心事件 |
| `usePomodoro` | 专注记录管理 |
| `useBlocklist` | 黑名单管理 |

## Tauri IPC 命令

```rust
// 计时器
get_timer_state, start_timer, pause_timer, reset_timer, skip_timer, set_timer_mode

// 宠物
get_pet_status, feed_pet, reset_pet

// 专注记录
add_pomodoro_record, complete_pomodoro_record, get_pomodoro_history, get_today_pomodoro_count

// 黑名单
get_blocklist, add_to_blocklist, remove_from_blocklist

// 监听器
start_monitoring, stop_monitoring, is_monitoring
```

## 开发路线

1. **Phase 1 (MVP)** ✅: Tauri + React 骨架、透明悬浮窗、Spine 动画、拖动/缩放
2. **Phase 2 (游戏化)** ✅: Rust 计时器、SQLite 存储、进程监听、宠物状态机
3. **Phase 3 (AI)**: 侧边栏面板、OpenAI API 接入、情绪反馈、Chat 界面
4. **Phase 4 (打磨)**: 托盘优化、多皮肤、打包发布

## 开发约定

- Tauri v2 项目，前端 pnpm，后端 cargo
- Rust 后端遵循模块化划分，每个核心功能独立文件
- 前端组件按功能划分，Hooks 封装 Tauri IPC 调用
- SQLite 数据库文件存于 Tauri app data 目录
- 像素动画素材放 `public/sprites/`，按宠物状态分子目录
- Commit message 使用中文，格式：`[类型]：精炼概要`
