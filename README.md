# Toumato - 番茄宠物

基于 Tauri v2 的本地优先 AI 番茄钟桌面宠物。专注时间具象化为宠物的情绪陪伴 -- 专注时宠物开心陪伴，摸鱼时宠物发出提醒。

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Tauri](https://img.shields.io/badge/Tauri-v2-blue.svg)
![React](https://img.shields.io/badge/React-19-blue.svg)
![Rust](https://img.shields.io/badge/Rust-1.70+-blue.svg)

## 特性

- **桌面宠物** -- 透明无边框窗口，始终置顶，Spine 4.1 骨骼动画
- **番茄钟** -- 25+5+15 专注模式，支持自定义时长
- **Claude Code 状态联动** -- 宠物实时反映 Claude Code 的工作状态（活跃/空闲/未运行）
- **成就系统** -- 专注统计、里程碑解锁、连续天数追踪
- **分心检测** -- 自动监听前台窗口，检测摸鱼行为
- **AI 聊天** -- Claude Code CLI 驱动的智能对话
- **任务管理** -- 待办事项增删改查

## 快速开始

### 环境要求

- [Node.js](https://nodejs.org/) >= 18
- [pnpm](https://pnpm.io/) >= 8
- [Rust](https://www.rust-lang.org/) >= 1.70
- [Tauri CLI](https://tauri.app/) >= 2
- [Claude Code CLI](https://docs.anthropic.com/en/docs/claude-code)（AI 功能依赖）

### 安装与运行

```bash
git clone https://github.com/Yaemikoreal/toumato.git
cd toumato
pnpm install
pnpm tauri dev
```

### 构建

```bash
pnpm tauri build
```

## 使用说明

### 宠物状态

宠物通过监听本机 Claude Code 进程自动切换情绪：

| Claude Code 状态 | 宠物情绪 | 说明 |
|---|---|---|
| CPU 活跃 (>10%) | Thinking | 正在执行任务 |
| CPU 空闲 | Happy | 运行中，等待输入 |
| 空闲超过 10 分钟 | Tired | 长时间无活动 |
| 未运行 | Sleeping | 进程不存在 |
| 异常退出 | Error | 进程意外消失 |

### 托盘菜单

右键系统托盘图标可手动覆盖宠物情绪（默认 5 分钟后自动恢复为 Claude Code 自动检测状态）。

### 基本操作

| 操作 | 说明 |
|------|------|
| 拖动 | 鼠标左键按住宠物拖动 |
| 计时 | 右下角状态面板控制专注/休息计时 |
| 任务 | 侧边栏任务标签管理待办事项 |
| 设置 | 侧边栏设置标签调整参数 |

## 项目结构

```
toumato/
├── src-tauri/              # Rust 后端 (Tauri)
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs          # IPC 命令 + 事件监听 + 系统托盘
│   │   ├── timer.rs        # 番茄钟引擎 (Focus/ShortBreak/LongBreak)
│   │   ├── pet.rs          # 宠物情绪状态机 (6 种情绪 + 手动覆盖)
│   │   ├── claude_monitor.rs  # Claude Code 进程监控
│   │   ├── monitor.rs      # 前台进程监听 (Windows API)
│   │   ├── ai.rs           # Claude CLI 子进程通信
│   │   ├── db.rs           # SQLite 数据库 (9 张表)
│   │   ├── task.rs         # 任务管理器
│   │   └── notification.rs # Windows 通知
│   └── tauri.conf.json
├── src/                    # React 前端
│   ├── components/         # SpinePet, StatusPanel, Sidebar, ChatPanel, TaskPanel
│   ├── hooks/              # useTimer, usePet, useChat, useTasks, useClaudeCodeStatus
│   ├── test-utils/         # mockTauri.ts (测试基础设施)
│   ├── App.tsx
│   └── main.tsx
├── public/
│   └── spines/             # Spine 动画资源
└── 素材/                   # 桌面宠物素材
```

## 技术栈

| 层 | 技术 |
|---|---|
| 桌面框架 | Tauri v2 (Rust) -- 单窗口透明浮窗，始终置顶 |
| 前端 | React 19 + TypeScript 5.8 + Framer Motion 12 |
| 动画 | PixiJS Legacy 7.4 + @pixi-spine/all-4.1 + Spine 4.1 |
| 本地存储 | SQLite (rusqlite 0.31) |
| AI | Claude Code CLI 子进程通信 |
| 进程监听 | Windows API (GetForegroundWindow) + sysinfo |
| 测试 | Vitest 4 (前端) + cargo test (后端) |

## 开发

```bash
# 前端测试
pnpm test

# 后端测试
cd src-tauri && cargo test

# TypeScript 类型检查
pnpm tsc --noEmit

# Rust 编译检查
cd src-tauri && cargo check
```

## 相关文档

- [设计方案](设计方案.md) -- 详细设计文档
- [CLAUDE.md](CLAUDE.md) -- Claude Code 开发指南
- [故障排除](docs/troubleshooting.md) -- 常见问题

## 许可证

[MIT License](LICENSE)
