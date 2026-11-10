# Technical Design

## Installer Mode

Set `bundle.windows.nsis.installMode` to `perMachine`.

The current installation already uses HKLM and Program Files. Keeping a single
stable scope avoids the `both` multi-user page selecting CurrentUser and failing
to match the existing installation. It also means the installer and uninstaller
always run elevated and can terminate elevated application processes.

## Process Cleanup Hook

Add `src-tauri/nsis/installer-hooks.nsh` and configure `installerHooks`.

`NSIS_HOOK_PREINSTALL` and `NSIS_HOOK_PREUNINSTALL` both:

1. Run `taskkill /F /T /IM "EcoPaste.exe"` through `nsExec`.
2. Wait briefly for the complete process tree to release executable and state
   file handles.

Failure is ignored when no process exists. The hook runs before Tauri's built-in
running-app check and before file replacement.

## Automated Upgrade

Add `scripts/installWindows.ps1`:

- Resolve a supplied installer or the newest
  `src-tauri/target/release/bundle/nsis/EcoPaste_*-setup.exe`.
- Verify the file exists.
- Start it with `/P /UPDATE /R` and wait for the installer process.
- Fail on a non-zero installer exit code.
- Wait for a new responsive `EcoPaste` process and report its path/PID.

PowerShell `Start-Process -Wait` also waits for installer descendants, including
the newly launched app. The wrapper therefore starts the installer with
`-PassThru` and calls `WaitForExit(180000)` on only the installer process.

Expose it as `pnpm install:windows`.

`/UPDATE` bypasses uninstall and preserves shortcuts/startup metadata. `/P`
hides setup pages while retaining passive progress, and `/R` asks the installer
to launch the app once after completion.

## Startup Safety

The primary startup reliability fix is eliminating stale process trees before
executable replacement. This addresses the observed state where a prior main
process remained stuck before tray/shortcut registration and subsequent
instances could not initialize.

The app's existing Tauri single-instance plugin remains the runtime duplicate
launch guard. The installer must not launch a second copy while the finish page
or previous process is still starting.

## Data Compatibility

- No database or settings migration is required.
- Updating with `/UPDATE` does not remove app data or HKCU startup entries.
- The administrator logon task remains untouched and points at the same stable
  executable path.

## Rollback

- Installer hooks are inert outside package maintenance.
- Reverting `installMode` to `both` restores prior installer behavior.
- A failed upgrade leaves the existing executable available until NSIS begins
  replacement; the automated wrapper stops before launch on installer failure.
