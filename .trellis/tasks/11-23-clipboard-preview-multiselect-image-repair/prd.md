# 增强剪贴板预览、多选粘贴与图片修复

## Goal

修复系统级预览框在指针快速切走或经过无需预览的记录后残留的问题，并评估、增强长文本与图片预览的可交互能力；为历史记录增加 Windows 风格的修饰键多选与按换行批量粘贴；增加一个默认关闭的纯图片剪贴板格式自动修复开关，同时保持 Excel/WPS 表格复制原有的富文本与纯文本粘贴行为。

## Background

当前请求包含三个可独立验收的交付物，因此使用父任务管理需求来源、子任务映射和最终集成验收：

- [x] `11-23-clipboard-preview-lifecycle-interactions`：预览生命周期与预览框交互。
- [x] `11-23-clipboard-multiselect-batch-paste`：Ctrl/Shift 多选与批量粘贴。
- [x] `11-23-image-clipboard-format-repair`：可选纯图片格式修复。

仓库现状将作为实现基线：

- 预览窗口目前覆盖工作区并统一 `set_ignore_cursor_events(true)`，面板无法接收鼠标进入、滚轮和文本选择事件；证据：`src-tauri/src/window/preview.rs:370`、`src-tauri/src/window/preview.rs:377`。
- 预览隐藏主要依赖卡片 `pointerleave`、列表区域 `pointerleave`、document `pointerout` 与 240ms 缓冲；证据：`src/pages/Clipboard/hooks/useClipboardPreviewController.ts:175`、`src/pages/Clipboard/hooks/useClipboardPreviewController.ts:388`。
- 文本预览当前由 `react-virtuoso` 按软切行渲染，适合浏览但不适合跨行选取；证据：`src/pages/Preview/components/PreviewContent.tsx:96`、`src/pages/Preview/components/PreviewContent.tsx:118`。
- 列表目前只有一个 `selectedId`，左键按下会立即按 `autoPaste` 执行动作；证据：`src/pages/Clipboard/components/List.tsx:85`、`src/pages/Clipboard/components/List.tsx:972`。
- 列表查询 API 支持排序、筛选、分页，并在列表视图裁剪文本正文；批量粘贴需要 Rust 按当前过滤和顺序读取完整正文，不能依赖前端缓存。证据：`src-tauri/src/db/items.rs:15`、`src-tauri/src/db/items.rs:165`。
- Office/WPS 来源复制的“渲染图片”记录已经通过 `search_text` 保留纯文本备用表示，写回时默认写图片并追加文本格式，纯文本粘贴只写文本；证据：`src-tauri/src/clipboard/read.rs:36`、`src-tauri/src/clipboard/write.rs:37`、`src-tauri/src/clipboard/write.rs:117`。

## Requirements

### R1 - 预览生命周期与交互

- 指针离开当前预览记录、进入无需预览的记录或离开剪贴板窗口后，悬停预览必须在既有缓冲期内可靠关闭，不得残留到下一次预览触发才更新。
- 长文本预览在指针移入预览面板后保持打开，并允许滚动、选择与复制。
- 图片预览在指针移入面板后保持打开，滚轮可缩放；缩放范围和状态恢复方式由子任务设计确定。
- 不破坏键盘 Space 预览、左右/上下布局、连接线、窗口隐藏和退出动画。

### R2 - 多选与批量粘贴

- Ctrl+单击执行单条切换选择，Shift+单击执行与 Windows 文件列表一致的连续范围选择；Mac 使用 Command 作为平台主修饰键。
- 修饰键选择不触发单击粘贴/复制，也不打开预览目标切换之外的额外动作。
- 多选状态使用现有 token 颜色体系表达，不增加复选框。
- 多选后用双击或 Enter 将所选文本按当前列表顺序以换行分隔，写入一次剪贴板并执行一次系统粘贴。
- 提供一个默认关闭的实验性子开关，允许图文/文件混选逐条粘贴；全文本选择始终合并为一次粘贴。
- 普通无修饰键单击、双击、中键、右键菜单与数字快捷粘贴保持原行为。

### R3 - 纯图片格式修复

- 新增默认关闭的“自动修复截图后无法粘贴”类设置，并补齐中英文偏好文案。
- 自动修复仅在 Windows 提供和生效。
- 开启后，仅当监听事件被识别为纯图片信息时，将记录写入历史后检查并修复系统剪贴板格式。
- 修复不得改变纯图片记录的图片内容或历史去重语义。
- Excel/WPS 等带表格纯文本/富文本语义的图片来源，不得被识别为纯图片并强制改为图片；原有“纯文本工具粘文本、富文本工具粘图片”的格式组合必须保留。
- 设置关闭时不执行额外格式检测或重写。

## Acceptance Criteria

- [ ] 三个子任务的全部验收项通过，且父任务进行跨功能集成复核。
- [ ] 快速移动、快速切换前台应用、短文本跳过预览、离开列表或窗口时均不会残留旧预览。
- [ ] 文本预览可滚动、选择和复制；图片预览可滚轮缩放，移入面板时预览不消失。
- [ ] Ctrl/Shift 多选、连续选择、取消选择、清除选择、排序/筛选变化后的状态恢复符合 Windows 文件列表语义。
- [ ] 全文本批量粘贴只产生一次剪贴板写入和一次模拟粘贴，内容按当前列表顺序以换行分隔。
- [ ] 实验性混选开关默认关闭；开启后混合内容按顺序逐条粘贴，相邻文本可合并，失败时停止并提示。
- [ ] 自动修复开关默认关闭；关闭时无额外副作用，开启时只修复纯图片事件。
- [ ] 自动修复设置只在 Windows 出现，macOS 不执行对应的格式重写。
- [ ] Excel/WPS 单元格复制在普通粘贴、纯文本粘贴和粘贴后历史记录类型上保持现有行为。
- [ ] macOS 与 Windows 均完成对应编译/测试；涉及平台事件的实际交互完成可用环境验证。
- [ ] `pnpm lint`、`pnpm tsc`、`cargo fmt --check`、`cargo clippy -- -D warnings`、`cargo test` 通过。

## Out of Scope

- 不增加显式“选择模式”按钮或复选框。
- 不改变列表虚拟滚动、搜索、排序与分页架构。
- 不支持把多张图片或文件合并为单一剪贴板载荷；实验性混选仅按顺序逐条粘贴。
- 不改变 Excel/WPS 的既有格式判定和写回优先级。
