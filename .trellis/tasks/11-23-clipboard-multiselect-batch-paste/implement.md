# Multi-Select and Batch Paste Implementation

## Ordered Checklist

1. Add selection reducer/helpers for normal, Ctrl/Cmd, Shift, and Ctrl/Cmd+Shift clicks.
2. Refactor `List` selection state from `selectedId` only to focus + anchor + selected index set.
3. Update card highlighting and keyboard navigation to preserve focused vs selected styling.
4. Clear selection on query changes, deletes, window reset, and unmount.
5. Defer single-click action only when a multi-selection already contains the clicked focus item.
6. Route Enter, double-click, and Ctrl/Cmd+C through multi-selection branch when relevant.
7. Add repository range querying for full rows at selected indices.
8. Add `paste_clipboard_items` Rust command and structured result.
9. Implement text-only single-payload batch path.
10. Add experimental mixed setting across Rust, TypeScript, preference schema, visual icon map, and locales.
11. Implement mixed segmented sequential paste with stop-on-failure result.
12. Add command constants/wrappers and localized partial-failure feedback.
13. Add Rust unit tests for range compression/query ordering and batch text construction.
14. Run frontend and Rust checks, then Windows desktop interaction tests.

## Risky Files

- `src/pages/Clipboard/components/List.tsx`: large existing keyboard/action surface; preserve all non-selection branches.
- `src-tauri/src/db/items.rs`: reuse filter/sort SQL; do not duplicate search semantics.
- `src-tauri/src/commands/clipboard.rs`: preserve paste focus timing and Office image format writeback.
- `src-tauri/src/clipboard/write.rs`: add reusable text payload writing without bypassing guards.
- `src/pages/Preference/config/preferenceSchema.ts`: experimental switch must sit near paste controls and use default false.

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

1. Ctrl-click several non-contiguous text items; verify no paste on modified click.
2. Shift-click above/below the anchor repeatedly; verify range replacement.
3. Ctrl+Shift-click a second range; verify additive behavior.
4. Press Enter and double-click a selected item; verify one joined paste in visible list order.
5. Repeat with `singleClickPaste`; verify the deferred first click does not fire when a double-click completes.
6. Enable experimental mixed paste and verify text/image/text/files ordering and stop-on-failure behavior.
7. Disable mixed paste and verify mixed selection cannot paste implicitly as text.
8. Verify single-click, Space preview, middle-click, quick actions, context menu, and drag remain unchanged.

## Rollback Point

Revert the child commit to restore single-ID selection and remove the experimental setting. No database rollback is needed.

