# Pomodachi 领域术语

一个围绕“流萤”Spine 资源构建、可接入 LLM 的独立桌面宠物应用。

## 核心概念

**桌面宠物（Desktop Pet）**：
常驻于桌面层、可与用户进行视觉与语言交互的虚拟角色应用。
_Avoid_: 桌宠程序、桌面精灵

**角色形象（Avatar）**：
呈现在屏幕上的可视化角色，当前特指“流萤”，由 Spine 骨骼动画模型驱动。
_Avoid_: 人物、模型（单独使用容易与数据模型混淆）

**资源包（Resource Pack）**：
桌宠应用使用的封装素材，例如仓库中的 `流萤桌宠.lpk`，格式为 `STD2_0`；解包后得到 Spine 标准文件。
_Avoid_: 皮肤、素材包

**Spine 运行时（Spine Runtime）**：
负责加载并渲染标准 Spine 模型（`.skel`、`.atlas`、纹理等）的 SDK 或引擎层。
_Avoid_: 播放器、查看器

**LLM 伙伴（LLM Companion）**：
通过大语言模型提供对话、陪伴与指令理解的智能体模块，控制角色的语言回复与部分行为触发。
_Avoid_: AI、聊天机器人（单独使用过于宽泛）

**应用壳（Application Shell）**：
承载 Spine 运行时与 LLM 伙伴的桌面程序框架，当前选定 Tauri（Rust 后端 + Web 前端）。
_Avoid_: 主程序、容器

**LLM 服务商（LLM Provider）**：
由用户配置的远程大语言模型 API，通过 OpenAI 兼容协议接入。
_Avoid_: AI 后端、模型接口

**任务（Task）**：
用户创建的待办事项，包含标题、截止时间、重复规则、完成状态以及期望提醒时呈现的情绪提示。
_Avoid_: 待办、备忘录

**提醒触发（Reminder Trigger）**：
到达任务截止时间时，由后端调度并通知前端，让角色以特定情绪和语言提醒用户的事件。
_Avoid_: 闹钟、通知事件

**通知表现（Notification Style）**：
提醒触发时角色通过情绪表情、语言气泡、可选音效和系统 toast 向用户传达信息的方式。
_Avoid_: 弹窗、消息框

**交互模式（Interaction Mode）**：
用户通过点击角色打开聊天气泡，以文字输入与 LLM 伙伴对话；同时可通过自然语言创建任务和提醒。
_Avoid_: 操作方式、输入方式

**意图解析（Intent Parsing）**：
LLM 伙伴把用户自然语言识别为具体动作（如创建任务、闲聊、查询状态）并输出结构化意图的过程。
_Avoid_: 命令识别、语义分析

**角色人设（Character Persona）**：
LLM 伙伴在对话中扮演的固定身份，当前为“流萤”，包含语言风格、对用户的称呼以及情绪表达方式。
_Avoid_: 角色设定、AI 人格

**系统提示词（System Prompt）**：
定义 LLM 伙伴角色、情绪标签协议、任务创建 JSON 格式及回复风格的全局指令。
_Avoid_: 提示词模板、Prompt

**情绪标签（Emotion Tag）**：
LLM 回复中用于指示角色应呈现何种表情/动作的标记，例如 `[开心]`、`[害羞]`，由前端映射到 Spine 动画名称。
_Avoid_: 表情指令、动作码

**窗口生命周期（Window Lifecycle）**：
桌面宠物窗口的显示、隐藏、置顶、点击穿透、系统托盘驻留及位置记忆等行为规则。
_Avoid_: 窗口管理、UI 状态

**设置（Settings）**：
用户可配置的偏好项，包括 LLM Provider、API 凭据、通知开关、开机自启、音量等。
_Avoid_: 配置、选项

**API 凭据（API Credential）**：
用户用于访问远程 LLM 服务的密钥，必须由应用安全地存储在本地钥匙串/加密库中。
_Avoid_: API key、密钥

**数据隐私策略（Privacy Policy）**：
用户对话、任务、API 凭据均本地存储，默认不上传任何遥测或诊断数据。
_Avoid_: 隐私协议

**角色状态机（Avatar State）**：
驱动角色在不同行为模式之间切换的规则集合，包括 Idle、Hover、Interact、Talking、Reminding、Dragging、Sleep 等状态。
_Avoid_: 动画状态、行为树

**资源使用授权（Asset License）**：
美术资源由用户自行提供，开源应用本身不内置、不分发任何受版权保护的素材。
_Avoid_: 素材版权、资源许可

**用户自备资源（User-Provided Asset）**：
用户将自己合法拥有的标准 Spine 模型（`.skel` + `.atlas` + 纹理）放入指定目录，由应用加载并运行。
_Avoid_: 自定义资源、外部模型

**角色配置（Avatar Config）**：
桌宠平台生成的 `model0.json`，描述点击区域、待机循环、动作分组及亲密系统等交互元数据，应用可读取它来决定用户点击不同部位时播放什么 Spine 动画。
_Avoid_: 模型配置、动作表

**命中检测（Hit Detection）**：
判断用户鼠标是否落在角色形象实际可交互区域内的计算过程，当前采用 Spine SkeletonBounds 在 CPU 侧完成，用于实现空白区域的点击穿透。
_Avoid_: 点击检测、碰撞检测

**渲染提示（Render Hints）**：
角色配置中声明的 Spine 渲染参数，例如是否使用预乘 alpha（PMA）、是否开启边缘填充（bleed）等，用于指导应用按资源导出方式正确渲染。
_Avoid_: 渲染参数、图形选项

**渲染修复（Render Fix）**：
当角色形象出现边缘白线、透明边框等渲染瑕疵时，应用提供的兜底机制，允许用户通过设置开关切换 PMA/非 PMA 或启用边缘补偿。
_Avoid_: 图像修复、显示修复

**目标平台（Target Platform）**：
应用首发支持的桌面操作系统，当前为 Windows，后续扩展至 macOS。
_Avoid_: 运行环境、系统支持

**错误兜底（Failure Mode）**：
当网络、LLM 服务或用户资源缺失时，应用以角色情绪、气泡提示和手动重试等方式优雅降级，而非抛出技术性错误弹窗。
_Avoid_: 异常处理、错误状态

---

*本文件随 grilling / domain-modeling 会话建立，后续如新增上下文可继续补充。*
