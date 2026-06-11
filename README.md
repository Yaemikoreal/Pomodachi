# 🍅 番茄宠物 (Toumato)

基于 Tauri v2 的本地优先、游戏化 AI 番茄钟桌面宠物。

> 把专注时间具象化为宠物的生命值和成长。专注时宠物开心成长，摸鱼时宠物受伤生气。

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Tauri](https://img.shields.io/badge/Tauri-v2-blue.svg)
![React](https://img.shields.io/badge/React-19-blue.svg)
![TypeScript](https://img.shields.io/badge/TypeScript-5-blue.svg)

## ✨ 特性

- 🐱 **桌面宠物** - 透明无边框窗口，始终置顶的可爱宠物
- 🎯 **番茄钟** - 25+5+15 专注模式（可自定义）
- 🎮 **游戏化** - 宠物成长系统、EXP、金币、HP
- 👀 **分心检测** - 自动监听前台窗口，抓包摸鱼行为
- 🤖 **AI 灵魂** - 情绪化反馈、每日战报、宠物聊天
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
| **缩放** | 滚轮上下滚动缩放 |
| **快捷键** | `Ctrl + +` 放大 / `Ctrl + -` 缩小 / `Ctrl + 0` 重置 |

### 缩放级别

`50% → 75% → 100% → 125% → 150% → 200%`

## 📁 项目结构

```
toumato/
├── src-tauri/          # Rust 后端 (Tauri)
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── timer.rs    # 番茄钟引擎 (规划中)
│   │   ├── monitor.rs  # 进程监听 (规划中)
│   │   ├── db.rs       # SQLite 操作 (规划中)
│   │   └── ai.rs       # AI 接口 (规划中)
│   └── tauri.conf.json
├── src/                # React 前端
│   ├── components/
│   │   └── SpinePet.tsx
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
| 前端 | React 19 + TypeScript + TailwindCSS |
| 动画 | PixiJS + pixi-spine |
| 本地存储 | SQLite (规划中) |
| AI 接入 | OpenAI 兼容格式 (规划中) |

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
