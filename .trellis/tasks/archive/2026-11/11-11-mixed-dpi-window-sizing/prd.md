# Fix mixed-DPI clipboard window sizing

## Goal

Preserve the clipboard window's intended logical size when it opens on, moves
between, or is restored onto displays with different scale factors, including
Remote Desktop resolution/DPI changes.

## Background

- The current machine has two active displays:
  - `\\.\DISPLAY1`: physical `3120x2080`, effective DPI `192` (200%).
  - `\\.\DISPLAY2`: physical `1920x1080`, effective DPI `96` (100%).
- Clipboard window positioning is `followCursor`.
- Direct window probing reproduced the defect:
  - On the 100% display the clipboard window is about `373x608` physical.
  - Opening it on the 200% display after moving the cursor produces
    `376x609` physical at DPI 192, i.e. only about `188x305` logical.
- The window is moved across monitors after restore. Tauri does not reliably
  emit a scale-factor resize for this programmatic move, so the physical size
  remains unchanged while the WebView DPI doubles.
- `restore_window_state` currently selects the saved monitor for size even when
  `followCursor`/`center` positions the window on a different monitor.

## Requirements

1. Persist an explicit logical width/height alongside physical geometry and
   source scale, while retaining backward compatibility with old state files.
2. Choose the size target monitor from the active positioning policy:
   cursor monitor for FollowCursor/Center, saved monitor for Remember.
3. After positioning the clipboard window, normalize its size using the actual
   destination monitor scale and the preserved logical dimensions.
4. Detect cross-monitor moves where the window's DPI changed but no
   `ScaleFactorChanged` event was delivered, and correct the size.
5. Clamp the final physical size and position to the target monitor.
6. Cover 100% <-> 200%, legacy state, explicit logical state, and live move
   decisions with tests.
7. Build and install a new version, then verify actual window rectangles on the
   current 100% and 200% displays.

## Acceptance Criteria

- [x] Opening on the 100% display preserves approximately `360x600` logical.
- [x] Opening on the 200% display produces approximately `720x1200` physical
      and `360x600` logical.
- [x] Moving between displays in either direction keeps a stable logical size.
- [x] FollowCursor opens on the cursor monitor without using the saved
      monitor's scale for sizing.
- [x] Legacy window-state files without logical dimensions remain readable.
- [x] Remote Desktop scale changes still clamp and preserve logical geometry.
- [x] Rust tests and project quality gates pass.
- [x] A new package is built, installed in place, and visually/API-verified on
      both monitors.

## Out of Scope

- Per-window user-configurable logical dimensions.
- Changing application content zoom independently from the native window size.
- Windows 7 support.

## Technical Notes

- Physical probe sizes include non-client borders while `inner_size()` is
  client-area physical size; acceptance tolerances must allow small differences.
- Do not select a monitor using the stale window current-monitor value after a
  programmatic cross-monitor move; query the monitor containing the new window
  rectangle.

## Validation Notes

- Current layout:
  - Left/main: `3120x2080` physical, DPI 192 (200%).
  - Right/secondary: `2560x1080` physical, DPI 96 (100%).
- Open on 200%: outer window about `794x1233`, logical about `397x617`.
- Open on 100%: outer window about `400x618`, logical about `400x618`.
- Drag 200% -> 100%: corrected to about `410x624`.
- Drag 100% -> 200%: corrected to about `804x1239`, logical about `402x620`.
- Installed version: `1.5.2`.
