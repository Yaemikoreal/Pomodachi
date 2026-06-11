# 番茄宠物 - 问题排查记录

本文档记录了项目搭建过程中遇到的问题及解决方案。

---

## 1. pnpm 安装依赖超时

**问题描述**：`pnpm install` 安装 Tauri CLI 等原生依赖时超时。

**原因**：默认 npm 源在国外，下载速度慢。

**解决方案**：切换到国内镜像源：

```bash
pnpm config set registry https://registry.npmmirror.com
```

---

## 2. esbuild 构建脚本未授权

**问题描述**：安装 `esbuild` 时提示 `esbuild's build scripts need to be approved`。

**原因**：pnpm 的安全策略阻止了未授权的构建脚本执行。

**解决方案**：手动批准构建脚本：

```bash
pnpm approve-builds esbuild
```

---

## 3. 端口 1420 被占用

**问题描述**：启动开发服务器时报错 `Port 1420 is already in use`。

**原因**：之前的开发服务器进程未完全退出。

**解决方案**：查找并杀掉占用端口的进程：

```bash
# 查找占用端口的进程
netstat -ano | grep 1420

# 杀掉进程（替换 PID）
taskkill //F //PID <进程ID>
```

---

## 4. WebGL 不支持（WebView2 环境）

**问题描述**：Tauri WebView2 中 PixiJS 报错 `Unable to auto-detect a suitable renderer` 或 `this browser does not support WebGL`。

**原因**：Tauri 使用的 WebView2 环境可能不支持 WebGL，而 PixiJS 默认使用 WebGL 渲染器。

**解决方案**：使用 `pixi.js-legacy` 替代 `pixi.js`，它包含 Canvas2D 渲染器：

```bash
pnpm add pixi.js-legacy
```

在代码中使用：

```typescript
import * as PIXI from 'pixi.js-legacy'

const app = new PIXI.Application({
  forceCanvas: true,  // 强制使用 Canvas2D
  // ...其他配置
})
```

---

## 5. Canvas 上下文冲突

**问题描述**：先用 `canvas.getContext('2d')` 绘制调试内容后，PixiJS 无法获取渲染上下文。

**原因**：同一个 Canvas 元素只能获取一种类型的上下文（2D 或 WebGL），调试代码先占用了 2D 上下文。

**解决方案**：让 PixiJS 自己创建 canvas，不要传入已有的 canvas 元素：

```typescript
// ❌ 错误：传入已有的 canvas
const app = new PIXI.Application({ view: existingCanvas })

// ✅ 正确：让 PixiJS 自己创建
const app = new PIXI.Application({ width: 300, height: 300 })
container.appendChild(app.view as HTMLCanvasElement)
```

---

## 6. Spine 插件未注册到 pixi.js-legacy

**问题描述**：使用 `pixi.js-legacy` 时，`PIXI.spine` 为 `undefined`。

**原因**：`pixi.js-legacy` 不会自动注册 Spine 插件。

**解决方案**：直接从 `@pixi-spine/all-4.1` 导入所需的类：

```typescript
import * as PIXI from 'pixi.js-legacy'
import { Spine, TextureAtlas, AtlasAttachmentLoader, SkeletonBinary } from '@pixi-spine/all-4.1'
```

---

## 7. 异步加载期间组件卸载导致崩溃

**问题描述**：Spine 资源异步加载期间，组件被卸载导致 `app.stage` 为 `null`，报错 `Cannot read properties of null (reading 'addChild')`。

**原因**：React 的 `useEffect` 清理函数会在组件卸载时执行，但异步加载可能还未完成。

**解决方案**：添加 `destroyed` 标志位，在每个异步操作后检查：

```typescript
useEffect(() => {
  let destroyed = false

  const init = async () => {
    const app = new PIXI.Application({ /* ... */ })

    // 每个异步操作后检查
    if (destroyed) {
      app.destroy(true)
      return
    }

    // 加载资源...
    const response = await fetch('/path/to/resource')

    if (destroyed || !app.stage) {
      app.destroy(true)
      return
    }

    // 添加到舞台
    if (!destroyed && app.stage) {
      app.stage.addChild(spine)
    }
  }

  init()

  return () => {
    destroyed = true
    if (appRef.current) {
      appRef.current.destroy(true)
    }
  }
}, [])
```

---

## 8. 浏览器自动请求 favicon.ico 导致 404

**问题描述**：控制台显示 `404 (Not Found)` 错误。

**原因**：浏览器会自动请求 `/favicon.ico`，但项目中没有该文件。

**解决方案**：在 `public/` 目录添加 favicon，并在 `index.html` 中引用：

```html
<link rel="icon" type="image/svg+xml" href="/favicon.svg" />
```

---

## 9. PixiJS Deprecation Warning（警告，非错误）

**问题描述**：控制台显示 `utils.rgb2hex is deprecated, use Color#toNumber instead`。

**原因**：`@pixi-spine/all-4.1` 内部使用了 PixiJS 的旧 API。

**影响**：这只是警告，不影响功能。需要等待 Spine 插件更新。

**临时方案**：可以忽略，或在开发环境中屏蔽弃用警告。

---

## 依赖版本参考

以下版本组合已验证可正常工作：

```json
{
  "dependencies": {
    "pixi.js-legacy": "^7.4.3",
    "@pixi-spine/all-4.1": "^4.0.6"
  }
}
```

**注意**：不要使用 `pixi.js`，必须使用 `pixi.js-legacy` 以支持 Canvas2D 渲染。
