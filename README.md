<div align="center">
  <img src="./public/logo.png" alt="EcoPaste" width="96" height="96" />

# EcoPaste

**A local-first clipboard manager for macOS and Windows.**

English | [简体中文](./README.zh-CN.md)

  <br />

  <img alt="Tauri v2" src="https://img.shields.io/badge/Tauri-v2-24c8db?style=flat-square" />
  <img alt="Rust first" src="https://img.shields.io/badge/Rust-first-b7410e?style=flat-square" />
  <img alt="React 19" src="https://img.shields.io/badge/React-19-61dafb?style=flat-square" />
  <img alt="macOS" src="https://img.shields.io/badge/macOS-supported-000000?style=flat-square&logo=apple&logoColor=white" />
  <img alt="Windows" src="https://img.shields.io/badge/Windows-supported-0078d4?style=flat-square&logo=windows&logoColor=white" />
</div>

## About

EcoPaste is an open-source desktop clipboard manager built with a Rust-first Tauri architecture: durable behavior lives in Rust, while the React frontend focuses on rendering and interaction.

The app is designed to be fast, lightweight, and maintainable, with local storage, SQLite search, native shortcuts, tray integration, backup support, and a focused cross-platform surface for macOS and Windows.

## Features

- Capture clipboard history for plain text, HTML, RTF, images, files, and folders.
- Search clipboard content and notes with SQLite FTS5.
- Filter history by source application and content type.
- Protect sensitive content by skipping high-confidence secrets such as private keys, service tokens, AWS keys, and JWTs.
- Preview text, images, and files in a dedicated preview window.
- Keep previews interactive with selectable text, scrolling, whole-panel image zoom, and reliable cleanup when the pointer moves away.
- Select multiple history entries with ordered badges, then batch paste in the chosen order.
- Paste, copy, copy as plain text, reveal files, open links, add notes, pin, favorite, delete, and drag items out to other apps.
- Organize history with favorites, pinned items, notes, custom groups, and configurable item actions.
- Tune capture order, size limits, retention, display density, list sorting, and window behavior.
- Export and import `.ecopastebak` backups, including encrypted backup containers.
- Keep clipboard data, resources, and settings local to your machine.
- Check signed updates from this repository's GitHub Releases.
- Follow the Windows system proxy automatically or choose a direct/custom update proxy.

## Updates

EcoPaste reads the signed updater manifest from
`https://github.com/FeatherBirth/EcoPaste/releases/latest/download/latest.json`.
Each release includes signed packages for Windows x64/ARM64 and macOS
Intel/Apple Silicon.

Version 1.6.0 introduces the current updater signing identity. Install 1.6.0
manually once; later versions can be downloaded and installed from the in-app
update window.

Update network access defaults to the system proxy. If GitHub is reachable only
through a specific local proxy, open Preferences -> Data -> Updates and select
the custom proxy mode.

## Install on Windows

Windows x64 users can install the latest release with one PowerShell command:

```powershell
irm https://github.com/FeatherBirth/EcoPaste/releases/latest/download/install-ecopaste.ps1 | iex
```

The command downloads and runs the latest installer script. The script verifies
the package SHA-256 checksum, upgrades the existing machine-wide installation
without running a separate uninstaller, preserves user data, and launches
EcoPaste once. Windows will show the normal UAC elevation prompt.

## Install on macOS

Download the DMG for your Mac:

- [Apple Silicon (M1/M2/M3/M4)](https://github.com/FeatherBirth/EcoPaste/releases/latest/download/EcoPaste-macos-arm64.dmg)
- [Intel](https://github.com/FeatherBirth/EcoPaste/releases/latest/download/EcoPaste-macos-x64.dmg)

Open the DMG and drag EcoPaste into `Applications`. If macOS blocks the first
launch, open System Settings -> Privacy & Security and choose Open Anyway, or
Control-click EcoPaste in Finder and choose Open.

## Contributing

Development setup, architecture notes, quality checks, and contribution
expectations live in the [contribution guide](./CONTRIBUTING.md).

## License

EcoPaste is licensed under the [Apache License 2.0](./LICENSE).
