# Windows Pure-Image Clipboard Repair Implementation

## Ordered Checklist

1. Add the default-false settings field and both language mirrors.
2. Add a Windows-only preference schema entry and visual icon mapping.
3. Capture Windows clipboard sequence number and raw image/text format state during image read.
4. Extend internal `ImagePayload` with repair metadata behind cross-platform-neutral fields.
5. Add `clipboard/repair.rs` with a pure classifier and Windows rewrite function.
6. Call repair after successful history persistence only when the setting is enabled.
7. Re-check sequence, pure-image state, and standard bitmap availability immediately before rewrite.
8. Reuse `write_image`/`ImageStore` and arming `WritebackGuard`; do not duplicate image encoding logic.
9. Add unit tests for classifier, Office exclusion, setting-off no-op, and changed sequence.
10. Add an ignored Windows clipboard round-trip test for PNG-only rewrite and loop suppression.
11. Run quality gates and Windows screenshot/Excel manual verification.

## Risky Files

- `src-tauri/src/clipboard/read.rs`: preserve current PNG-first and DIB/TIFF fallback behavior.
- `src-tauri/src/clipboard/watcher.rs`: repair must not block normal non-image events or fail ingestion.
- `src-tauri/src/clipboard/write.rs`: preserve Office image + text writeback and plain-text-only mode.
- `src-tauri/src/settings/model.rs`: additive default false only; no schema migration.
- `src/pages/Preference/config/preferenceSchema.ts`: hide the entry on macOS using the existing platform spread pattern.

## Validation

Frontend:

```powershell
pnpm lint
pnpm tsc
```

Rust:

```powershell
cd src-tauri
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

Manual Windows:

1. Switch off and reproduce the problematic screenshot copy; behavior must match current release.
2. Switch on and copy the same screenshot; verify history ingest, standard bitmap availability, and direct paste.
3. Copy the same image again; verify no duplicate history row from repair.
4. Copy Excel/WPS cells and verify normal paste can yield text or image according to target, while plain-text paste is text only.
5. Copy browser/editor images with standard bitmap formats; verify no unnecessary rewrite.
6. Copy another clipboard value immediately after a screenshot to verify sequence-based stale repair suppression.

## Rollback Point

Disable the setting or revert the child commit. History rows and image files require no cleanup.

