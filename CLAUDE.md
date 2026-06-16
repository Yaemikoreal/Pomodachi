# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

**Pomodoro Pet (番茄宠物 / Toumato)** — 基于 Tauri v2 的本地优先 AI 番茄钟桌面宠物/个人助理。

核心理念：专注时间具象化为宠物情绪反馈。专注时宠物开心陪伴，摸鱼时宠物提醒，AI 作为"宠物灵魂"提供拟人化交互和任务管理。

## 技术栈

| 层 | 技术 |
|---|---|
| 桌面框架 | Tauri v2 (Rust) — 单窗口透明浮窗，始终置顶，无边框 |
| 前端 | React 19 + TypeScript 5.8 + Framer Motion 12 (内联样式，无 TailwindCSS) |
| 动画 | PixiJS Legacy 7.4 (`forceCanvas`) + `@pixi-spine/all-4.1` + Spine 4.1 骨骼动画 |
| 本地存储 | SQLite via `rusqlite 0.31` (bundled)，9 张表 |
| AI | Claude Code CLI 子进程通信 (`claude -p <prompt> --output-format json --bare`) |
| 进程监听 | Windows API (`GetForegroundWindow`) + `sysinfo 0.30`，每 3 秒轮询 |
| 测试 | Vitest 4 (前端) + `cargo test` (后端) |
| 包管理 | pnpm (前端) + cargo (后端) |

## 开发命令

```bash
# 启动开发服务器（前端 + 后端）
pnpm tauri dev

# 仅前端开发（无 Tauri 后端）
pnpm dev

# 构建生产版本
pnpm tauri build

# === 后端测试 ===
cd src-tauri

# 运行所有 Rust 测试
cargo test

# 运行指定模块测试
cargo test test_timer              # 按函数名过滤
cargo test test::timer              # 按模块路径过滤
cargo test test_achievements_init   # db.rs 成就测试

# 查看测试 stdout（--nocapture）
cargo test -- --nocapture

# 检查 Rust 代码（不编译）
cargo check

# 格式化 Rust 代码
cargo fmt

# 回到项目根
cd ..

# === 前端测试 ===
pnpm test              # 运行所有 Vitest 测试
pnpm vitest -- --reporter verbose   # 详细输出
pnpm vitest run src/hooks/__tests__/useTimer.test.ts  # 单文件测试
pnpm tsc --noEmit      # TypeScript 类型检查
pnpm build             # tsc + vite build（生产构建用 pnpm tauri build 替代）
```

## 架构概览

```
┌─────────────────────────────────────────────────────────────────┐
│                      React 前端 (src/)                          │
│                                                                │
│  App.tsx (根组件)                                               │
│  ├── SpinePet.tsx        ─ PixiJS + Spine 骨骼动画渲染         │
│  ├── StatusPanel.tsx     ─ 计时器面板 (开始/暂停/重置/跳过)     │
│  ├── Sidebar.tsx         ─ 右侧滑动面板 (聊天/任务/设置 3 Tab)  │
│  ├── ChatBubble.tsx      ─ 桌面浮动聊天气泡                    │
│  ├── ChatPanel.tsx       ─ 侧栏内完整聊天视图                  │
│  ├── TaskPanel.tsx       ─ 任务列表 (增/完成/删)              │
│  └── AchievementToast.tsx─ 成就解锁通知动画 (金渐变 Toast)      │
│                                                                │
│  Hooks (IPC 封装层):                                            │
│  useTimer | usePet | usePomodoro | useBlocklist |              │
│  useChat | useTasks | useSettings | useAchievements            │
│       │                                                       │
└───────┼─────────────────────────────────────────────────────────┘
        │ Tauri IPC (invoke / listen)
        ▼
┌─────────────────────────────────────────────────────────────────┐
│                   Rust 后端 (src-tauri/src/)                    │
│                                                                │
│  lib.rs          → 43 个 IPC 命令 + 事件监听 + 系统托盘         │
│    AppState      → 全局共享状态 (Arc 包裹所有管理器)            │
│  timer.rs        → PomodoroTimer (Focus/ShortBreak/LongBreak)  │
│  pet.rs          → PetManager (6 种情绪状态机 + 皮肤 ID)        │
│  db.rs           → Database (SQLite, 9 张表 CRUD)              │
│  monitor.rs      → ProcessMonitor (Windows 前台进程检测)        │
│  ai.rs           → AiClient (Claude CLI 子进程, 流式/非流式)    │
│  task.rs         → TaskManager (任务 CRUD 包装)                │
│  notification.rs → Windows MessageBoxW 通知                     │
└─────────────────────────────────────────────────────────────────┘
```

**关键架构模式**:

- **AppState 共享状态**: `lib.rs` 的 `AppState` 包含全部管理器，通过 `tauri::State<'_, AppState>` 注入命令，管理器内部用 `Arc<Mutex<T>>` 或 `Arc<tokio::sync::Mutex<T>>` 保证线程安全
- **事件驱动**: Rust → 前端单向推送：`timer-tick`(每秒)、`timer-complete`、`distraction-detected`、`achievement-unlocked`、`pet-skin-changed`
- **命令调用**: 前端通过 `invoke()` 调用 Rust IPC 命令，Hook 层封装 invoke 逻辑，组件层只调用 Hook
- **定时器在 Rust 侧**: 使用 Tokio 异步任务驱动倒计时，防止窗口休眠时计时丢失
- **成就系统**: `db.update_focus_stats()` 累计数据 → `db.check_achievements()` 检查条件 → IPC 事件 `achievement-unlocked` → 前端 Toast

## 数据库 (SQLite, 9 张表)

`toumato.db` 存储在 Tauri app data 目录，通过 `Database` 结构体 (内部 `Mutex<Connection>`) 访问。

| 表名 | 关键字段 | 用途 |
|---|---|---|
| `pet` | id(1), name, mood, skin_id, updated_at | 单行宠物状态 |
| `pomodoro_record` | id, started_at, finished_at, duration, completed, distraction_count | 专注会话日志 |
| `blocklist` | id, process_name(UNIQUE), added_at | 分心应用黑名单 |
| `chat_message` | id, role, content, created_at | 聊天记录 |
| `ai_config` | id(1), max_turns(3), max_budget_usd(0.1), updated_at | AI 参数（单行） |
| `tasks` | id, title, completed, priority, due_date, created_at, updated_at | 待办事项 |
| `app_settings` | key(PK), value, updated_at | 键值设置（10 项） |
| `achievements` | id, key(UNIQUE), name, description, unlocked_at, icon | 9 个种子成就 |
| `focus_stats` | id(1), total_pomodoros, total_focus_seconds, current_streak, longest_streak, last_focus_date | 累计统计（单行） |

有迁移机制 (`run_migrations`)，当前迁移：pet 表添加 `skin_id` 列。旧 `ai_config` 表兼容处理（检测旧结构自动重建）。

## 前端组件详解

### SpinePet.tsx — 宠物渲染核心
- 使用 PixiJS + Spine 4.1 渲染骨骼动画
- **Track 0**: 身体待机动画 (`Move_Sit_Idle`)
- **Track 1**: 面部表情，根据 mood 映射：happy→`Face_Happy`, focused→`Face_Star`, tired/sleeping→`Face_CloseEyesHappy`, listening→`Face_Happy`, thinking→`Face_Star`
- 渲染后自动裁剪 Tauri 窗口贴合精灵边界 (`fitWindowToSpine`)
- 皮肤切换：销毁旧 PIXI 实例 → 淡出 → 重新加载 → 淡入
- 错误降级：资产加载失败时显示错误文本覆盖层
- 鼠标按下触发 `window.startDragging()` 实现拖动

### StatusPanel.tsx — 状态面板 (右下角)
- 折叠式浮窗，显示 mood emoji、宠物名、番茄计数
- 计时器：模式名、格式化时间、进度条、控制按钮（开始/暂停/重置/跳过）
- 专注统计：累计专注时间、番茄数、连续天数
- 成就列表：3 列网格，已解锁高亮，可展开/折叠

### Sidebar.tsx — 侧边栏 (右侧 320px 滑动抽屉)
- 3 Tab：聊天、任务、设置
- 设置面板包含：语言、开机自启、透明度滑块、3 个计时器时长、分心检测开关/冷却、AI 轮次/预算、皮肤网格选择器
- 皮肤选择调用 `set_pet_skin` IPC → Rust 更新内存+DB+发出 `pet-skin-changed` 事件
- framer-motion spring 动画

### ChatPanel.tsx / ChatBubble.tsx — 聊天
- ChatPanel 在侧栏内完整显示，ChatBubble 是桌面浮动迷你版（底部居中 280px）
- 最近消息列表 + 输入框 + Enter 发送 + Escape 关闭
- 通过 `useChat` Hook 调用 Rust IPC `send_chat_message`
- Rust 端：存用户消息 → 拼系统提示（含情绪+今日专注数）+ 最近 20 条历史 → 调 Claude CLI → 存回复 → 返回

### TaskPanel.tsx / AchievementToast.tsx
- TaskPanel：添加表单（标题+优先级）、checkbox 切换完成、骨架加载/空状态/错误状态
- AchievementToast：`achievement-unlocked` 事件触发，金渐变 Toast，自动 5 秒消失，点击关闭

## 前端 Hooks (8 个)

| Hook | IPC 命令 | 说明 |
|---|---|---|
| `useTimer` | get/start/pause/reset/skip/set_timer_state | 计时器状态；监听 `timer-tick`/`timer-complete` 事件 |
| `usePet` | get_pet_mood | 宠物情绪+名字；初始加载，后续靠事件驱动 |
| `usePomodoro` | add/complete/get_record/get_today_count | 专注记录 CRUD |
| `useBlocklist` | get/add/remove + start/stop/is_monitoring | 黑名单管理；支持监听器启停 |
| `useChat` | send/get/clear + get/update_ai_config + clear_session | AI 聊天；乐观更新(先显示用户消息)；失败显示 fallback 错误消息 |
| `useTasks` | get/add/complete/delete | 任务 CRUD；乐观更新；暴露 `pendingCount` |
| `useSettings` | get_all/set_setting | 10 项设置；字符串 ↔ 类型转换 (`parseSettings`/`serializeSettings`)；手动保存模式 |
| `useAchievements` | get_achievements/get_focus_stats | 成就列表+专注统计；监听 `achievement-unlocked` 事件；5 秒自动清除 Toast |

**测试模式**: 所有 Hook 测试使用 `src/test-utils/mockTauri.ts`，它 mock 了 `@tauri-apps/api/core` 的 `invoke` 和 `@tauri-apps/api/event` 的 `listen`。暴露 3 个工具函数：`__setInvokeMock`(注册返回值)、`__emitEvent`(模拟事件推送)、`__resetMocks`(重置)。

## Rust 后端模块

### timer.rs
- `PomodoroTimer` 结构体，内部管理 `Arc<Mutex<TimerState>>`
- 默认状态：Focus / 25min / 0 completed / not running
- 三种时长：Focus(25min)、ShortBreak(5min)、LongBreak(15min)
- cancel 通过 `tokio::sync::oneshot::channel` 发送取消信号
- 自动模式切换：每 4 个番茄 (completed % 4 == 0) 进入 LongBreak
- 6 个单元测试

### pet.rs
- `PetManager`：纯内存状态机，`Arc<Mutex<String>>` 存 mood 和 skin_id
- 情绪事件：`on_focus_start`→Focused、`on_rest`→Sleeping、`on_focus_complete`→Happy、`on_chat_start`→Listening、`on_ai_thinking`→Thinking
- 皮肤通过 `set_pet_skin` IPC 同步持久化到 DB
- 9 个单元测试（tokio::test）

### db.rs
- `Database` 结构体：`Mutex<Connection>`，所有方法同步
- `#[cfg(test)]` 模式下有 `new_in_memory()` 创建内存数据库
- `check_achievements` 按 9 个条件查询 stats 并原子解锁（`unlocked_at IS NULL` 防止重复）
- 18 个单元测试

### ai.rs
- `AiClient`：通过子进程 `claude -p <prompt> --output-format json --bare` 通信
- `chat()` 非流式：120 秒超时，解析 JSON 输出，提取 `result` 字段
- `chat_stream()` 流式：逐行解析 `stream-json`，通过 `mpsc::channel` 推送
- 会话续传：从 JSON 响应提取 `session_id`，后续请求加 `--resume <sid>`
- 系统提示通过 `generate_system_prompt(mood, today_count)` 动态生成

### monitor.rs
- Windows 专用：`GetForegroundWindow` + `GetWindowThreadProcessId` → `sysinfo` 匹配黑名单
- 3 秒轮询间隔，30 秒防抖冷却
- 非 Windows 返回 `None` (not implemented)
- 通过 `AppHandle` 发出 `distraction-detected` 事件

### task.rs / notification.rs
- task.rs：Database 的轻量包装，按 `completed ASC, priority DESC, created_at DESC` 排序，4 个测试
- notification.rs：使用 Windows `MessageBoxW`，标记为 "temporary"，TODO 待替换为 WinRT Toast

## 事件流

```
┌────────────┐     timer-tick(每秒)     ┌──────────────┐
│            │ ◄━━━━━━━━━━━━━━━━━━━━━━ │ PomodoroTimer │
│            │     timer-complete       │              │
│            │ ◄━━━━━━━━━━━━━━━━━━━━━━ │  (timer.rs)  │
│            │                          └──────────────┘
│            │   distraction-detected   ┌──────────────┐
│  前端      │ ◄━━━━━━━━━━━━━━━━━━━━━━ │ ProcessMonitor│
│  React     │                          │  (monitor.rs) │
│            │   achievement-unlocked   └──────────────┘
│            │ ◄━━━━━━━━━━━━━━━━━━━━━━  ┌──────────────┐
│            │                          │  Database     │
│            │   pet-skin-changed       │  (db.rs)      │
│            │ ◄━━━━━━━━━━━━━━━━━━━━━━  └──────────────┘
│            │
│            └── invoke ──→ Rust IPC commands ←── AppState
```

## 宠物模型 (Spine 4.1)

Spine 骨骼动画素材在 `public/spines/firefly/`：
- `atlases_0_atlas_0` — 纹理图集定义 (136 个区域：面部/身体/头发/配件/特效)
- `atlases_0_textures_0_0.png` — 2048×2048 图集
- `model0.json` — 模型配置（7 个点击区域、亲密值系统、自动呼吸/眨眼）
- `skeleton_0` — 二进制骨架数据

动画映射：前端 `SpinePet.tsx` 中 `MOOD_FACE_MAP` 将 6 种情绪映射为 Spine 中定义的面部动画名。

## 测试策略

### Rust 后端 (38+ 测试)
- `cargo test` 跑所有，包含集成测试（数据库 CRUD 全链路）
- db.rs 测试用 `new_in_memory()` 防止污染真实 DB
- pet.rs 和 timer.rs 测试纯逻辑，无外部依赖

### 前端 (23+ 测试)
- `pnpm test` 跑 Vitest，使用 `jsdom` 环境
- `mockTauri.ts` 提供 Tauri IPC 的完全 mock
- 测试模式：`__setInvokeMock` 注册 → 调用 hook → 断言状态变化
- 事件测试：`__emitEvent` 推送事件 → 断言 hook 状态更新
- 每个测试前无需手动 reset（`setupTests.ts` 在 vitest 配置中自动运行）

## 项目结构强调

```
toumato/
└── src/                        # React 前端
    ├── components/             # 7 个组件
    ├── hooks/                  # 8 个 Hook
    │   └── __tests__/          # 4 个测试文件 (23+ 测试)
    ├── test-utils/             # mockTauri.ts (测试基础设施)
    ├── data/                   # skins.ts (皮肤定义)
    └── types/                  # model.ts (Spine 类型)
└── src-tauri/                  # Rust 后端
    └── src/
        ├── *.rs               # 8 个模块
        └── lib.rs             # 43 IPC 命令 + 事件监听 + 托盘
```

## 开发约定

- Tauri v2 项目，前端 pnpm，后端 cargo
- Rust 后端 8 个模块各司其职，AppState 统一注入
- 前端 8 个 Hook 封装所有 IPC 调用，组件只消费 Hook
- 所有 IPC 结构体用 `#[serde(rename_all = "camelCase")]`（Rust→JS 自动转换）
- SQLite 数据库文件存于 Tauri app data 目录
- Spine 素材按皮肤分子目录放 `public/spines/`
- 新功能开发：先写测试（mock 前端 or 内存 DB 后端）→ 确认失败 → 写实现 → 确认通过
- Commit message 使用中文，格式：`[类型]：精炼概要`
- AI 集成依赖外部 `claude` CLI，测试时需 mock
