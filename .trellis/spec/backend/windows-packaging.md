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

## Scenario: GitHub Latest One-Command Install

### 1. Scope / Trigger

Adding or changing remote release assets, download URLs, checksum verification,
or the published PowerShell bootstrap installer.

### 2. Signatures

- Source script: `scripts/installFromGitHub.ps1`
- Published script: `install-ecopaste.ps1`
- Published package alias: `EcoPaste_x64-setup.exe`
- Published checksum: `EcoPaste_x64-setup.exe.sha256`
- Default base URL:
  `https://github.com/FeatherBirth/EcoPaste/releases/latest/download`
- Optional test parameter: `-ReleaseBaseUrl <URL>`

### 3. Contracts

- Stable latest URLs are used; do not embed the release version in bootstrap
  download paths.
- HTTPS is required. Plain HTTP is accepted only for loopback test mirrors.
- The script downloads the package and checksum to a unique temp directory.
- SHA-256 must match before `Start-Process` is called.
- The installer runs with `/P /UPDATE /R`.
- The installed executable must equal the installer product version.
- Exactly one responsive Program Files process must be observed.
- Temporary downloads are removed in a `finally` block.

### 4. Validation & Error Matrix

- 32-bit Windows -> error before download.
- Non-HTTPS remote source -> error before download.
- Malformed checksum file -> error before execution.
- Hash mismatch -> error before execution.
- Installer non-zero -> propagate exit code.
- Installer timeout -> error after 180 seconds.
- No or multiple main processes -> error.

### 5. Good/Base/Bad Cases

- Good: latest public script downloads `v1.5.1`, verifies SHA-256, updates in
  place, and starts one process.
- Base: no existing install -> fresh machine-wide installation.
- Bad: `Invoke-WebRequest ... | Invoke-Expression` executes unchecked remote
  text without a saved inspectable file or package verification.

### 6. Tests Required

- Parse the script with the PowerShell AST parser.
- Run a loopback HTTP mirror with a deliberately wrong checksum and assert the
  installer is never started.
- Repeat with the correct checksum and assert installation succeeds.
- Verify all public latest URLs return HTTP 200.
- Verify public script and checksum bytes match the release source/build.
- Execute the downloaded public script and assert version plus single process.

### 7. Wrong vs Correct

Wrong:

```powershell
irm https://github.com/.../install-ecopaste.ps1 | iex
```

Correct:

```powershell
$script = "$env:TEMP\install-ecopaste.ps1"
Invoke-WebRequest "https://github.com/.../releases/latest/download/install-ecopaste.ps1" -OutFile $script
powershell -NoProfile -ExecutionPolicy Bypass -File $script
```
