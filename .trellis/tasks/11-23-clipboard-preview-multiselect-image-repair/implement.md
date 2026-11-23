# Cross-Feature Integration Implementation

## 1.6.0 Release and Auto Update

1. Move all product/release metadata to 1.6.0 and the current repository.
2. Remove legacy update settings, endpoints, sponsor QR, and obsolete UI links.
3. Add the fork updater public key and store its private key only as a GitHub
   Actions secret.
4. Build each platform with signatures, then construct one complete latest.json
   in the final release job.
5. Configure Windows updater requests to honor the WinINet HTTP proxy.
6. Add System/Direct/Custom proxy controls and restore the current GitHub link.
7. Preserve the original P icon geometry while regenerating platform icons and
   branded NSIS header/sidebar artwork.
8. Run frontend/Rust quality gates, sign the Windows package, install it, and
   verify update check/download/install through an HTTPS signed manifest.

## Ordered Checklist

1. Complete planning review for all children.
2. Implement preview lifecycle and interaction first because it addresses the active defect.
3. Implement multi-select and batch paste.
4. Implement Windows pure-image repair.
5. Run per-child focused validation.
6. Run full frontend and Rust quality gates.
7. Perform integrated manual verification on Windows.
8. Perform available macOS build/check verification.
9. Update shared clipboard/preference specs with newly established contracts.
10. Commit each independently coherent unit without mixing unrelated dirty-worktree changes.

## Validation Matrix

- Preview: fast cross-item, cross-app, skip short text, enter/leave panel, text select/copy, image wheel zoom.
- Multi-select: Ctrl/Cmd toggle, Shift range, anchor recalculation, Enter/double-click, text-only batch, mixed sequential batch, failure stop.
- Image repair: switch off, PNG-only screenshot, normal browser image, Excel/WPS mixed table copy, plain-text paste.
- Settings: defaults, persistence across restart, reset, preference search, zh-CN/en-US text, Windows-only visibility.

## Risky Integration Points

- `src/commands/index.ts`: command names, argument shapes, error behavior, centralized wrappers.
- `src-tauri/src/lib.rs`: command registration.
- `src-tauri/src/clipboard/write.rs`: existing Office multi-format behavior.
- `src-tauri/src/clipboard/watcher.rs`: readonly/no-op when settings disable features and no self-copy loop.
- `src/pages/Clipboard/components/List.tsx`: auto-paste semantics and virtual selection indices.
- `src-tauri/src/window/preview.rs`: request ordering, pointer input routing, and focus restoration.

## Rollback Points

- Revert preview child commit if native input routing regresses cross-platform behavior.
- Revert multi-select child commit if selection/query ordering drifts.
- Revert image repair call independently if a target application rejects rewritten formats.
- None of these rollbacks require database or resource migration.

