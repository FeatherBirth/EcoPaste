# Clipboard Hover Preview Stability Design

## Architecture and Boundaries

This change stays within the existing preview path:

- React owns pointer intent, item DOM rect collection, and panel content measurement.
- Rust owns screen-space layout and overlay placement.
- The command contract remains `showClipboardPreview(itemId, anchor)`.
- React reports only the item rect. Rust derives the connection anchor from that full rect.

No database, persistence, or settings contract changes are required.

## Contract Changes

Remove `pointerY` from:

- TypeScript `PreviewAnchorRect`.
- Rust `PreviewAnchorRect`.
- Controller pending/retarget state.
- Rust source-rect resolution and its pointer-specific tests.

Clients still send `{ left, top, width, height }`. Rust returns the full intersected item rect, preserving its vertical center for connector placement.

The connector's existing nearest-edge candidate algorithm remains the single owner of choosing the list-side edge. With a full item rect, a right-side panel resolves to the item's right edge at mid-height, while a left-side panel resolves to its left edge at mid-height.

## Pointer Lifecycle

- Keep `pointerenter` for initial delayed open and cross-item retargeting.
- Keep `pointerleave` and document/area leave for hide buffering.
- Delete the preview controller's same-item pointer-move retarget frame and remove the forwarded `pointermove` prop from clipboard cards.
- `onHoverSelect` still runs once on `pointerenter`; same-item movement performs no preview work.

This avoids IPC, payload loading, event emission, and motion updates while the pointer remains within one item.

## Measurement Lifecycle

Replace the conditional ref-based observer setup with a callback-ref/node-state measurement API:

1. The preview attaches the measurement callback only while the measure layer exists.
2. A layout effect binds `ResizeObserver` whenever that node becomes available.
3. The observer stays bound through image loading and reports the final natural panel size.
4. Cleanup disconnects the observer when the node changes or unmounts.

The existing image metadata fast path remains. Callback-ref measurement covers images whose intrinsic size is unavailable initially and single-image file previews.

## First Layout Readiness

The motion layer receives "layout ready" only when payload content exists and the effective panel size has left the fallback size. This prevents the transient empty/loading measurement from clearing the session's jump-only phase before image content arrives. Once content-backed geometry is available, the first panel layout jumps directly to the final size instead of springing from the loading shell.

## Compatibility and Rollback

- No persisted data or migration changes.
- Removing an optional command field is safe because frontend and Rust are shipped together.
- Rollback is limited to restoring `pointerY`, the pointer-move handler, and the previous measurement hook behavior.

## Trade-offs

- Removing hover retargeting means the connector no longer follows the pointer within a card. This is intentional and matches the requested item-centered interaction.
- Callback-ref measurement adds one small state transition when the measure node mounts, but avoids permanently stale observers and per-render node polling.
