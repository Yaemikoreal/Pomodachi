# ADR-0002：Spine 渲染闪烁问题修复

## 状态

已接受

## 背景

在 test_pet 分支中，Spine 宠物出现渲染问题：
- 刚打开应用时能看到宠物动画
- 闪烁一瞬间后，宠物消失，只剩空白透明窗口
- 控制台显示 PixiJS 弃用警告（非根本原因）

## 问题分析

### 根本原因

`fitWindow` 函数中的 `app.renderer.resize()` 调用导致 PIXI WebGL 渲染上下文重置：

```typescript
// 问题代码
app.renderer.resize(w, h)  // ← 这行导致渲染上下文丢失
```

### 触发链

1. `fitWindow` 在 `requestAnimationFrame` 中被调用
2. 调用 `app.renderer.resize()` 调整 Canvas 尺寸
3. PIXI 的 WebGL 渲染上下文被重置
4. 所有已渲染的内容（包括 Spine 对象）被清除
5. 虽然 ticker 仍在运行，但 Spine 对象已被移除

### 次要原因

`fitWindow` 调整窗口尺寸可能触发 React 重新渲染，导致 `useEffect` 重新运行，进而调用 `destroy()` 销毁 PIXI 实例。

## 解决方案

### 1. 移除 resize 调用

```typescript
// 修复后
const fitWindow = useCallback(() => {
  // ... 其他代码 ...

  // 移除这行：
  // app.renderer.resize(w, h)

  // 只保留窗口尺寸调整
  getCurrentWindow().setSize(new LogicalSize(w, h))
}, [])
```

### 2. 增大初始 Canvas 尺寸

```typescript
// 修复后：使用较大初始尺寸避免 resize 需求
const INITIAL_SIZE = 400
const app = new PIXI.Application({
  width: INITIAL_SIZE,
  height: INITIAL_SIZE,
  // ... 其他配置
})
```

### 3. 移除 fitWindow 调用

```typescript
// 修复后：不调用 fitWindow，避免触发重新渲染
// 启动渲染循环
const tickCallback = () => {
  if (spineRef.current && appRef.current) {
    spineRef.current.update(0)
  }
}
app.ticker.add(tickCallback)

// 移除这行：
// requestAnimationFrame(() => fitWindow())
```

### 4. 保留 ticker 渲染循环

```typescript
// 关键：确保 Spine 动画持续更新
app.ticker.add(() => {
  if (spineRef.current) {
    spineRef.current.update(0)
  }
})
```

## 技术细节

### PIXI 渲染机制

- PIXI 使用 WebGL 渲染，渲染上下文是状态相关的
- `resize()` 调用会重置渲染上下文，清除所有已渲染内容
- ticker 负责持续更新和渲染，但前提是对象仍在 stage 中

### Tauri 窗口交互

- Tauri 窗口尺寸变化可能触发 React 重新渲染
- React 重新渲染会导致 useEffect 重新运行
- useEffect 清理函数会销毁 PIXI 实例

## 验证

修复后：
- ✅ 宠物持续显示，不再闪烁消失
- ✅ Spine 动画正常播放
- ✅ 窗口可以拖动
- ✅ 情绪切换正常

## 经验教训

1. **避免 resize**：尽量在初始化时设置正确的 Canvas 尺寸，避免运行时 resize
2. **分离关注点**：窗口尺寸调整和渲染上下文应该分离处理
3. **测试渲染稳定性**：在修改渲染相关代码时，需要测试长时间运行的稳定性

## 相关文件

- `src/components/SpinePet.tsx` — 主要修改文件
- `docs/adr/0001-keep-backend-skin-interface.md` — 相关决策
