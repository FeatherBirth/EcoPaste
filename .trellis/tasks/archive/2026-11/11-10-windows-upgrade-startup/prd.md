# Harden Windows upgrade installation and startup

## Goal

Make a Windows package upgrade replace the existing machine-wide installation
without a separate manual uninstall, guarantee that old EcoPaste processes are
terminated before files are replaced, and start exactly one usable instance
after installation.

## Background

- The installed application is registered per-machine at
  `HKEY_LOCAL_MACHINE\Software\Microsoft\Windows\CurrentVersion\Uninstall\EcoPaste`
  and lives in `C:\Program Files\EcoPaste`.
- The current NSIS configuration uses `installMode: "both"`, which asks the
  installer to choose user/machine scope again and can conflict with the
  existing machine-wide registration.
- Tauri's generated NSIS script supports `/UPDATE` to overwrite an existing
  installation without running the uninstaller, `/P` for passive UI, and `/R`
  to launch the installed app once after a passive/silent install.
- Tauri's default reinstall page otherwise defaults upgrades to uninstall first.
- The PREINSTALL hook runs immediately before file copying, while the built-in
  running-app check runs just after it.
- NSIS rejects or corrupts installation when the previous application process
  still holds executable files or startup resources.
- The reported stuck startup had WebView child processes but no tray icon or
  global shortcut. Killing the stale process group and launching a fresh
  instance restored normal operation.

## Requirements

1. Switch the Windows installer to a stable per-machine install mode so upgrades
   use the existing `C:\Program Files\EcoPaste` and HKLM uninstall registration.
2. Before installation and uninstallation file operations, terminate the
   complete EcoPaste process tree, including WebView children, with bounded wait
   and no interactive prompt.
3. Provide an automated upgrade command that invokes the generated installer in
   passive update mode and launches the installed application once.
4. Preserve user data, settings, shortcuts, autostart intent, and administrator
   task configuration.
5. Build version `1.5.1` and replace the currently installed `1.5.0`.
6. Verify there is exactly one responsive main process after upgrade and that
   the installed executable and uninstall entry report `1.5.1`.
7. Keep actual uninstall behavior destructive only when the user explicitly
   runs the uninstaller.

## Acceptance Criteria

- [x] Running the upgrade command over the current 1.5.0 installation does not
      open a separate uninstall UI and does not require manual uninstall.
- [x] Every old EcoPaste process is gone before NSIS copies the new executable.
- [x] The installer exits successfully and the application starts once.
- [x] No duplicate EcoPaste main process remains after startup.
- [x] Tray/global-shortcut initialization can proceed because stale lock-holding
      processes have been removed.
- [x] Existing clipboard data and settings remain in place.
- [x] Installed executable and HKLM uninstall metadata report `1.5.1`.
- [x] Rust and frontend quality gates pass.
- [x] Repeated upgrade execution remains idempotent.

## Out of Scope

- Changing the application's user-facing uninstall behavior for explicit
  uninstalls.
- Removing user data during upgrade.
- Signing the installer or changing update-server infrastructure.

## Technical Notes

- NSIS hooks are Rust-independent installer assets and must not rely on the app
  being runnable.
- `taskkill /F /T /IM EcoPaste.exe` is acceptable only in installer/uninstaller
  hooks where the user has explicitly initiated package maintenance.
- The generated installer's `/UPDATE /P /R` flags preserve installation
  metadata and avoid invoking `uninstall.exe`.

## Validation Notes

- Successful one-command run: `pnpm package:windows`.
- Repeated upgrade changed the main process PID and left exactly one responsive
  process each time.
- `Alt+C` opened the clipboard window after install.
- Installed executable reports `1.5.1`; HKLM reports `1.5.1`.
- The installer is unsigned and built with `--no-sign`.
- NSIS package SHA-256:
  `60ACF01A58E33A58C4A90EBBC2EC1BC085A71A2A7E8EDC57993CC46F573F12F8`.
