# Design

## Visual Theme
暗色极客风格 (Dark Tech/Geeky Theme)。背景基于极深蓝色 `#080a0f`，并搭配微妙的霓虹背景发光（靛蓝、紫色、暗金）。

## Color Palette
*   **Neutrals**:
    *   主背景: `bg-[#080a0f]`
    *   卡片背景: `bg-[#0f131c]/60` 与 `bg-[#121622]/30`
    *   边框色: `border-[#1b2233]`、`border-[#222a3d]`
    *   主文字色: `text-slate-100`、`text-slate-200`
    *   次要文字色: `text-slate-400`、`text-slate-500`
*   **Accents**:
    *   Codex / OpenAI 绿: `#10a37f`
    *   Claude / Anthropic 橙: `#d97757`
    *   OpenCode 琥珀黄: `#f59e0b`
    *   主色调 / 靛蓝: `indigo-500` (`#6366f1`)

## Typography
*   字体家族: 默认系统无衬线字体 (`font-sans`)，代码/路径/参数使用等宽字体 (`font-mono`)。
*   字体大小:
    *   标题: `text-lg font-bold`
    *   表单标签: `text-[10px] font-bold uppercase tracking-wider`
    *   普通正文: `text-xs`
    *   提示信息: `text-[11px]`、`text-[10px]`

## Layout
*   单屏卡片网格布局。在桌面端锁定高度不发生溢出。
*   左栏 (5/12 宽度) 放置项目列表，自带内部滚动；操作栏固定在左下角。
*   右栏 (7/12 宽度) 放置全局配置、项目编辑，以及最下方的启动按钮，按功能层次进行垂直收缩。
