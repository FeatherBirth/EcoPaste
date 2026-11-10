# Technical Design

## Release Asset Contract

Every non-prerelease GitHub Release publishes:

- `EcoPaste_x64-setup.exe`: exact copy of the versioned NSIS installer built by
  the Windows x64 matrix job.
- `EcoPaste_x64-setup.exe.sha256`: standard lowercase SHA-256 followed by two
  spaces and the stable installer filename.
- `install-ecopaste.ps1`: download-and-verify installer from the release.

Users can inspect/download the script from:

```text
https://github.com/FeatherBirth/EcoPaste/releases/latest/download/install-ecopaste.ps1
```

The script and default asset base URL use repository
`FeatherBirth/EcoPaste`.

## Remote Installer Flow

1. Validate `$env:ProgramFiles` and `PROCESSOR_ARCHITECTURE`.
2. Enable TLS 1.2+.
3. Download the checksum, installer, and script inputs over HTTPS.
4. Parse and compare SHA-256 using `Get-FileHash`.
5. Read the installer file version and retain it as the expected version.
6. Start the installer with `/P /UPDATE /R` and wait for the installer process.
7. Require exit code zero.
8. Wait for one `EcoPaste` main process at
   `$env:ProgramFiles\EcoPaste\EcoPaste.exe`.
9. Require the installed executable version to match the installer version.
10. Delete the temporary directory in a `finally` block.

An optional `-ReleaseBaseUrl` parameter supports deterministic validation
against a local/static mirror without changing the production default.

## Release Workflow Integration

After `tauri-action` uploads matrix artifacts in the Windows x64 job:

1. Locate the generated versioned NSIS installer recursively.
2. Copy it to a staging directory as `EcoPaste_x64-setup.exe`.
3. Generate its SHA-256 sidecar.
4. Copy `scripts/installFromGitHub.ps1` as `install-ecopaste.ps1`.
5. Upload the three files to the draft release with
   `gh release upload --clobber`.

Add a final publish job that flips the draft release to published only after all
matrix builds and upload steps succeed.

## Security and Compatibility

- HTTPS plus checksum prevents partial/corrupted downloads.
- The checksum is not a substitute for Authenticode; document that limitation.
- `Invoke-WebRequest -UseBasicParsing` remains compatible with Windows
  PowerShell 5.1.
- UAC remains required for machine-wide installation under Program Files.
- The installer's `/UPDATE` mode preserves settings and data.

## Rollback

- Remove the remote script and stable assets from a release without affecting
  users who already installed.
- Revert workflow upload steps to restore versioned-only release artifacts.
