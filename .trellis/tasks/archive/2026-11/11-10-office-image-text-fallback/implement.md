# Implementation Plan

## Ordered Changes

1. Extend `ImagePayload` with optional source plain text and update constructors.
2. Replace the Office/WPS text-forcing branch with image-first capture plus
   fallback-text attachment.
3. Persist the fallback in `ClipboardItem.search_text` while keeping image kind
   and image preview.
4. Backfill fallback text on image de-duplication when an older image row lacks
   it.
5. Add image writeback modes for text-only and image-plus-text.
6. Implement Windows multi-format image/text writeback with one clipboard open.
7. Expose plain-text actions for image records with fallback text and strip the
   fallback from list IPC after actions are computed.
8. Add focused tests for capture decisions, item construction, action
   availability, and writeback-mode selection.
9. Update clipboard pipeline specs with the mixed-format contract.
10. Run formatting, Clippy, Rust tests, TypeScript, Biome, and production build.
11. Perform a Windows clipboard smoke test and inspect that both image and text
    formats are readable after default writeback.
12. Commit and push the completed task.

## Validation

- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- `pnpm lint`
- `pnpm tsc`
- `pnpm build`
- Windows smoke test using a generated image with fallback text:
  verify image flavor, `CF_UNICODETEXT`, and plain-text-only writeback.

## Risk and Rollback

- Clipboard writeback touches OS state; preserve existing behavior for records
  without fallback text.
- If Windows multi-format writeback cannot retain both formats, stop and revise
  before shipping rather than silently falling back to text-only.
- No persisted data migration is required, so rollback is code-only.
