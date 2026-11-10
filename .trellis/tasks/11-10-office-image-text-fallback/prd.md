# Preserve Office image and text clipboard formats

## Goal

Restore native Office/WPS table-paste behavior while retaining an explicit
plain-text path. Default paste/copy must expose both image and text formats so
the destination application can choose, while image history remains
image-oriented and "paste as plain text" writes text only.

## Background

- Excel/WPS table selections commonly expose image, HTML/RTF, and plain-text
  representations at the same time.
- Before the Windows issue hotfix, EcoPaste stored the image representation and
  WeChat pasted an image, but plain-text editors could not use the table text.
- The current hotfix in `clipboard/read.rs` forces Office text and therefore
  regresses applications that should receive the image.
- `ClipboardItem.search_text` already exists, so the fallback text can be stored
  without a database schema migration.
- The user's settings use default image paste, middle-click plain-text paste,
  and image preview. The implementation must preserve those paths.

## Requirements

1. Keep Office/WPS mixed clipboard captures as `ClipboardKind::Image` when image
   capture is enabled and encountered first in the configured capture order.
2. Preserve the source plain-text representation in the image record as a
   fallback, without displaying it as the image card or preview body.
3. Normal paste/copy of an image record with fallback text writes both image and
   text clipboard formats. Applications can choose image or text natively:
   WeChat should receive the image; VSCode should receive text.
4. An explicit plain-text paste/copy writes only the fallback text. This covers
   the context menu, middle click, Enter shortcut payloads that request plain
   text, and configured plain-text quick actions.
5. Expose `pasteAsPlainText` for image records only when a non-empty fallback
   text exists.
6. Keep image-only records unchanged: no text action and no additional format.
7. Do not expose full fallback text through ordinary list IPC; compute actions
   in Rust, then omit the image fallback from the returned list model.
8. Preserve existing settings, database schema, and user data.

## Acceptance Criteria

- [x] Office/WPS image+text capture produces an image history record with the
      plain text stored as its searchable fallback.
- [x] The image card and preview continue to render the image.
- [x] Default paste/copy writes a multi-format clipboard containing both the
      image and fallback text when fallback text exists.
- [x] WeChat-style image-capable targets can choose the image, while plain-text
      editors can choose text from the same clipboard payload.
- [x] Context-menu plain-text paste and middle-click plain-text paste write only
      the table text for an Office image record.
- [x] Plain-text actions are hidden for ordinary images without fallback text.
- [x] Existing image-only copy/paste behavior remains unchanged.
- [x] `cargo fmt --check`, Clippy with warnings denied, Rust tests, `pnpm lint`,
      `pnpm tsc`, and the production build pass.
- [x] A real Windows clipboard smoke test confirms that image and text formats
      coexist after default writeback.

## Out of Scope

- App-specific target detection for choosing image versus text; destination
  applications choose from the multi-format clipboard themselves.
- OCR, spreadsheet cell reconstruction, or conversion between image and text.
- Database migration or rewrites of historical records.

## Technical Notes

- Source detection remains limited to the existing Office/WPS executable list.
- `search_text` doubles as the plain-text fallback for image records; list IPC
  must strip it after available actions are computed.
- Windows writeback must avoid clearing the opened clipboard when adding text
  after the image format. macOS can use the clipboard library's multi-content
  write API.
