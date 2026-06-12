# Phase 3 问题讨论与需求规划

> 日期：2026-06-12
> 状态：待实施

---

## 问题清单

### 1. 🐛 Bug：角色表情不变（一直生气）

**现象**：宠物动画始终显示生气表情，不随状态变化。

**根因分析**：

mood 只在特定事件触发时改变：
- `on_distraction()` → Angry/Sad（进程监听检测到分心时）
- `on_focus_complete()` → Happy（完成专注时）
- `on_focus_start()` → Focused
- `on_rest()` → Sleeping

问题很可能是进程监听器（`monitor.rs`）持续触发分心事件，每次 `-5 HP` 并设为 Angry。即使没有在"摸鱼"，某些后台进程可能被误匹配到黑名单。

**修复方案**：
- [ ] 增加分心事件的防抖/冷却机制（30 秒内只触发一次）
- [ ] 检查黑名单匹配是否过于宽泛
- [ ] 前端确认 `petStatus.mood` 值是否真的在变化
- [ ] 验证 Spine 动画名是否与 `.atlas` 文件中的名称匹配

**优先级**：P0
**预计工作量**：30 分钟

---

### 2. 🐛 Bug：没有关闭按钮

**现象**：窗口配置了 `decorations: false`（无标题栏）和 `skipTaskbar: true`（不在任务栏显示），导致没有任何关闭入口。

**解决方案**：与第 4 点（系统托盘）合并解决。通过系统托盘右键菜单提供「退出」选项，窗口右上角 × 按钮点击后隐藏到托盘而非退出。

**优先级**：P1（与第 4 点合并）
**预计工作量**：包含在系统托盘任务中

---

### 3. 📢 需求：Windows 原生通知

**场景**：
- 番茄钟完成 → 通知「专注完成！宠物很开心 🐱」
- 宠物 HP < 20 → 通知「宠物快没血了，快去喂食！」
- 长时间未专注 → 通知「宠物想你了～」

**技术方案**：使用 `tauri-plugin-notification` 插件，调用 Windows 原生通知系统。

**优先级**：P2
**预计工作量**：30 分钟

---

### 4. 🖥️ 需求：系统托盘图标

**现象**：应用运行时在右下角没有常驻图标，用户感知不到应用存在。

**需求描述**：类似输入法、QQ 等应用，右下角菜单栏常驻图标。点击图标可以看到基础信息，右键可以选择设置。

**技术方案**：使用 Tauri v2 内置的 `tauri::tray` 模块。

**功能设计**：
- 创建系统托盘图标（应用图标常驻右下角）
- 左键点击 → 显示/隐藏宠物窗口
- 右键菜单：
  - 「显示宠物」
  - 「今日专注：X 个」（只读展示）
  - 「设置」（打开侧边栏）
  - 「退出」
- 窗口右上角 × 按钮 → 隐藏到托盘（不退出）

**优先级**：P1
**预计工作量**：1-2 小时

---

### 5. 🤖 需求：AI 对接 OpenClaw

**背景**：当前自建的 `ai.rs` 模块直接调用 OpenAI 兼容 API，用户希望改为对接 OpenClaw，不作为单独的自主开发模块。

**OpenClaw 简介**：
- 开源个人 AI 助手平台（[GitHub](https://github.com/openclaw/openclaw)）
- 本地优先架构，Gateway 守护进程运行在本地
- 支持多模型（OpenAI、Ollama、通义千问等）
- 统一管理 API Key、模型路由、会话管理
- 默认端口：18789

**集成方案**：
- OpenClaw Gateway 运行在 `localhost:18789`
- toumato 的 Rust 后端通过 HTTP 请求调用 Gateway 的 agent API
- 不再需要用户单独配置 API Key / Model / Endpoint —— OpenClaw 统一管理
- `ai.rs` 简化为轻量的 OpenClaw 客户端

**优势**：
- 用户只需启动 OpenClaw，toumato 自动连接
- 支持 OpenClaw 已配置的任意模型
- 会话管理由 OpenClaw 处理
- 后续可利用 OpenClaw 的 Skills 扩展能力

**待确认**：
- [ ] 用户本地是否已部署 OpenClaw
- [ ] Gateway agent API 的具体 endpoint 和请求格式
- [ ] 是否保留直连 API 作为 fallback（OpenClaw 未启动时）

**优先级**：P3
**预计工作量**：1 小时

---

## 实施计划

| 阶段 | 任务 | 优先级 | 预计工作量 | 状态 |
|------|------|--------|-----------|------|
| Phase 3a | 修复表情 bug（防抖 + 黑名单排查） | P0 | 30 分钟 | ⏳ 待开始 |
| Phase 3b | 系统托盘 + 右键菜单 + 关闭按钮 | P1 | 1-2 小时 | ⏳ 待开始 |
| Phase 3c | Windows 原生通知 | P2 | 30 分钟 | ⏳ 待开始 |
| Phase 3d | AI 对接 OpenClaw | P3 | 1 小时 | ⏳ 待确认 |

---

## 技术依赖

### 新增 Cargo 依赖（预计）

```toml
# 系统托盘（Tauri v2 内置，无需额外依赖）

# Windows 通知
tauri-plugin-notification = "2"

# HTTP 请求（已有）
reqwest = { version = "0.12", features = ["json"] }
```

### Tauri 插件

```rust
// lib.rs 中注册通知插件
.plugin(tauri_plugin_notification::init())
```

---

## 相关文件

| 文件 | 涉及问题 |
|------|---------|
| `src-tauri/src/pet.rs` | #1 表情 bug |
| `src-tauri/src/monitor.rs` | #1 分心检测逻辑 |
| `src-tauri/src/lib.rs` | #4 托盘初始化、#5 AI 命令 |
| `src-tauri/src/ai.rs` | #5 OpenClaw 对接 |
| `src-tauri/tauri.conf.json` | #2 窗口配置 |
| `src/components/SpinePet.tsx` | #1 动画切换逻辑 |
| `src/components/Sidebar.tsx` | #4 设置面板 |
