# Technical Design

## Data Model

No schema migration is needed.

- `ImagePayload` gains `plain_text: Option<String>`.
- For a mixed Office/WPS capture, the plain-text representation is copied into
  that field before image storage.
- `build_item_with_settings` maps it to `ClipboardItem.search_text` while keeping
  `kind = image`, `content = <hash>.png`, and `summary = null`.
- FTS therefore indexes the table text without changing the card kind.

## Capture Flow

1. Detect the source executable with the existing Office/WPS matcher.
2. Pre-read the text family only for Office/WPS sources.
3. Walk the configured capture order as before.
4. When image capture wins, attach the non-empty source plain text to the image
   payload and return the image record.
5. When image is disabled or text appears earlier in a custom user order, retain
   the existing text-record behavior.

This restores the pre-hotfix image default without discarding the fallback text.

## Writeback Modes

Image records have three writeback modes:

- `plain = true` and fallback text exists: write text only.
- `plain = false` and fallback text exists: write image plus text.
- No fallback text: retain image-only behavior.

On macOS, use `ClipboardContext::set` with image and text contents.

On Windows:

1. Use the existing clipboard-rs image writer for PNG/DIB/Bitmap formats.
2. Reopen the clipboard through `clipboard-win`.
3. Append Unicode text with `NoClear`.

This avoids clearing the image when adding text and mirrors the format
combination produced by Excel.

The writeback guard suppresses both the image hash and fallback-text hash.

## Actions and IPC

`compute_available_actions` adds `PasteAsPlainText` to an image record only when
`search_text` is a non-empty fallback.

The list and single-list-item command paths compute actions before clearing
image `search_text`, preventing full table text from being sent through routine
list IPC. Full writeback/preview commands continue loading the stored value.

No frontend action model change is required because
`pasteAsPlainText` is already mapped to `pasteClipboardItem(id, true)`.

## Compatibility

- Existing settings and database files remain valid.
- Existing image records without fallback text behave exactly as before.
- Existing text records from the previous hotfix remain text records; future
  copies create or reuse image records under the new behavior.
- Default preview and Enter/default-click paste remain image-oriented.

## Risks

- Destination applications ultimately choose which clipboard flavor to use.
  Providing both formats is the native contract and avoids app-specific
  heuristics.
- Windows clipboard APIs are global and may be briefly locked by another
  process; use the existing bounded retry behavior.
