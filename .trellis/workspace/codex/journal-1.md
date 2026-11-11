# Journal - codex (Part 1)

> AI development session journal
> Started: 2026-11-08

---



## Session 1: Windows issues, Remote Desktop DPI, and warm menus

**Date**: 2026-11-08
**Task**: Windows issues, Remote Desktop DPI, and warm menus
**Branch**: `codex/windows-remote-dpi-fixes`

### Summary

Fixed Windows keyboard/clipboard/autostart/storage issues, added scale-aware Remote Desktop window geometry and DPI monitoring, retained context menus warm after first use, built the NSIS installer, and documented concise backend contracts. Machine-wide replacement remains pending UAC approval; tests and package build pass.

### Main Changes

(Add details)

### Git Commits

| Hash | Message |
|------|---------|
| `0986cbf` | (see git log) |
| `58fbb9d` | (see git log) |

### Testing

- [OK] (Add test results)

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 2: Remote desktop clipboard DPI recovery

**Date**: 2026-11-10
**Task**: Remote desktop clipboard DPI recovery
**Branch**: `codex/clipboard-filter-preview-refinements`

### Summary

Fixed a Remote Desktop DPI restore regression where legacy physical-only clipboard geometry opened at half size; enforced the 360x600 logical minimum during restore and live scale changes, documented the invariant, and validated Rust/frontend quality gates.

### Main Changes

(Add details)

### Git Commits

| Hash | Message |
|------|---------|
| `7f79cad` | (see git log) |

### Testing

- [OK] (Add test results)

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 3: Office image and text clipboard fallback

**Date**: 2026-11-10
**Task**: Office image and text clipboard fallback
**Branch**: `codex/clipboard-filter-preview-refinements`

### Summary

Restored native Office/WPS table paste semantics by keeping image records with an alternate plain-text representation, writing image plus text by default, exposing plain-text paste only when fallback text exists, and preserving image-only behavior. Unit, integration, Windows clipboard smoke, frontend, and build checks passed.

### Main Changes

(Add details)

### Git Commits

| Hash | Message |
|------|---------|
| `9cdba05` | (see git log) |

### Testing

- [OK] (Add test results)

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 4: Windows in-place upgrade and startup hardening

**Date**: 2026-11-10
**Task**: Windows in-place upgrade and startup hardening
**Branch**: `codex/clipboard-filter-preview-refinements`

### Summary

Added per-machine NSIS updates, preinstall/preuninstall process-tree cleanup, passive /UPDATE /R install scripts, and version 1.5.1. Verified repeated in-place upgrades, one responsive process, version/registry metadata, normal startup logging, and Alt+C shortcut activation.

### Main Changes

(Add details)

### Git Commits

| Hash | Message |
|------|---------|
| `5fd0f63` | (see git log) |

### Testing

- [OK] (Add test results)

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 5: GitHub one-command Windows installer

**Date**: 2026-11-10
**Task**: GitHub one-command Windows installer
**Branch**: `codex/clipboard-filter-preview-refinements`

### Summary

Published v1.5.1 stable Windows release assets and a checksum-verified PowerShell bootstrap installer. Added release workflow uploads, bilingual documentation, negative checksum testing, public latest URL verification, and a successful end-to-end remote in-place upgrade.

### Main Changes

(Add details)

### Git Commits

| Hash | Message |
|------|---------|
| `a513ea4` | (see git log) |
| `48c5e7b` | (see git log) |

### Testing

- [OK] (Add test results)

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 6: Mixed-DPI clipboard window sizing

**Date**: 2026-11-11
**Task**: Mixed-DPI clipboard window sizing
**Branch**: `codex/clipboard-filter-preview-refinements`

### Summary

Added explicit logical window dimensions, position-aware restore, post-placement monitor normalization, and moved-event DPI correction. Verified 200% to 100% and reverse drags on real displays; released v1.5.2 with stable GitHub installer assets and checksum-verified PowerShell bootstrap.

### Main Changes

(Add details)

### Git Commits

| Hash | Message |
|------|---------|
| `90d45b6` | (see git log) |

### Testing

- [OK] (Add test results)

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 7: Stabilize clipboard hover previews and publish v1.5.3

**Date**: 2026-11-11
**Task**: Stabilize clipboard hover previews and publish v1.5.3
**Branch**: `codex/clipboard-filter-preview-refinements`

### Summary

Stabilized same-item hover previews, centered connector anchors on item edges, fixed first-image measurement, built the Windows NSIS package, and published GitHub Release v1.5.3 after adapting release CI for fork authentication and unsigned fallback builds.

### Main Changes

(Add details)

### Git Commits

| Hash | Message |
|------|---------|
| `14315e4` | (see git log) |
| `4161ae8` | (see git log) |
| `7aa79ab` | (see git log) |
| `1ae8ee2` | (see git log) |
| `96f10c4` | (see git log) |

### Testing

- [OK] (Add test results)

### Status

[OK] **Completed**

### Next Steps

- None - task complete
