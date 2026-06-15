# WTLinker 🚀

[**English**](./README.en.md) | **简体中文**

WTLinker 是一款专为极客开发者打造的 Windows Terminal 标签页分发与管理工具。它能够一键在指定的 Windows Terminal 窗口（支持追加新建标签页模式）中快速拉起您的 AI 助手（如 Codex、Claude、OpenCode），支持无缝的会话恢复（Resume）与参数隔离分发，是多项目并发开发时的效率大杀器。

---

## 💎 核心亮点与特色

### 1. 弹窗式项目配置（拒绝界面拥挤与滚动）
左侧项目卡片在鼠标悬停时会自动浮现 **⚙️ 编辑** 按钮。点击即可拉起半透明遮罩的极简暗黑风 Modal 弹窗，集中进行项目的局部别名修改、重新绑定目录、AI 选型和会话恢复。主面板右侧仅保留全局配置和终端追加选项，彻底消灭了界面纵向滚动条，各组件自适应完美渲染。

### 2. 双行 Combobox 会话 ID 检索
在恢复 Codex 会话时，将会话 ID 输入框与下拉框完美融合为一体。点击即可在下方绝对定位展开历史会话列表，每个下拉项均支持双行排版：
- **第一行**：高对比度的会话 ID 缩略与修改时间。
- **第二行**：💬 展现后端提取出的**第一句用户真实 Goal 提问**（自动过滤 instructions 系统提示词及冗余模板），方便您一目了然根据开发诉求挑选要恢复的会话。
- 引入了点击外部自动关闭浮层的 Overlay 遮罩。

### 3. 📖 气泡式历史对话查看器（弹窗叠弹窗）
在会话输入框右侧内嵌了 **📖 按钮**，点击即可在顶层拉起详细会话历史记录弹窗。
- 对话采用经典的左右气泡流排版（USER 靠右呈深灰蓝气泡，ASSISTANT 靠左呈深黑气泡）。
- 后端 Rust 命令已对日志行进行深度净化，完美剥离了庞大的系统级说明包，仅保留您真实的对话轨迹，让您在 Resume 前对日志上下文一清二楚。

### 4. 仿真终端标签页折叠预览
支持在“追加模式”下，读取并渲染指定目标窗口的活动标签页布局，并以 Windows Terminal 仿真标签的样式在右下角横向平铺。支持一键 `▼ 折叠 / ▶ 展开`，为其他表单项留出黄金高度。

### 5. 全局全选覆盖与单独局部覆盖
在全局配置修改默认 AI 时，可同步覆盖所有已导入项目的 AI 程序选型；且依然支持在单个项目的编辑弹窗中，针对独立项目进行个性的局部覆盖。

### 6. 参数隔离分发
智能识别项目 AI 选型，仅当 AI 程序为 `Codex` 时，启动时才会在命令中追加全局附加参数（如 `--dangerously-bypass-approvals-and-sandbox`）；对于 `Claude` 或 `OpenCode` 项目则自动过滤此参数，避免命令报错。

---

## 🛠️ 技术架构

- **前端核心**：React 18 + TypeScript + Vite + Tailwind CSS
- **后端核心**：Tauri v2 + Rust
- **系统底层通信**：使用 Windows EnumWindows、Win32 API 和 UI Automation（通过封装的 PowerShell 异步交互）提取终端窗口信息及控制激活置顶。

---

## 📂 配置文件持久化路径

WTLinker 的所有数据和项目列表都持久化保存在本地 YAML 文件中，支持直接用任何文本编辑器编辑：

- **保存路径**：`C:\Users\<您的用户名>\AppData\Roaming\wtlinker\config.yaml`
- **快捷访问方法**：
  1. 按下快捷键 `Win + R` 打开 Windows 运行窗口。
  2. 输入 **`%APPDATA%\wtlinker`** 并点击回车，即可直达该文件夹目录。

---

## 🚀 开发者快速开始

### 1. 准备工作
请确保您的 Windows 系统上已安装：
- **Node.js** (推荐 v18+)
- **Rust** 编译链 (Cargo 编译器)
- **Windows Terminal** (支持 wt.exe 启动)

### 2. 安装依赖
```bash
npm install
# 或者使用 pnpm
pnpm install
```

### 3. 启动开发服务器（调试）
```bash
npm run tauri dev
```

### 4. 构建与发布 Release 包
```bash
npm run tauri build
```
编译生成的安装包路径为：`src-tauri/target/release/bundle/msi/wtlinker_x.x.x_x64_zh-CN.msi`
