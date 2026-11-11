# Implementation Plan

## Ordered Changes

1. Add explicit logical dimensions to `WindowState` with legacy fallback.
2. Add pure helpers for logical-size resolution and physical conversion.
3. Make clipboard restore position-aware for Remember, FollowCursor, and Center.
4. Normalize logical size against the monitor containing the final window
   rectangle after positioning.
5. Harden Moved/ScaleFactorChanged handling against event-order races.
6. Add regression tests for 100% <-> 200% and legacy state.
7. Update the Remote Desktop/windowing spec.
8. Run Rust/frontend quality gates and build NSIS.
9. Install the new package and probe the real clipboard window on both displays.
10. Commit and push the completed fix.

## Validation

- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- `pnpm lint`
- `pnpm tsc`
- `pnpm package:windows`
- Win32 rectangle/DPI probes on DISPLAY1 and DISPLAY2.
- Verify settings and clipboard history remain present.

## Risk and Rollback

- Changing restore monitor priority affects clipboard window placement only.
- Secondary windows retain their existing restore path.
- If explicit logical fields conflict with physical fields, logical fields win.
- Rollback is code-only; extra JSON fields remain safely ignored by older code.
