# Implementation Plan

## Ordered Changes

1. Set the NSIS Windows install mode to `perMachine`.
2. Add installer hooks that terminate the EcoPaste process tree before install
   and uninstall file operations.
3. Add a Windows upgrade PowerShell wrapper using `/P /UPDATE /R`, exit-code
   validation, and post-install process verification.
4. Expose the wrapper through a package script.
5. Bump `package.json`, `Cargo.toml`, and `Cargo.lock` to `1.5.1`.
6. Build the NSIS package from the updated source.
7. Run the automated installer over the existing 1.5.0 installation.
8. Verify process count, responsiveness, executable version, and HKLM uninstall
   metadata.
9. Run the upgrade a second time to verify overwrite/reinstall idempotence.
10. Update installer/startup specs and commit all changes.

## Validation

- Rust: `cargo fmt -- --check`, Clippy with warnings denied, `cargo test`.
- Frontend: `pnpm lint`, `pnpm tsc`, `pnpm build`.
- Package: `pnpm tauri build --bundles nsis`.
- Upgrade: `pnpm install:windows -- --InstallerPath <setup.exe>`.
- Runtime: one responsive main process; installed file version `1.5.1`.
- Registry: HKLM uninstall DisplayVersion `1.5.1`, InstallLocation
  `C:\Program Files\EcoPaste`.

## Risk and Rollback

- Force termination can lose unsaved in-app transient state but preserves
  persisted clipboard data and settings.
- The installer must not remove app data.
- If the passive update does not overwrite the existing installation, stop and
  inspect the generated NSIS script rather than running a separate uninstall.
