# CONTEXT.md — Toumato 术语表

## 核心概念

### 番茄宠物 (Toumato)
基于 Tauri v2 的本地优先 AI 番茄钟桌面宠物/个人助理。专注时间具象化为宠物情绪反馈。

### Spine 宠物 (Spine Pet)
使用 Spine 4.1 骨骼动画渲染的宠物。支持丰富的表情和动作动画。

### Agent Pet 兼容
参考 agent-pet 项目的设计，采用状态映射方式将情绪映射到 Spine 动画。

---

## 宠物状态

### 情绪 (Mood)
宠物的 8 种情绪状态，参考 agent-pet 的 messageMap 设计：
- **happy** — 开心（success/idle）→ Face_Happy
- **focused** — 专注（processing）→ Face_Star
- **tired** — 疲惫（waiting）→ Face_CloseEyesHappy
- **sleeping** — 睡眠（idle）→ Face_CloseEyesHappy
- **listening** — 聆听（new_message）→ Face_Happy
- **thinking** — 思考（review_required）→ Face_Star + Hand_Ponder
- **error** — 错误/分心（error）→ Face_Angry + Hand_Akimbo
- **waving** — 挥手打招呼（new_message）→ Face_Happy + Hand_Clap

### 皮肤 (Skin)
宠物的外观主题。当前使用 firefly 皮肤，保留皮肤切换接口。

---

## 技术术语

### Spine 渲染
使用 PixiJS + @pixi-spine/all-4.1 渲染 Spine 骨骼动画。需要 ticker 渲染循环持续更新。

### 渲染稳定性
PIXI 的 WebGL 渲染上下文是状态相关的，resize() 调用会重置上下文。应避免运行时 resize，尽量在初始化时设置正确的 Canvas 尺寸。

### Ticker 渲染循环
PIXI 使用 ticker 持续更新和渲染。Spine 动画需要在 ticker 中调用 update(0) 才能正常播放。

---

## 架构决策

### 后端保留接口 (Keep Backend Interface)
保留 skin_id 相关的 IPC 命令和数据库字段，但前端禁用皮肤选择器。确保向后兼容性，将来可恢复皮肤系统。

> 详细决策记录见 [ADR-0001](docs/adr/0001-keep-backend-skin-interface.md)

### Spine 渲染修复 (Spine Rendering Fix)
修复 Spine 宠物闪烁消失的问题。根本原因是 fitWindow 中的 resize 调用导致渲染上下文重置。

> 详细决策记录见 [ADR-0002](docs/adr/0002-spine-rendering-fix.md)
