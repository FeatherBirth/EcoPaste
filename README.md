<div align="center">
  <img src="./public/logo.png" alt="EcoPaste" width="96" height="96" />

  # EcoPaste

  **适用于 macOS 与 Windows 的本地优先剪贴板管理器。**

  <img alt="Tauri v2" src="https://img.shields.io/badge/Tauri-v2-24c8db?style=flat-square" />
  <img alt="Rust First" src="https://img.shields.io/badge/Rust-First-b7410e?style=flat-square" />
  <img alt="React 19" src="https://img.shields.io/badge/React-19-61dafb?style=flat-square" />
  <img alt="Windows" src="https://img.shields.io/badge/Windows-supported-0078d4?style=flat-square&logo=windows&logoColor=white" />
  <img alt="macOS" src="https://img.shields.io/badge/macOS-supported-000000?style=flat-square&logo=apple&logoColor=white" />
</div>

EcoPaste 使用 Rust-First 的 Tauri 架构，专注快速复制、搜索、预览和复用剪贴板内容。历史数据、图片资源与设置默认保存在本机，并针对 Windows 与 macOS 提供原生快捷键、托盘和窗口体验。

## 下载与安装

| 平台 | 下载 |
| --- | --- |
| Windows x64 | [安装包](https://github.com/FeatherBirth/EcoPaste/releases/latest/download/EcoPaste_x64-setup.exe) |
| Windows ARM64 | [前往最新 Release](https://github.com/FeatherBirth/EcoPaste/releases/latest) 选择 `arm64-setup.exe` |
| macOS Apple Silicon | [DMG](https://github.com/FeatherBirth/EcoPaste/releases/latest/download/EcoPaste-macos-arm64.dmg) |
| macOS Intel | [DMG](https://github.com/FeatherBirth/EcoPaste/releases/latest/download/EcoPaste-macos-x64.dmg) |

Windows 安装器会覆盖升级现有安装并保留用户数据。macOS 首次运行若被系统拦截，可在“系统设置 → 隐私与安全性”中选择“仍要打开”。

## 核心功能

### 收集与查找

- 支持纯文本、HTML、RTF、图片、文件和文件夹。
- 使用 SQLite FTS5 搜索正文与备注，并按来源应用、内容类型、自定义分组过滤。
- 可配置采集顺序、大小上限、清理周期、最大条数和列表排序。
- 自动识别并跳过私钥、Token、AWS Key、JWT 等高置信敏感内容。

### 选择与复用

- 支持单击、双击、中键、数字快捷键、右键菜单和拖出到其他应用。
- 可复制、纯文本复制、粘贴、纯文本粘贴、收藏、置顶、备注、删除和定位文件。
- 支持有序多选与批量粘贴：Ctrl/Cmd 单击追加选择，Shift 单击连续选择，按序号顺序粘贴。
- 实验性混合粘贴可按顺序处理图片和文件；纯文本选择仍合并为一次粘贴。

### 交互式预览

- 文本预览随内容伸缩，达到列表高度上限后自动滚动。
- 文本可直接拖选、全选和复制，鼠标移入预览框时不会关闭。
- 图片预览支持滚轮、Ctrl + 滚轮和 Windows 精确触摸板缩放，面板与图片整体等比缩放。
- 快速切换条目、移入空白区域或切换应用时，旧预览会可靠关闭。

### 本地存储与备份

- 历史记录、图片、图标、设置和日志均保存在本机。
- 支持 `.ecopastebak` 备份导入导出、加密备份、合并导入和覆盖恢复。
- 支持自定义数据目录、缓存清理和旧版数据迁移。

### Windows 增强

- 可用 Win+V 唤起 EcoPaste，替代系统剪贴板面板。
- 支持开机启动、管理员权限启动、焦点保护、托盘与任务栏控制。
- 可自动修复部分电脑“截图后无法直接粘贴”的纯图片格式问题，默认关闭。
- Excel/WPS 表格复制会同时保留图片和文本语义，不改变目标应用原有的粘贴行为。

## 1.5 以后重点更新

- **1.5.1**：提供经过校验的 GitHub Release Windows 安装脚本；完善 Excel/WPS 表格多格式复制与纯文本粘贴。
- **1.5.2**：修复不同 DPI 显示器、远程桌面和跨屏移动后的窗口尺寸问题。
- **1.5.3**：修复预览闪烁、连接线错位和图片首次预览尺寸不足。
- **1.6.0**：加入可靠预览生命周期、可选文本/可缩放预览、有序多选批量粘贴、截图格式修复、跨屏 DPI 对齐和品牌化安装器。
- **1.6.0**：自动更新切换到当前 GitHub Release，支持 Windows/macOS 四平台签名产物与系统代理、直连、自定义代理三种模式。

## 自动更新

应用内更新读取当前仓库的签名清单：

```text
https://github.com/FeatherBirth/EcoPaste/releases/latest/download/latest.json
```

更新网络默认跟随系统代理。若 GitHub 只能通过特定代理访问，可在“偏好设置 → 数据 → 更新”中选择自定义代理。

从 1.5.x 升级到 1.6.0 需要手动安装一次，以切换到新的 updater 签名身份；之后的版本可在应用内自动检查、下载和安装。

## 开发

技术栈：Tauri v2、Rust、React 19、Ant Design、SQLite、Vite、pnpm。

开发环境、架构说明和质量检查命令见[贡献指南](./CONTRIBUTING.zh-CN.md)。

## 开源协议

EcoPaste 基于 [Apache License 2.0](./LICENSE) 开源。
