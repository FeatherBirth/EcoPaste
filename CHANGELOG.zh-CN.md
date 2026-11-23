# 更新日志

## [1.6.0](https://github.com/FeatherBirth/EcoPaste/compare/v1.5.3...v1.6.0) (2026-11-23)

### ✨ 新功能

- 重做悬停预览生命周期：快速移动鼠标、切换应用、移到无需预览的空白区域或窗口失焦时，旧预览都会可靠关闭。
- 文本预览支持滚动、拖选、全选和复制；预览框高度随内容伸缩，上限与剪贴板列表高度一致，只有内容真正溢出时才显示滚动条。
- 图片预览支持滚轮、Ctrl + 滚轮和 Windows 精确触摸板捏合缩放；预览框与图片整体等比缩放，每次重新打开都会恢复默认尺寸和位置。
- 新增有序多选：按 Ctrl/Cmd 单击可追加或取消单项，按 Shift 单击可选择连续范围；已选条目显示粘贴顺序，取消选择后剩余序号自动重排。
- 新增批量粘贴，按选择顺序组合内容；实验性的混合粘贴只会在所选记录包含图片或文件时逐条执行，纯文本仍合并为一次粘贴。
- 新增 Windows“自动修复截图后无法粘贴”开关，默认关闭。该修复只处理纯图片剪贴板格式，不改变 Excel/WPS 表格原有行为：粘贴到纯文本工具仍为文本，粘贴到富文本工具仍可为图片。
- 自动更新改为检查当前 GitHub 仓库 Releases，并为 Windows x64/ARM64 与 macOS Intel/Apple Silicon 发布经过签名校验的更新产物。
- Windows 自动更新会遵循系统 HTTP 代理设置，避免 GitHub 可访问但应用内更新请求失败。
- 新增“跟随系统 / 直连 / 自定义”三种更新代理模式，并校验自定义代理地址。
- 在“关于”页面恢复当前 GitHub 仓库入口。
- 保留原图白色 P 造型并轻微更新背景质感，同时新增品牌化 NSIS 顶部横幅、侧栏图和安装器图标。

### 🐛 问题修复

- 修复 Windows 125%、150%、200% 缩放及跨显示器 DPI 场景下，左右预览连接点位置不一致或侵入列表条目的问题。
- 预览连接线、间距和面板尺寸统一使用 CSS 逻辑像素，只在窗口坐标映射时转换为物理像素，避免重复缩放。
- 多选时自动跳过不可用条目、已删除文件及未开启混合粘贴时不受支持的图片或文件；Shift 范围选择同样跳过。
- 修复文本预览首次打开时滚动条短暂闪现的问题，并在指针移入预览框时保持预览打开，便于完成文本选择。
- 移除旧更新服务器、测试版/每日构建更新入口、作者赞赏二维码及不再使用的应用外链入口。

### ⚠️ 升级说明

- 本分支重置了更新签名身份。请先手动安装一次 1.6.0；从后续版本开始，可以直接通过 GitHub Release 的签名更新清单位完成应用内自动更新。
- Windows 一键安装脚本继续可用，会覆盖升级现有机器级安装、保留用户数据，并在安装后启动应用。

## [1.5.3](https://github.com/FeatherBirth/EcoPaste/compare/v1.5.2...v1.5.3) (2026-11-11)

### 🐛 问题修复

- 修复鼠标在同一剪贴板项内移动时，悬停预览反复刷新和闪烁的问题。
- 修复悬停预览连接线未对齐列表项边框垂直中点的问题。
- 修复图片首次预览高度不足、内容显示不完整的问题。

## [1.5.2](https://github.com/FeatherBirth/EcoPaste/compare/v1.5.1...v1.5.2) (2026-11-11)

### 🐛 问题修复

- 修复剪贴板窗口在不同缩放比例显示器之间打开或移动时无法保持逻辑尺寸的问题。
- 修复 Windows 未发送 DPI 缩放事件时，程序化跨屏移动后窗口尺寸错误的问题。

## [1.5.1](https://github.com/FeatherBirth/EcoPaste/compare/v1.5.0...v1.5.1) (2026-11-10)

### ✨ 新功能

- 新增经校验的 GitHub Release Windows 一键安装脚本。
- 复制 Excel 与 WPS 表格时同时保留图片和纯文本格式。
- 为 Office 图片记录新增显式粘贴纯文本操作。

### 🐛 问题修复

- 修复远程桌面 DPI 变化后剪贴板窗口以半尺寸显示的问题。
- Windows 安装和卸载文件操作前自动结束残留的 EcoPaste 进程树。
- 覆盖现有机器级 Windows 安装时不再需要单独手动卸载。
- 自动升级后校验只启动一个 EcoPaste 进程。

## [1.1.0](https://github.com/EcoPasteHub/EcoPaste/compare/v1.0.0...v1.1.0) (2026-07-22)

### ✨ 新功能

* 新增保存图片右键菜单 ([#1353](https://github.com/EcoPasteHub/EcoPaste/issues/1353)) ([746cc05](https://github.com/EcoPasteHub/EcoPaste/commit/746cc0558d656ddb694c47c81a46549b89139c79))
* 新增每日构建更新通道支持 ([#1350](https://github.com/EcoPasteHub/EcoPaste/issues/1350)) ([9544725](https://github.com/EcoPasteHub/EcoPaste/commit/95447258f9e443d5285d46b775c7279d3d14fddc))

### 🐛 问题修复

* 修正旧版数据导入的时区偏移 ([#1338](https://github.com/EcoPasteHub/EcoPaste/issues/1338)) ([d6ea9ed](https://github.com/EcoPasteHub/EcoPaste/commit/d6ea9ed4d1ce6af99673a53a3c1ed78068f83d53))
* 去除重复的 Windows 开机启动项 ([#1355](https://github.com/EcoPasteHub/EcoPaste/issues/1355)) ([0d87458](https://github.com/EcoPasteHub/EcoPaste/commit/0d87458b70323da27456cfaa16b51dcde9ea220e))
* 防止应用卡死导致历史记录丢失 ([#1342](https://github.com/EcoPasteHub/EcoPaste/issues/1342)) ([7a75196](https://github.com/EcoPasteHub/EcoPaste/commit/7a75196e90f4a4abfb49d9f84c591d80c735a468))
* 为 Windows 开机启动路径添加引号 ([#1369](https://github.com/EcoPasteHub/EcoPaste/issues/1369)) ([d9d718e](https://github.com/EcoPasteHub/EcoPaste/commit/d9d718e456173e62f112ebd96609c049a331a1e8))
* 截图采集时重试读取 Windows 剪贴板 ([#1367](https://github.com/EcoPasteHub/EcoPaste/issues/1367)) ([e6f7e80](https://github.com/EcoPasteHub/EcoPaste/commit/e6f7e80e83ae18681a976929d710d99119168866))
* 没有来源应用的剪贴板记录显示 Logo ([#1370](https://github.com/EcoPasteHub/EcoPaste/issues/1370)) ([ad9eee0](https://github.com/EcoPasteHub/EcoPaste/commit/ad9eee0f1b7e3abe6546d4b3ac0fc8d4899bd5cb))
* 跳过旧版数据导入中的无效记录 ([#1326](https://github.com/EcoPasteHub/EcoPaste/issues/1326)) ([c0f2d58](https://github.com/EcoPasteHub/EcoPaste/commit/c0f2d587677ab51f9c707f1e50fda6a83ce78a3e))
* 更新测试版偏好设置文案 ([#1329](https://github.com/EcoPasteHub/EcoPaste/issues/1329)) ([185bb25](https://github.com/EcoPasteHub/EcoPaste/commit/185bb2520e8ce90da8246c0c73b1ce87e54403e5))

## [1.0.0](https://github.com/EcoPasteHub/EcoPaste/compare/v0.6.0-beta.3...v1.0.0) (2026-07-03)

这是 EcoPaste 的全新重构版本，应用整体更轻、更快，也更稳定。

### ✨ 新功能

- Windows 打开 EcoPaste 时支持不抢占其他窗口焦点。
- 新增 Windows Win+V 唤起开关，可用 EcoPaste 替代系统剪贴板面板。
- 新增首次启动引导，帮助完成权限、快捷键、忽略应用和旧版数据导入设置。
- 新增来源应用识别与忽略应用，可按应用控制哪些内容不保存到历史记录。
- 新增采集偏好，可控制保存的内容类型、大小限制和优先顺序。
- 新增敏感内容保护，自动跳过私钥、Token、AWS Key、JWT 等高风险内容。
- 新增完整内容预览，支持预览文本、图片和文件记录。
- 新增拖出能力，可将文本、富文本、图片和文件记录拖到外部应用。
- 新增收藏项与置顶项删除保护。
- 新增偏好设置搜索和重置偏好设置。
- 备份导入导出新增加密备份和合并导入支持。
- 新增 Windows 管理员权限启动设置。
- 软件更新改为独立窗口，可检查、下载并安装新版本。
- 更多新功能，等你下载体验与探索。

### 🐛 问题修复

- 修复 Windows 以管理员权限运行时开机自启动失效的问题。

### ⚡️ 性能优化

- 剪贴板监听、搜索、列表展示和内容预览更快更稳定。
- 应用启动、日常使用和后台资源占用进一步优化。
- 图片缩略图、应用图标和文件图标加载更高效。
- 新增轻量模式，窗口隐藏后减少后台刷新与内存占用。

### ⚠️ 升级说明

- 本重构版仅支持 macOS 和 Windows。
- 本版本支持迁移旧版历史数据，可在首次启动引导中完成导入。
