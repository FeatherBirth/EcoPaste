# Preview Lifecycle and Interaction Implementation

## Ordered Checklist

1. Add failing-focused Rust tests for monotonic preview request acceptance.
2. Add `requestId` to preview show/close command contracts and frontend wrappers.
3. Gate Rust show/close/hide scheduling on the latest request id and include request id in preview state/session.
4. Add list-area pointer target accounting so background/non-preview targets enter the hide buffer.
5. Add the preview interaction rect command and register/mirror its command constant.
6. Add Rust cursor polling, mouse-event toggling, interaction event emission, and cleanup for stale sessions.
7. Add Windows foreground save/restore and macOS nonactivating key-panel focus.
8. Subscribe the clipboard preview controller to interaction events and suspend/resume hide timers.
9. Replace text virtual rows with a selectable text container, raise the default text panel height, and rely on real overflow for scrollbar visibility.
10. Add image wheel zoom with pointer-relative scroll adjustment and reset-on-item-change.
11. Run focused checks and Windows desktop validation.

## Risky Files

- `src-tauri/src/window/preview.rs`: preserve placement, topmost stacking, session id, exit delay, and destroy lifecycle.
- `src/pages/Clipboard/hooks/useClipboardPreviewController.ts`: preserve Space preview, scroll close, filter close, and keyboard retargeting.
- `src/pages/Preview/index.tsx`: report final dynamic panel rect and avoid reporting while exit-only state is rendered.
- `src/pages/Preview/components/PreviewContent.tsx`: textarea selection, wheel default prevention, and scroll containment.
- `src-tauri/src/window/windows.rs` and `window/macos.rs`: focus behavior must not regress clipboard-window focus or paste timing.

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

1. Open a long-text preview and immediately move to a short/fully visible item, list blank space, another monitor, and another foreground app.
2. Traverse several previewable items rapidly and stop on a non-preview item.
3. Move from the source item into the panel; scroll, select, Ctrl+A, Ctrl+C, then move out and verify the preview closes after the buffer.
4. Open an image, wheel zoom in/out, scroll a zoomed image, leave and reopen; verify fit reset.
5. Confirm clicks outside the panel reach the underlying app.
6. Hold Space and verify existing keyboard preview still opens/closes.

macOS checks:

- compile both targets;
- manually verify panel pointer capture, text selection, and image wheel zoom where a desktop session is available.

## Rollback Point

Revert this child commit to restore click-through-only preview and the prior text viewer. No data rollback is needed.

