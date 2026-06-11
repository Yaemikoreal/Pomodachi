# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

**Pomodoro Pet (番茄宠物)** — 基于 Tauri v2 的本地优先、游戏化 AI 番茄钟桌面宠物。

核心理念：把专注时间具象化为宠物的生命值和成长。专注时宠物开心成长，摸鱼时宠物受伤生气。AI 作为"宠物灵魂"提供拟人化情绪反馈。

## 技术栈

| 层 | 技术 |
|---|---|
| 桌面框架 | Tauri v2 (Rust) |
| 前端 | React + TypeScript + TailwindCSS + Framer Motion |
| 后端核心 | Rust |
| 本地存储 | SQLite (rusqlite 或 Tauri 插件) |
| AI 接入 | OpenAI 兼容格式 (本地 Ollama / 云端 API) |
| 进程监听 | sysinfo (Rust crate) |

## 架构

```
Frontend (React/TS)  ←IPC→  Backend (Rust)
  - 宠物动画                   - 番茄钟计时器 (Rust 驱动，防休眠被杀)
  - 状态面板                   - 窗口/进程监听 (sysinfo)
  - 设置页                     - 数据持久化 (SQLite)
                               - AI Interface (OpenAI 兼容)
```

前后端通过 Tauri IPC 通信。计时逻辑在 Rust 后端运行，前端仅负责渲染和交互。

## 核心模块

- **桌面宠物系统**: 无边框透明窗口，150x150px，始终置顶，点击穿透 (`set_ignore_cursor_events`)。宠物状态机: Idle → Focusing → Distraction → Resting
- **番茄钟引擎**: 25+5+15 模式（可自定义），Rust 后端驱动计时，专注完成结算 EXP/金币，失败扣 HP
- **分心检测器**: Rust 后台线程每 3 秒获取前台窗口进程名，匹配用户自定义黑名单
- **AI 灵魂**: 情绪化反馈（被抓包时吐槽、完成时夸奖）、每日专注战报、宠物聊天

## 数据模型 (SQLite)

四张表：`pet`（宠物状态，单行）、`pomodoro_record`（专注记录）、`blocklist`（分心黑名单）、`chat_history`（AI 对话历史）。

## 开发路线

1. **Phase 1 (MVP)**: Tauri + React 骨架、透明悬浮窗、点击穿透、像素猫动画、基础计时
2. **Phase 2 (游戏化)**: 计时移至 Rust、SQLite 存储、进程监听、状态联动
3. **Phase 3 (AI)**: 侧边栏面板、OpenAI API 接入、情绪反馈、Chat 界面
4. **Phase 4 (打磨)**: 托盘优化、多皮肤、打包发布

## 项目结构（规划）

```
toumato/
├── src-tauri/          # Rust 后端 (Tauri)
│   ├── src/
│   │   ├── main.rs
│   │   ├── timer.rs    # 番茄钟引擎
│   │   ├── monitor.rs  # 进程监听
│   │   ├── db.rs       # SQLite 操作
│   │   └── ai.rs       # AI 接口
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                # React 前端
│   ├── components/
│   ├── hooks/
│   └── App.tsx
├── public/
├── 素材/               # 桌面宠物素材（lpk 等格式）
├── package.json
├── tsconfig.json
├── tailwind.config.js
└── 设计方案.md
```

## 开发约定

- Tauri v2 项目用 `pnpm create tauri-app` 初始化
- Rust 后端遵循 Tauri 插件式模块划分，每个核心模块独立文件
- 前端组件按功能划分：宠物动画、状态面板、设置、聊天
- SQLite 数据库文件存于 Tauri app data 目录
- AI 配置（Base URL、API Key）通过设置页管理，存本地
- 像素动画素材放 `public/sprites/`，按宠物状态分子目录
