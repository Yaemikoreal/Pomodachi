# ADR 0001: 桌宠点击穿透、尺寸缩放与边缘渲染修复

## 状态

已接受

## 背景

用户反馈桌面宠物 Pomodachi 存在以下问题：

1. 角色形象（Avatar）周围出现白线/透明边框，与桌面衔接不自然；
2. 设置面板、任务面板功能过于简陋；
3. 需要支持尺寸设置；
4. 空白区域应能点击到后面的桌面内容；
5. 想去掉透明边框线，只保留模型动画。

本 ADR 记录与角色形象边界、交互边界、渲染修复相关的架构决策。设置面板与任务面板的功能范围另行记录。

## 决策

### 1. 窗口保持矩形，启用不规则点击穿透

- **不采用** Windows 原生 `SetWindowRgn` 不规则窗口裁剪方案，避免破坏 Tauri 跨平台能力；
- **不采用** 前端捕获事件后模拟转发方案，因为 WebView 一旦拦截就无法真正点击到桌面；
- **采用** Tauri 窗口级 `set_ignore_cursor_events` 动态切换方案：窗口默认穿透，鼠标命中模型时恢复接收事件。

### 2. 命中检测采用 Spine SkeletonBounds CPU 方案

- **不采用** canvas `readPixels` 像素级检测，避免 GPU 回读性能开销；
- **不采用** 固定几何热区近似，避免动画变形时轮廓不准；
- **采用** Spine 运行时提供的 `SkeletonBounds` / mesh vertices 在 CPU 侧做命中检测，能自然跟随动画变形且开销极小。

### 3. 动态切换采用预切换 + 节流 + 状态差分

- 鼠标移动时以 100ms 节流做命中检测；
- 仅在"命中/未命中"状态真正变化时调用 Rust 命令切换 `ignore_cursor_events`；
- 鼠标按下并移动超过阈值时判定为拖拽，临时关闭点击穿透并启动 `start_dragging`。

### 4. 尺寸设置采用统一整体缩放比例

- **不单独**设置窗口大小或 Spine `scale`；
- **采用**单一 `scale_factor`（范围 `0.5x ~ 1.5x`，默认 `1.0x`），同时按比例调整 Tauri 窗口尺寸和 Spine `scale`。

### 5. 白线/透明边框采用"资源声明为主、设置兜底为辅"

- 在角色配置 `model0.json` 中新增 `render_hints` 字段，资源作者声明 `premultipliedAlpha`、`bleed` 等参数；
- 应用按声明加载；未声明时按当前默认 `premultipliedAlpha: true` 加载；
- 设置面板提供简单的"边缘修复"开关，在 PMA/非 PMA/bleed 补偿之间做兜底切换。

## 后果

### 正面

- 实现跨平台、低开销的不规则点击穿透；
- 尺寸调整视觉一致，空白区域不会突兀；
- 资源规范明确后，白线问题可根治，同时保留对旧资源/用户自备资源的兼容。

### 负面

- SkeletonBounds 命中区域比实际像素轮廓略"胖"，手臂等附件附近的空白处也可能被判定为命中；
- 需要在 Rust 侧新增 `set_ignore_cursor_events` 命令；
- 需要更新 `Avatar Config` 解析逻辑以支持 `render_hints`。

## 相关术语

- 命中检测（Hit Detection）
- 渲染提示（Render Hints）
- 渲染修复（Render Fix）
- 窗口生命周期（Window Lifecycle）
- 角色配置（Avatar Config）
