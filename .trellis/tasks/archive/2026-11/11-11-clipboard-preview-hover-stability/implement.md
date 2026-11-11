# Clipboard Hover Preview Stability Implementation

## Ordered Checklist

1. Update the Trellis task scope and branch metadata.
2. Remove pointer-Y from the TS/Rust preview anchor contract.
3. Simplify the preview controller:
   - remove same-item pointer-move retargeting;
   - remove requestAnimationFrame retarget refs and cleanup;
   - keep enter/leave/hide behavior unchanged.
4. Remove the now-unused pointer-move plumbing from `List` and `ClipboardCard`.
5. Update Rust source-rect resolution so the full item rect is retained.
6. Replace the conditional measurement hook with callback-ref node lifecycle handling.
7. Update first-layout readiness so transient loading geometry cannot end the jump-only phase.
8. Update Rust unit tests from pointer-anchor behavior to item-center geometry.

## Risky Files

- `src/pages/Clipboard/hooks/useClipboardPreviewController.ts`
  - Preserve hover delay, cross-item switching, keyboard preview, scroll close, and hide buffering.
- `src-tauri/src/window/preview.rs`
  - Preserve monitor/DPI mapping, placement fallback, clamp, and overlay geometry.
- `src/pages/Preview/measurement.ts`
  - Ensure observer cleanup is correct across conditional mount/unmount.
- `src/pages/Preview/index.tsx`
  - Verify loading, cached payload, image, text, files, and exit animation paths.

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

Manual Windows verification:

1. Open a long text hover preview, move within the item, and verify no flicker.
2. Move from one item to another and verify preview retargeting.
3. Position the clipboard window so preview is on the right, then on the left; verify the source dot is on the corresponding item border at its vertical midpoint.
4. Open image items repeatedly, including the first preview after app start, and verify the full image appears without pointer movement.
5. Verify keyboard Space preview, scroll closing, and hover hide buffering.

## Rollback Point

Revert this task's commit to restore pointer-following anchors and the previous measurement behavior. No data rollback is needed.
