# Cross-Feature Integration Design

## 1.6.0 GitHub Release and Updater Integration

- `package.json.version` remains the release source; Cargo metadata is updated
  with it for the desktop binary.
- The updater public key is the fork-specific signing key. The private key exists
  only as a GitHub Actions secret and in the local release-signing directory for
  this session.
- GitHub Actions builds each platform with `includeUpdaterJson: false`, uploads
  the platform installer plus Tauri `.sig`, then a single final job downloads all
  signatures, reads the release assets, and writes one `latest.json` containing:
  `windows-x86_64`, `windows-aarch64`, `darwin-x86_64`, and `darwin-aarch64`.
- The public endpoint is the repository's stable latest URL, so moving to a new
  release requires only publishing a newer stable GitHub Release.
- Windows updater HTTP requests explicitly use the current user's WinINet proxy
  when enabled. GitHub access through the normal system proxy therefore works
  without requiring global environment variables.
- The legacy release server, beta/nightly channel switches, donation QR, and
  tray source link are intentionally removed. The About page links to the
  current repository, and update release notes link to the matching current
  repository release.
- The update proxy defaults to System. Direct mode bypasses all proxies; Custom
  mode validates and persists an HTTP(S) proxy URL.
- The icon generator extracts the original white P silhouette, preserves its
  geometry, and only refreshes the background treatment. NSIS header/sidebar
  images are generated from the same icon.

## Architecture and Boundaries

The request is split into three independently verifiable child tasks:

1. `clipboard-preview-lifecycle-interactions`
2. `clipboard-multiselect-batch-paste`
3. `image-clipboard-format-repair`

Each child owns its full Rust/React/platform contract. This parent task owns only:

- source requirements and cross-child acceptance;
- shared settings/preference/i18n consistency review;
- clipboard writeback and loop-suppression integration review;
- final Windows desktop and macOS build verification.

## Shared Contracts

The children intentionally touch separate settings fields:

- preview interaction needs no persisted setting;
- mixed batch paste needs `clipboard.content.experimentalMixedBatchPaste`;
- Windows image repair needs `clipboard.content.repairPureImageClipboardFormat`.

Both content settings default to `false` and rely on serde defaults for upgrade compatibility.

Shared settings files are expected to be touched sequentially rather than in parallel:

- `src-tauri/src/settings/model.rs`
- `src/types/settings.ts`
- `src/pages/Preference/config/preferenceSchema.ts`
- `src/pages/Preference/components/settingControls/settingVisual.ts`
- `src/locales/zh-CN/preferences.json`
- `src/locales/en-US/preferences.json`

## Clipboard Writeback Interaction

Batch paste and image repair both call existing clipboard writeback paths:

- batch text and individual items must register `WritebackGuard` hashes;
- image repair must register the existing image content hash;
- Office image records must keep their image + text behavior;
- no new writeback path may clear or replace clipboard content after an unrelated newer copy.

The parent review checks that:

- a batch paste does not create a new history row from its own joined text;
- an image repair does not create a duplicate image row;
- Excel/WPS mixed image+text retains both image and text representations.

## Migration and Rollout

- No SQLite schema migration is expected.
- Settings are additive JSON fields with defaults, so no explicit migration file is required.
- Preview command changes are shipped atomically with frontend and Rust.
- Image repair is Windows-only and defaults off, limiting rollout risk.
- Mixed batch paste is experimental and defaults off.

## Rollback Shape

- Preview behavior can roll back independently by restoring request-id-free show/close and click-through-only window treatment.
- Multi-select can roll back by restoring the single `selectedId` click flow and hiding the experimental setting.
- Image repair can roll back by ignoring the setting and removing the Windows repair call; stored history and image resources remain valid.

## Cross-Child Acceptance

- Build all frontend and Rust targets with lint/type/clippy/test gates.
- Run UI on Windows for preview, selection, mixed paste, and screenshot repair.
- Run macOS build/checks for preview and multi-select; verify no Windows-only repair UI is exposed.
- Inspect actual clipboard formats for Excel/WPS and pure PNG image cases.
- Confirm no generated data migration is required for existing installations.

