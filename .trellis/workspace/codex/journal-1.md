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
