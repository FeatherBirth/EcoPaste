# Technical Design

## Persistent State

Extend `WindowState` with:

- `logicalWidth?: f64`
- `logicalHeight?: f64`

They are optional for backward compatibility. New saves derive them from
physical inner size divided by the scale factor used during capture.

Logical-size resolution:

1. Use explicit logical dimensions when both are finite and positive.
2. Otherwise derive from legacy `width / scaleFactor`.
3. Convert to the target scale with rounding and minimum-size enforcement.

This gives all later corrections a stable logical anchor even if physical-size
events arrive while DPI is changing.

## Position-Aware Restore

Introduce a position-aware restore path used by the clipboard window:

- `Remember`: saved monitor if available; otherwise cursor/current monitor.
- `FollowCursor` and `Center`: cursor monitor first, then current/saved monitor.

The selected monitor supplies the initial physical size conversion. The saved
position is used only for Remember; FollowCursor/Center are positioned by the
existing position module.

Generic secondary windows continue using the existing restore behavior.

## Post-Placement Normalization

After the clipboard window has been positioned:

1. Read its new outer rectangle.
2. Resolve the monitor containing the rectangle.
3. Use that monitor's effective scale factor.
4. Convert the stored logical dimensions to physical size.
5. Clamp to monitor bounds and apply `set_size`.
6. Reapply FollowCursor/Center positioning so the resized window remains
   visible.

This explicit normalization covers programmatic cross-monitor moves that do not
emit `ScaleFactorChanged`.

## Live Move Handling

Keep `ScaleFactorChanged` handling, using the monitor-provided suggested size
where appropriate and preserving the stored logical dimensions.

On `WindowEvent::Moved`, compare the window's actual scale with the last stored
scale before overwriting metrics. If they differ, run the DPI correction first;
then remember the corrected geometry.

## Compatibility

- Existing JSON state files load with optional logical fields absent.
- Existing physical-only state converts from `width/scale`.
- State writes add logical fields; older app versions ignore unknown JSON fields.
- No database or user-settings migration is required.

## Validation Strategy

- Unit-test explicit and legacy logical conversion.
- Unit-test minimum and monitor clamping.
- Build a local package and probe the real clipboard HWND with Win32
  `GetWindowRect` and `GetDpiForWindow` after opening on each monitor.
- Expected probe values:
  - 100%: roughly `372-380 x 606-614` physical at DPI 96.
  - 200%: roughly `744-760 x 1212-1228` physical at DPI 192.
