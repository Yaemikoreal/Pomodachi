# 🍅 番茄宠物 (Toumato)

基于 Tauri v2 的本地优先 AI 番茄钟桌面宠物/个人助理。

> 把专注时间具象化为宠物的情绪陪伴。专注时宠物开心陪伴，摸鱼时宠物发出提醒。AI 作为"宠物灵魂"提供拟人化交互和任务管理帮助。

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Tauri](https://img.shields.io/badge/Tauri-v2-blue.svg)
![React](https://img.shields.io/badge/React-19-blue.svg)
![TypeScript](https://img.shields.io/badge/TypeScript-5-blue.svg)

## ✨ 特性

- 🐱 **桌面宠物** - 透明无边框窗口，始终置顶的可爱宠物
- 🎯 **番茄钟** - 25+5+15 专注模式（可自定义）
- 🎮 **成就系统** - 专注统计、里程碑解锁、连续天数追踪
- 👀 **分心检测** - 自动监听前台窗口，抓包摸鱼行为
- 🤖 **AI 助理** - Claude Code CLI 驱动的智能聊天、任务管理
- 🎨 **Spine 动画** - 流畅的骨骼动画支持

## 📸 截图

*coming soon...*

## 🚀 快速开始

### 环境要求

- [Node.js](https://nodejs.org/) >= 18
- [pnpm](https://pnpm.io/) >= 8
- [Rust](https://www.rust-lang.org/) >= 1.70
- [Tauri CLI](https://tauri.app/) >= 2

### 安装

```bash
# 克隆仓库
git clone https://github.com/Yaemikoreal/toumato.git
cd toumato

# 安装前端依赖
pnpm install

# 安装 Tauri CLI (如果尚未安装)
pnpm add -g @tauri-apps/cli
```

### 开发

```bash
# 启动开发服务器
pnpm tauri dev
```

### 构建

```bash
# 构建生产版本
pnpm tauri build
```

## 🎮 使用说明

### 基本操作

| 操作 | 说明 |
|------|------|
| **拖动** | 鼠标左键按住宠物拖动 |
| **聊天** | 点击宠物弹出聊天气泡，或点击右上角 💬 打开侧边栏 |
| **计时** | 右下角状态面板控制专注/休息计时 |
| **任务** | 侧边栏任务标签管理待办事项 |

## 📁 项目结构

```
toumato/
├── src-tauri/          # Rust 后端 (Tauri)
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs      # IPC 命令 + 事件监听 + 系统托盘
│   │   ├── timer.rs    # 番茄钟引擎 (Focus/ShortBreak/LongBreak)
│   │   ├── monitor.rs  # 进程监听 (Windows API)
│   │   ├── db.rs       # SQLite 操作 (7 表)
│   │   ├── pet.rs      # 宠物情绪状态机 (6 种情绪)
│   │   ├── ai.rs       # Claude CLI AI 接口
│   │   ├── task.rs     # 任务管理器
│   │   └── notification.rs  # Windows 通知
│   └── tauri.conf.json
├── src/                # React 前端
│   ├── components/     # SpinePet, StatusPanel, Sidebar, ChatPanel, TaskPanel...
│   ├── hooks/          # useTimer, usePet, useChat, useTasks, usePomodoro...
│   ├── App.tsx
│   └── main.tsx
├── public/
│   └── spines/         # Spine 动画资源
└── 素材/               # 桌面宠物素材
```

## 🛠️ 技术栈

| 层 | 技术 |
|---|---|
| 桌面框架 | Tauri v2 (Rust) |
| 前端 | React 19 + TypeScript + TailwindCSS + Framer Motion |
| 动画 | PixiJS (pixi.js-legacy) + pixi-spine 4.1 |
| 本地存储 | SQLite (rusqlite) |
| AI 接入 | Claude Code CLI 子进程通信 |

## 📖 开发文档

- [设计方案](设计方案.md) - 详细的设计文档
- [CLAUDE.md](CLAUDE.md) - Claude Code 开发指南
- [故障排除](docs/troubleshooting.md) - 常见问题解决方案

## 🤝 贡献

欢迎贡献！请查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解详情。

## 📝 许可证

本项目基于 [MIT License](LICENSE) 开源。

## 🙏 致谢

- [Tauri](https://tauri.app/) - 优秀的跨平台桌面框架
- [PixiJS](https://pixijs.com/) - 强大的 2D 渲染引擎
- [Spine](http://esotericsoftware.com/) - 专业的骨骼动画工具
