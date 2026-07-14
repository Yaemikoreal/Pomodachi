# Pomodachi 开发方案

> 基于已澄清的需求和领域模型（见 `CONTEXT.md`）制定的可执行开发计划。

---

## 1. 核心决策

| 决策项 | 结论 |
|---|---|
| 美术格式 | 实际为 **Spine 4.1**（不是 Live2D），使用 `.skel` + `.atlas` + 纹理 |
| 应用形态 | 独立跨平台桌面应用，Windows 首发，后续 macOS |
| 应用框架 | **Tauri 2**（Rust 后端 + Web 前端） |
| 渲染引擎 | **Spine WebGL Runtime / spine-player 4.1** |
| 前端技术 | Vue 3 + TypeScript（团队若更熟悉 React 可替换） |
| LLM 接入 | 仅远程 API，OpenAI 兼容协议 |
| 本地数据 | SQLite（任务、提醒、对话历史） |
| API Key | 系统钥匙串 / Tauri Stronghold 加密存储 |
| 资源策略 | 开源应用不携带素材，由用户自备合法 Spine 模型 |

---

## 2. 项目结构

```
Pomodachi/
├── AGENTS.md
├── CONTEXT.md              # 领域术语
├── PLAN.md                 # 本文件
├── README.md               # 使用/构建说明
├── .gitignore              # 排除用户素材和构建产物
├── src-tauri/              # Rust 后端 + Tauri 配置
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       ├── commands/       # Tauri invoke 命令
│       │   ├── llm.rs      # LLM 调用
│       │   ├── task.rs     # 任务/提醒 CRUD
│       │   ├── settings.rs # 设置读写
│       │   └── window.rs   # 窗口控制
│       ├── db.rs           # SQLite 连接与迁移
│       ├── crypto.rs       # API Key 加密存储
│       ├── scheduler.rs    # 提醒调度器
│       └── models.rs       # 数据类型定义
├── src/                    # Web 前端
│   ├── main.ts
│   ├── App.vue
│   ├── components/
│   │   ├── PetWindow.vue       # 主窗口：Spine 渲染层
│   │   ├── ChatBubble.vue      # 聊天气泡
│   │   ├── TaskPanel.vue       # 任务列表面板
│   │   └── SettingsPanel.vue   # 设置面板
│   ├── spine/                  # Spine 运行时封装
│   │   ├── SpineRenderer.ts    # 加载模型、播放动画
│   │   ├── emotion-map.ts      # 情绪标签 → 动画名
│   │   └── avatar-config.ts    # 解析 model0.json
│   ├── llm/
│   │   ├── client.ts           # 调用 Rust LLM 命令
│   │   ├── system-prompt.ts    # 系统提示词
│   │   └── intent-parser.ts    # 解析 LLM 返回的意图 JSON
│   ├── stores/
│   │   ├── app-store.ts        # 全局状态
│   │   └── task-store.ts       # 任务状态
│   └── utils/
│       └── time.ts
├── resources/
│   └── avatar/             # 用户自备模型目录（gitignored）
│       ├── skeleton.skel
│       ├── skeleton.atlas
│       ├── UI_Spine_Avatar_1301.png
│       └── model0.json
└── demo/                   # 当前验证 demo（保留参考）
    ├── index.html
    └── public/firefly/     # gitignored
```

---

## 3. 数据模型

### 3.1 SQLite 表结构

```sql
-- 任务与提醒
CREATE TABLE tasks (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    title       TEXT NOT NULL,
    due_at      INTEGER,              -- Unix 毫秒
    repeat_rule TEXT,                 -- 'none' | 'daily' | 'weekly'
    emotion_hint TEXT DEFAULT '认真',  -- 提醒时使用的情绪/动画
    is_done     INTEGER DEFAULT 0,
    created_at  INTEGER NOT NULL
);

-- 对话历史（短期上下文）
CREATE TABLE messages (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    role        TEXT NOT NULL,        -- 'user' | 'assistant' | 'system'
    content     TEXT NOT NULL,
    emotion_tag TEXT,                 -- 助手消息的情绪标签
    created_at  INTEGER NOT NULL
);

-- 用户设置
CREATE TABLE settings (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL
);
```

### 3.2 Rust 数据结构（核心）

```rust
pub struct Task {
    pub id: i64,
    pub title: String,
    pub due_at: Option<i64>,
    pub repeat_rule: RepeatRule,
    pub emotion_hint: String,
    pub is_done: bool,
    pub created_at: i64,
}

pub enum RepeatRule { None, Daily, Weekly }

pub struct LlmMessage {
    pub role: String,
    pub content: String,
}

pub struct LlmResponse {
    pub content: String,
    pub emotion_tag: Option<String>,
    pub intent: Option<Intent>,
}

pub enum Intent {
    CreateTask { title: String, due_at: Option<i64> },
    Chat,
}
```

---

## 4. 架构设计

### 4.1 分层

```
┌─────────────────────────────────────────────┐
│  前端（Vue 3 + TypeScript + Spine Player）   │
│  - 渲染 Spine 模型、聊天、任务面板、设置      │
├─────────────────────────────────────────────┤
│  Tauri Invoke / Events                       │
├─────────────────────────────────────────────┤
│  后端（Rust）                                │
│  - LLM 调用      - 任务/提醒调度             │
│  - SQLite        - API Key 安全存储          │
│  - 窗口控制      - 系统托盘                  │
└─────────────────────────────────────────────┘
```

### 4.2 关键交互流程

#### 用户发送消息

1. 前端 `ChatBubble` 收集输入，调用 `invoke("send_message", { text })`。
2. Rust `llm::send_message` 读取系统提示词 + 最近 N 条历史，调用远程 LLM。
3. LLM 返回带情绪标签和意图的文本/JSON。
4. Rust 解析意图：
   - 若为 `create_task`，写入 `tasks` 表。
   - 否则作为普通回复。
5. Rust 返回 `{ content, emotion_tag, intent_result }`。
6. 前端显示气泡，并调用 `SpineRenderer.play(emotion_tag)` 播放对应动画。

#### 提醒触发

1. Rust `scheduler` 在启动时扫描未来 24 小时的未完成任务。
2. 使用异步定时器（如 `tokio::time`）在到期时发送 Tauri Event `reminder:trigger`。
3. 前端收到事件，弹出气泡、播放 `emotion_hint` 动画、播放音效。
4. 用户点击“完成”或“稍后提醒”，前端调用 Rust 命令更新任务状态或推迟提醒。

---

## 5. Spine 渲染层

### 5.1 模型加载

- 运行时从 `resources/avatar/` 读取：
  - `skeleton.skel`（骨骼二进制）
  - `skeleton.atlas`（图集）
  - `UI_Spine_Avatar_1301.png`（纹理）
  - `model0.json`（可选：交互配置）
- 使用 `spine-player` 或 `spine-webgl` 的 `AssetManager` 加载。
- 若文件缺失，显示占位提示。

### 5.2 动画映射

已确认的可用动画：

```typescript
export const EMOTION_ANIMATION_MAP: Record<string, string> = {
  开心: 'Face_Happy',
  害羞: 'Face_CloseEyesHappy',
  认真: 'Hand_Ponder',
  惊讶: 'Face_Star',
  委屈: 'Face_Doubt',
  生气: 'Face_Angry',
  困: 'Move_Sit_Idle',
};

export const IDLE_ANIMATIONS = [
  'Move_Sit_Idle',
  'Move_Stand_Idle',
];
```

- 收到情绪标签时，混合（mix）到对应动画。
- 无交互时循环播放 `Move_Sit_Idle`。
- 点击不同身体部位时，读取 `model0.json` 的 `hit_areas` 播放对应动作组。

### 5.3 点击区域

解析 `model0.json` 中的 `hit_areas`，得到类似：

```typescript
[
  { name: '头顶', motion: '摸头#1' },
  { name: '脸',   motion: '摸脸#1' },
  { name: '胸口', motion: '摸胸#1' },
  // ...
]
```

前端监听 Spine 渲染层的点击坐标，根据骨骼/插槽边界判断命中区域，触发动作组。

---

## 6. LLM 集成

### 6.1 系统提示词模板

```text
你是“流萤”，一位温柔、略带害羞的虚拟伙伴，称呼用户为“开拓者”。

规则：
1. 回复简短、可爱，适合在桌面宠物气泡中显示。
2. 在回复开头用情绪标签表达当前情绪，可选标签：[开心]、[害羞]、[认真]、[惊讶]、[委屈]、[生气]、[困]。
3. 如果用户要求创建提醒/任务，请严格返回如下 JSON，不要加 Markdown 代码块：
   {"intent":"create_task","title":"任务标题","due_at":"ISO8601 时间或 null","emotion_hint":"认真"}
4. 不要主动提及你是 AI。
```

### 6.2 调用参数

- 模型：用户自选（默认 `gpt-4o-mini` 或性价比高的 OpenAI 兼容模型）。
- 温度：`0.7`。
- 最大 token：`300`（桌面气泡不宜过长）。
- 上下文：最近 10 轮对话。

### 6.3 意图解析

Rust 端用正则/简单 JSON 解析提取 `intent`。

```rust
if let Ok(intent) = serde_json::from_str::<Intent>(&raw_content) {
    // 执行创建任务等操作
}
```

解析失败时按普通聊天处理。

---

## 7. 窗口与系统集成

### 7.1 Tauri 窗口配置

```json
{
  "windows": [
    {
      "label": "pet",
      "width": 400,
      "height": 400,
      "transparent": true,
      "decorations": false,
      "alwaysOnTop": true,
      "skipTaskbar": true,
      "resizable": false,
      "visible": true
    }
  ]
}
```

### 7.2 行为

- **点击穿透**：非交互状态下启用（`setIgnoreCursorEvents(true)`），不遮挡桌面操作。
- **点击角色**：禁用穿透，显示聊天气泡。
- **拖拽**：按住角色任意位置移动窗口。
- **系统托盘**：右键菜单“显示/隐藏”、“设置”、“退出”。
- **位置记忆**：关闭时保存窗口坐标，启动时恢复。
- **开机自启**：可选，写入注册表/登录项（二期）。

---

## 8. 安全与隐私

- **API Key**：使用 `tauri-plugin-stronghold` 或 `keyring-rs` 存储，不落盘明文。
- **对话/任务数据**：本地 SQLite，不上传。
- **遥测**：第一期不做，未来如需崩溃报告必须显式征得同意。
- **素材**：应用不内置任何版权素材，README 明确用户自备并自负版权责任。

---

## 9. 开发阶段

### Phase 1：MVP（目标：可对话的桌面宠物）

1. 初始化 Tauri + Vue 3 项目。
2. 集成 Spine Player 4.1，加载 `resources/avatar/` 下的模型。
3. 实现基础窗口行为：无边框、透明、置顶、穿透、托盘、拖拽。
4. 实现 LLM 远程调用和系统提示词。
5. 实现情绪标签 → Spine 动画映射。
6. 实现聊天气泡 UI。

### Phase 2：任务提醒

1. SQLite 任务/提醒表与 CRUD。
2. Rust 定时调度器。
3. 提醒触发时的角色表现（气泡 + 动画 + 音效）。
4. 自然语言创建任务（意图解析）。

### Phase 3：打磨与扩展

1. 解析 `model0.json` 点击区域，实现“摸头/摸脸”互动。
2. 对话历史摘要/长期记忆。
3. macOS 适配。
4. 语音输入/输出（可选）。
5. 系统通知兜底、开机自启。

---

## 10. 构建与运行

```bash
# 1. 安装依赖
cd Pomodachi
npm install

# 2. 放入用户自备模型（首次运行前）
mkdir -p resources/avatar
cp /path/to/firefly/* resources/avatar/
# 并重命名为：skeleton.skel / skeleton.atlas / UI_Spine_Avatar_1301.png

# 3. 开发运行
npm run tauri dev

# 4. 构建 Windows 安装包
npm run tauri build
```

---

## 11. 风险与应对

| 风险 | 应对 |
|---|---|
| Spine 模型版权 | 应用不携带素材，用户自备 |
| macOS 窗口层级限制 | 放到 Phase 3，单独测试 |
| LLM 返回格式不稳定 | JSON 失败兜底为普通聊天 |
| 远程 API 不可用 | 本地任务提醒仍可工作，优雅降级 |
| 用户未提供模型 | 显示占位角色和引导文案 |

---

## 12. 立即开始的第一项任务

**搭建 Tauri + Vue 3 骨架，并把当前的 `demo/` Spine 渲染搬进主窗口。**

这一步完成后，你就能看到一个漂浮在桌面上的流萤，为后续接入 LLM 和任务系统打下基础。
