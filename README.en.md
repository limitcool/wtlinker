# WTLinker 🚀

**English** | [**简体中文**](./README.md)

WTLinker is a geek-oriented Windows Terminal tab distribution and management tool. It allows you to quickly open AI assistants (such as Codex, Claude, OpenCode) with one click inside a specified Windows Terminal window (supports appending to existing sessions as new tabs). Supporting seamless session resume and command flag isolation, WTLinker is the ultimate productivity tool for concurrent multi-project development.

---

## 💎 Key Features and Highlights

### 1. Modal-Based Project Configurations (Zero Layout Compression & Scrollbars)
Hovering over a project card in the left list reveals a **⚙️ Edit** icon. Clicking it opens a dark-themed, semi-transparent modal dialog where you can change the project's alias, bind local working directories, select project-specific AI runtimes, and edit session parameters. By moving the editor form into a modal, the right panel is kept clean, completely eliminating vertical scrollbars.

### 2. Dual-Line Combobox for Session ID Searches
When resuming a Codex session, the session ID text field and select dropdown are integrated into a single, cohesive Combobox. Focusing on the input opens a floating session list where each item is formatted in a dual-line layout:
- **First Line**: Shows the session ID abbreviation and its last modified timestamp.
- **Second Line**: Shows a 💬 user request preview (automatically extracted from the session log by stripping heavy system prompt declarations), making it incredibly easy to locate the exact session you want to resume.
- Features a click-outside detection overlay to automatically close the dropdown.

### 3. 📖 Chat-Like Conversation History Viewer (Modal-over-Modal)
Embedded right next to the session input box is a **📖 button** that triggers a top-level overlay displaying the detailed chat logs.
- Renders messages in clean bubbles (USER messages aligned to the right in dark slate blue, ASSISTANT messages to the left in dark grey).
- The Rust backend pre-processes the log, stripping system-level instructions and aborted turns to display only your clean conversation trajectory.

### 4. Interactive Windows Terminal Tab Previews
Under the "Append Mode", WTLinker retrieves and renders the active tab structure of your target terminal window, laying them out in a realistic Windows Terminal style. Features an easy `▼ Collapse / ▶ Expand` toggle to optimize screen estate.

### 5. Global Config Cascading & Local Overrides
Changing the default AI runtime in the global settings propagates to all projects in the list, while still allowing individual projects to be overridden locally via the project modal dialog.

### 6. Command Flag Isolation
WTLinker intelligently checks the AI runtime of your projects. It appends global sandboxing flags (e.g., `--dangerously-bypass-approvals-and-sandbox`) only when launching `Codex` projects. For `Claude` and `OpenCode` projects, these flags are filtered out to prevent terminal startup crashes.

---

## 🛠️ Technology Stack

- **Frontend**: React 18 + TypeScript + Vite + Tailwind CSS
- **Backend**: Tauri v2 + Rust
- **OS Integration**: Windows EnumWindows, Win32 API, and UI Automation (orchestrated asynchronously via a PowerShell scripting interface) to capture terminal window handles and bring them to the foreground.

---

## 📂 Configuration Storage Path

All WTLinker project profiles and preferences are saved locally in a YAML file, allowing direct manual edits using any text editor:

- **File Path**: `C:\Users\<YourUsername>\AppData\Roaming\wtlinker\config.yaml`
- **Quick Shortcut**:
  1. Press `Win + R` on Windows to open the Run window.
  2. Type **`%APPDATA%\wtlinker`** and hit Enter to jump directly to the config directory.

---

## 🚀 Quick Start for Developers

### 1. Prerequisites
Make sure you have installed:
- **Node.js** (v18+ recommended)
- **Rust** compiler and cargo chain
- **Windows Terminal** (capable of launching via `wt.exe`)

### 2. Install Dependencies
```bash
npm install
# or using pnpm
pnpm install
```

### 3. Run Dev Server (Debug Mode)
```bash
npm run tauri dev
```

### 4. Build Release Bundle
```bash
npm run tauri build
```
The installer package can be found at: `src-tauri/target/release/bundle/msi/wtlinker_x.x.x_x64_zh-CN.msi`
