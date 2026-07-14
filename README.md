# Pomodachi — 流萤 LLM 桌宠

一个基于 Tauri + Vue 3 + Spine 的桌面宠物，可连接远程 LLM，并兼具任务提醒功能。

## 功能

- 🎨 使用 Spine 4.1 渲染流萤模型
- 💬 接入 OpenAI 兼容协议的远程 LLM，流萤人设对话
- 📌 自然语言创建任务 / 手动任务管理
- ⏰ 到点提醒，角色以情绪和语言提示
- 🖱️ 点击角色不同部位触发互动动画
- 🪟 无边框透明窗口、置顶、系统托盘

## 项目结构

```
Pomodachi/
├── src/               # Vue 3 前端
├── src-tauri/         # Rust 后端（Tauri）
├── public/avatar/     # 用户自备 Spine 模型（gitignored）
├── public/spine-player/ # Spine Player 运行时
├── PLAN.md            # 开发方案
└── CONTEXT.md         # 领域术语
```

## 准备素材

应用本身不携带版权素材。请将解包后的流萤模型文件放到 `public/avatar/`：

```bash
# 从 img/firefly 复制（或你自己的 Spine 模型目录）
cp img/firefly/skeleton_0      public/avatar/skeleton.skel
cp img/firefly/atlases_0_atlas_0 public/avatar/skeleton.atlas
cp img/firefly/atlases_0_textures_0_0.png public/avatar/UI_Spine_Avatar_1301.png
cp img/firefly/model0.json     public/avatar/model0.json
```

## 开发运行

```bash
npm install
npm run tauri dev
```

首次运行 Tauri 会自动编译 Rust 后端，需要网络下载 crates（可能较慢，建议配置 Cargo 国内镜像）。

## Windows 安装包

当前版本安装包已生成：

- `Pomodachi_0.1.0_x64-setup.exe`（项目根目录，约 3.6MB）
- 或从 `src-tauri/target/release/bundle/nsis/Pomodachi_0.1.0_x64-setup.exe` 获取

双击安装即可。安装包已内置前端资源，运行时需要系统已安装 WebView2（Windows 10/11 通常已自带）。

如需自行构建：

```bash
npm run tauri build
```

产物在 `src-tauri/target/release/bundle/`。

## 配置 LLM

双击角色打开聊天后，点击左上角「设置」，填入：

- **API Base**：如 `https://api.openai.com/v1`
- **Model**：如 `gpt-4o-mini`
- **API Key**：你的 OpenAI 兼容 API 密钥

密钥会存储在系统钥匙串中，不会写入配置文件。

## 交互说明

- **拖动角色**：按住角色空白处拖动窗口。
- **打开聊天**：双击角色，或点击「聊天」按钮。
- **创建任务**：在聊天中输入“提醒我下午三点开会”等自然语言。
- **任务面板**：点击「任务」按钮查看、完成、删除或延后任务。

## 开源声明

本项目开源，但不包含任何第三方美术资源。用户需自行提供合法的 Spine 模型。
