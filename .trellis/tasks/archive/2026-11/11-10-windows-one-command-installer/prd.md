# Publish one-command Windows installer

## Goal

Publish a versioned PowerShell installer asset on GitHub so a Windows user can
download one script, verify the installer package, overwrite the existing
machine-wide EcoPaste installation, and launch exactly one working instance.

## Background

- The local `pnpm package:windows` flow already builds and upgrades in place,
  but users need a repository checkout and Node/pnpm environment.
- GitHub Releases can expose stable asset names through
  `releases/latest/download/<asset-name>`.
- The existing Release CI builds Windows x64 NSIS artifacts with
  `tauri-action`, but does not publish a fixed-name installer, checksum, or
  remote installer script.
- Downloading and piping arbitrary PowerShell with `irm | iex` is unsafe.
  The supported flow should download the script to a file so it can be
  inspected, then execute it.
- The package is currently unsigned, so SHA-256 detects corruption and
  accidental tampering but is not an Authenticode trust boundary.

## Requirements

1. Add a PowerShell installer that downloads the latest stable GitHub Release
   assets and installs them in-place.
2. Publish these stable Release assets for Windows x64:
   - `install-ecopaste.ps1`
   - `EcoPaste_x64-setup.exe`
   - `EcoPaste_x64-setup.exe.sha256`
3. The remote script must:
   - Require 64-bit Windows.
   - Use TLS 1.2 or newer.
   - Download into a unique temporary directory.
   - Verify the SHA-256 checksum before execution.
   - Invoke the installer with `/P /UPDATE /R`.
   - Wait only for the installer process, not the launched application tree.
   - Verify the installed version and exactly one responsive process.
   - Remove temporary downloads on success or failure.
4. Release CI must upload the stable-named assets from the actual Windows x64
   package built for the same tag.
5. Add Chinese and English README instructions using a download-then-run
   command rather than `irm | iex`.
6. Publish GitHub Release `v1.5.1` with the stable assets and verify the public
   `releases/latest/download` URLs.
7. Run the remote script against its public GitHub URL and verify an in-place
   upgrade and single-process startup.

## Acceptance Criteria

- [x] Release workflow uploads all three stable Windows assets.
- [x] Public latest-release URLs return HTTP 200 for all three assets.
- [x] Downloaded installer checksum matches the published checksum.
- [x] The remote script rejects a checksum mismatch before executing the setup.
- [x] Running the public script upgrades the existing installation without a
      separate uninstall prompt.
- [x] Final installed version is `1.5.1` and exactly one responsive process
      remains.
- [x] Existing settings and clipboard data remain intact.
- [x] PowerShell parser, lint, TypeScript, Rust tests, and package build pass.

## Out of Scope

- Authenticode signing infrastructure.
- Bypassing Windows UAC.
- Supporting 32-bit Windows or Windows 7.
- Replacing the app's existing updater endpoint.

## Technical Notes

- Prefer GitHub `releases/latest/download` URLs with stable asset names.
- Keep the script self-contained; it may use only Windows PowerShell 5.1 built-in
  modules.
- HTTP is never accepted for package downloads.

## Validation Notes

- Release:
  `https://github.com/FeatherBirth/EcoPaste/releases/tag/v1.5.1`.
- Published assets:
  `EcoPaste_1.5.1_x64-setup.exe`, `EcoPaste_x64-setup.exe`,
  `EcoPaste_x64-setup.exe.sha256`, `install-ecopaste.ps1`.
- All three `releases/latest/download` URLs returned HTTP 200.
- Stable installer SHA-256:
  `60ACF01A58E33A58C4A90EBBC2EC1BC085A71A2A7E8EDC57993CC46F573F12F8`.
- The downloaded public script checksum matched the tracked source.
- A local loopback mirror test rejected an invalid checksum before execution,
  then completed successfully with the corrected checksum.
- Executing the downloaded public script completed an in-place upgrade and
  started exactly one responsive `EcoPaste 1.5.1` process.
