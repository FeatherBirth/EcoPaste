# Windows Packaging and Upgrade

## Scenario: Machine-Wide Upgrade Installation

### 1. Scope / Trigger

Changing NSIS install mode, installer hooks, package versioning, or Windows
upgrade automation.

### 2. Signatures

- Config: `bundle.windows.nsis.installMode = "perMachine"`
- Config: `bundle.windows.nsis.installerHooks = "nsis/installer-hooks.nsh"`
- Script: `scripts/installWindows.ps1 [-InstallerPath <path>]`
- Script: `scripts/packageWindows.ps1 [-SkipInstall]`
- Package commands:
  - `pnpm install:windows`
  - `pnpm package:windows`
- Installer arguments: `/P /UPDATE /R`

### 3. Contracts

- EcoPaste uses one machine-wide install at `C:\Program Files\EcoPaste` with HKLM
  uninstall metadata. Do not reintroduce `installMode = "both"` for upgrades.
- `NSIS_HOOK_PREINSTALL` and `NSIS_HOOK_PREUNINSTALL` run
  `taskkill /F /T /IM EcoPaste.exe` before file operations.
- `/UPDATE` overwrites the existing installation without invoking `uninstall.exe`
  or deleting user data.
- `/P` runs the installer with passive progress UI.
- `/R` starts the installed application once after completion.
- `package:windows` skips updater signing only when
  `TAURI_SIGNING_PRIVATE_KEY` is absent; otherwise it preserves signing.
- The install wrapper waits for the installer process, not its launched child
  application, then verifies one responsive installed process.

### 4. Validation & Error Matrix

- No EcoPaste process -> `taskkill` is ignored; installation continues.
- Running EcoPaste main/WebView processes -> complete process tree is terminated
  before executable replacement.
- Installer exit code non-zero -> wrapper throws and does not claim success.
- App not started within 30 seconds -> wrapper throws.
- More than one matching main process -> wrapper throws.
- Missing signing key -> package still builds with `--no-sign`.

### 5. Good/Base/Bad Cases

- Good: an installed 1.5.0 is replaced in place by 1.5.1 and one 1.5.1 process
  starts.
- Base: a fresh machine installs into Program Files and launches from the finish
  page.
- Bad: choose user/machine scope again during every upgrade and leave an old
  elevated process holding files or startup resources.

### 6. Tests Required

- Parse-check both PowerShell scripts.
- Build NSIS with `--no-sign` in the local unsigned environment.
- Upgrade an existing installation twice and assert old PIDs disappear.
- Assert exactly one responsive process remains.
- Assert installed file/ProductVersion and HKLM `DisplayVersion` match the
  package version.
- Run Rust/frontend quality gates.

### 7. Wrong vs Correct

Wrong:

```powershell
Start-Process $setup -ArgumentList "/S", "/R" -Wait
```

`-Wait` can wait for the launched application tree indefinitely.

Correct:

```powershell
$installer = Start-Process $setup -ArgumentList "/P", "/UPDATE", "/R" -PassThru
$installer.WaitForExit(180000)
```
