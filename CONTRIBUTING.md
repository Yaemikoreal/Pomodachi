# 🤝 贡献指南

感谢你对番茄宠物 (Toumato) 项目的关注！我们欢迎各种形式的贡献。

## 📋 目录

- [行为准则](#行为准则)
- [如何贡献](#如何贡献)
- [开发流程](#开发流程)
- [提交规范](#提交规范)
- [Issue 指南](#issue-指南)
- [PR 指南](#pr-指南)
- [开发环境](#开发环境)

## 行为准则

本项目采用开放、友好的态度欢迎每一位贡献者。请保持尊重和建设性的交流。

## 如何贡献

### 贡献类型

1. **🐛 报告 Bug** - 提交 Issue 描述问题
2. **💡 功能建议** - 提出新功能或改进建议
3. **📝 文档改进** - 完善文档、修复错别字
4. **🎨 代码贡献** - 修复 Bug、实现新功能
5. **🎨 素材贡献** - 提供宠物皮肤、动画素材

### 不确定如何开始？

查看标记为 [`good first issue`](https://github.com/Yaemikoreal/toumato/labels/good%20first%20issue) 的 Issue，这些是适合新贡献者的任务。

## 开发流程

### 1. Fork & Clone

```bash
# Fork 仓库到你的 GitHub 账号
# 然后克隆你的 Fork
git clone https://github.com/<你的用户名>/toumato.git
cd toumato

# 添加上游仓库
git remote add upstream https://github.com/Yaemikoreal/toumato.git
```

### 2. 创建分支

```bash
# 同步上游
git fetch upstream
git checkout main
git merge upstream/main

# 创建功能分支
git checkout -b feature/你的功能名
# 或修复分支
git checkout -b fix/修复描述
```

### 3. 开发

```bash
# 安装依赖
pnpm install

# 启动开发服务器
pnpm tauri dev
```

### 4. 提交

```bash
git add .
git commit -m "[feat]：添加了某某功能"
```

### 5. 推送 & PR

```bash
git push origin feature/你的功能名
```

然后在 GitHub 上创建 Pull Request。

## 提交规范

使用中文提交，格式如下：

```
[类型]：精炼概要

- 变更点：（做了什么改动）
- 优化点：（改进了什么）
- 解决问题：（修复了什么）
```

### 类型说明

| 类型 | 说明 |
|------|------|
| `feat` | 新功能 |
| `fix` | 修复 Bug |
| `docs` | 文档更新 |
| `style` | 代码格式（不影响功能） |
| `refactor` | 重构 |
| `perf` | 性能优化 |
| `test` | 测试相关 |
| `chore` | 构建/工具链 |

### 示例

```
[feat]：添加宠物缩放功能

- 变更点：支持滚轮缩放和键盘快捷键
- 优化点：窗口大小随缩放自动调整
```

```
[fix]：修复拖动时偶尔失效的问题

- 解决问题：鼠标快速移动时拖动中断
```

## Issue 指南

### Bug 报告

使用 [Bug Report](https://github.com/Yaemikoreal/toumato/issues/new?template=bug_report.md) 模板，包含：

- 清晰的标题
- 复现步骤
- 期望行为 vs 实际行为
- 环境信息（OS、Node 版本、Rust 版本）
- 截图或日志（如有）

### 功能建议

使用 [Feature Request](https://github.com/Yaemikoreal/toumato/issues/new?template=feature_request.md) 模板，包含：

- 功能描述
- 使用场景
- 实现思路（可选）

## PR 指南

### PR 要求

1. **描述清晰** - 说明做了什么、为什么做
2. **关联 Issue** - 使用 `Closes #123` 关联相关 Issue
3. **测试通过** - 确保 `pnpm tauri dev` 正常运行
4. **代码风格** - 遵循项目现有风格
5. **单一职责** - 一个 PR 只做一件事

### PR 模板

```markdown
## 描述

简要描述这个 PR 做了什么

## 变更类型

- [ ] 新功能
- [ ] Bug 修复
- [ ] 文档更新
- [ ] 重构
- [ ] 其他

## 关联 Issue

Closes #<issue 编号>

## 测试

描述如何测试这些变更

## 截图（如适用）

添加相关截图
```

## 开发环境

### 必需工具

- [Node.js](https://nodejs.org/) >= 18
- [pnpm](https://pnpm.io/) >= 8
- [Rust](https://www.rust-lang.org/) >= 1.70
- [Tauri CLI](https://tauri.app/) >= 2

### 推荐工具

- [VS Code](https://code.visualstudio.com/)
  - [Tauri Extension](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
  - [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

### 项目结构

```
toumato/
├── src-tauri/      # Rust 后端
├── src/            # React 前端
├── public/         # 静态资源
└── 素材/           # 设计素材
```

## 问题？

如有任何问题，欢迎在 [Discussions](https://github.com/Yaemikoreal/toumato/discussions) 中提问！

---

再次感谢你的贡献！🎉
