# Implementation Plan

## Ordered Changes

1. Add `scripts/installFromGitHub.ps1` with checksum verification and install
   verification.
2. Extend Release CI to create and upload stable Windows asset names.
3. Add a final release publish job after the build matrix succeeds.
4. Document download-then-run installation in both README languages.
5. Parse-check PowerShell scripts and run quality gates.
6. Add `v1.5.1` changelog sections required by Release CI.
7. Build and install/test the package locally.
8. Publish release `v1.5.1` with the normal installer, stable installer alias,
   checksum, and remote script.
9. Verify public latest asset URLs and checksum.
10. Execute the remote script against the public URL and verify one running
    1.5.1 instance.
11. Commit and push all source/workflow/documentation changes.

## Validation

- PowerShell AST parse for both local and remote install scripts.
- Negative checksum test using a temporary local HTTP mirror or altered sidecar.
- `pnpm lint`, `pnpm tsc`, `pnpm build`.
- `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test`.
- `pnpm package:windows` for local packaging and in-place install.
- `Invoke-WebRequest` for all public release URLs.
- Run the downloaded public `install-ecopaste.ps1`.

## Risk and Rollback

- Do not overwrite a legitimate release asset with a different version; the
  stable alias must come from the same tag's matrix build.
- A failed checksum must stop before launching setup.
- Keep the versioned installer asset for traceability.
