# Fix clipboard hover preview stability and geometry

## Goal

让剪贴板列表的悬停预览在指针停留于同一 item 内时保持稳定，并让连接线和预览面板始终以 item 的几何中线为锚点。首次打开图片预览时，面板必须直接使用完整内容的正确尺寸，不依赖用户再次移动鼠标触发重测。

## Background

- 当前 `handleItemPointerMove` 会在同一 item 内按动画帧持续调用 `showClipboardPreview`，造成预览状态和 overlay 反复刷新。证据：`src/pages/Clipboard/hooks/useClipboardPreviewController.ts:199`。
- `pointerY` 会从列表传给 Rust，并把完整 item 的 source rect 压缩为指针位置的 1px 锚点，因此连接线不是 item 垂直中点。证据：`src/pages/Clipboard/hooks/useClipboardPreviewController.ts:415`、`src-tauri/src/window/preview.rs:657`。
- 预览页已有四边中点连接候选，但被压缩后的 source rect 破坏了该几何语义。证据：`src/pages/Preview/geometry.ts:83`。
- 隐藏测量层是条件渲染；`useMeasuredPanelSize` 只在 hook 初次挂载时尝试读取并监听节点，节点稍后出现时不会重新绑定 `ResizeObserver`。证据：`src/pages/Preview/index.tsx:180`、`src/pages/Preview/measurement.ts:20`。
- 图片内容加载前的空内容测量可能被误判为最终尺寸，使首个图片面板从错误尺寸以弹簧动画扩展。证据：`src/pages/Preview/index.tsx:78`、`src/pages/Preview/motion.ts:67`。

## Requirements

### R1 - 同 item 内指针移动不刷新预览

- hover 预览打开后，指针只要仍在当前 item 内移动，就不得再次调用预览命令、更新预览状态或重新计算锚点。
- 指针进入不同 item 时仍应正常切换预览目标。
- 指针离开 item / 列表时继续沿用现有隐藏缓冲与取消逻辑。

### R2 - 连接线使用 item 几何中线

- 列表侧锚点必须固定为当前 item 对应连接边的垂直中点，不再使用指针 Y 坐标。
- 预览面板位于 item 右侧时，圆点显示在 item 右边框；位于左侧时，圆点显示在 item 左边框。
- 若窗口空间迫使面板位于下方或上方，继续使用相应水平边的中点。
- 契约中不再传输或消费指针 Y 坐标。

### R3 - 首次图片预览尺寸正确

- 隐藏测量层在预览内容挂载、卸载或内容尺寸变化后都必须持续同步真实尺寸。
- 首个图片内容就绪后，面板应直接采用完整图片的适配尺寸；图片在 480px 高度上限内等比缩放并完整显示，不得被面板裁切。
- 用户无需移动鼠标即可获得正确尺寸。
- 只有预览内容已经就绪后的尺寸才可结束首次布局的 jump-only 阶段；避免空加载态尺寸触发后续错误弹簧过渡。

## Acceptance Criteria

- [x] 预览已打开时，在同一 item 内上下或左右移动鼠标不会重复调用 `showClipboardPreview`，预览框和连线保持稳定。
- [x] 从当前 item 移入相邻 item 时，预览内容、面板和连线仍能正常切换。
- [x] 预览框在右侧和左侧两种布局下，列表侧圆点分别落在 item 右边框和左边框的垂直中点。
- [x] 首次打开任意图片 item 时，图片在首次可见的最终布局中即完整等比显示；不需要移动鼠标修复高度。
- [x] 文本、文件、单图片文件预览仍能正常显示，滚动预览内容不受影响。
- [x] `pnpm lint`、`pnpm tsc`、`cargo fmt --check`、`cargo clippy -- -D warnings`、`cargo test` 通过。
- [x] 在 Windows 桌面实际验证同 item 悬停、跨 item 切换、左右侧连接线以及首次图片预览主路径。
- [x] 生成 Windows x64 NSIS 安装包。
- [x] 发布 GitHub Release `v1.5.3`，包含 macOS 与 Windows 构建资产。

## Delivery

- Local unsigned package: `src-tauri/target/release/bundle/nsis/EcoPaste_1.5.3_x64-setup.exe`
- Local SHA-256: `c3672c3162e0fbd626be4c21e31f42ca5e0b24fb6608fcf67dbfc6276f2930aa`
- GitHub release: `https://github.com/FeatherBirth/EcoPaste/releases/tag/v1.5.3`
- Release assets: Windows x64/ARM64 installers, macOS Intel/Apple Silicon DMGs and updater archives, checksum, and one-command installer.

## Out of Scope

- 不修改预览面板 480px 的最大宽高、hover 延迟、隐藏缓冲时间或内容缓存上限。
- 不新增用户设置项。
- 不重构预览内容渲染器或剪贴板数据结构。
