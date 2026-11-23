# Windows Pure-Image Clipboard Repair Design

## Architecture and Boundaries

The repair belongs in Rust:

- `clipboard/read.rs` captures enough Windows format metadata to classify pure image state;
- `clipboard/repair.rs` owns Windows format inspection and rewrite;
- `clipboard/watcher.rs` invokes repair only after successful persistence;
- settings and preference UI expose a Windows-only default-off switch.

React does not inspect clipboard formats and does not trigger repair.

## Detection Contract

Extend the internal image payload with Windows-only metadata:

```text
source_sequence: Option<u32>
repair_candidate: bool
```

`repair_candidate` is true only when all are true:

- the event was classified as `ClipboardPayload::Image`;
- the image has no Office/WPS plain-text fallback and no source text/html/RTF/HDROP format;
- the Windows clipboard exposes a PNG image representation;
- none of `CF_BITMAP`, `CF_DIB`, or `CF_DIBV5` is available.

The concrete failing case is therefore “PNG-only image”: EcoPaste can read it, but applications that request a native Windows bitmap cannot paste it until EcoPaste rewrites standard image formats.

No other capture kind is eligible.

## Office/WPS Protection

Office/WPS mixed table selections already produce:

- `ClipboardKind::Image`;
- non-empty `search_text` fallback;
- image + text writeback;
- text-only writeback for plain mode.

Repair explicitly rejects any payload with:

- a non-empty plain-text fallback;
- `CF_UNICODETEXT`;
- `HTML Format`;
- `Rich Text Format`;
- `CF_HDROP`.

This keeps Excel/WPS behavior independent of the repair setting.

## Race and Sequence Protection

Windows clipboard changes are asynchronous and users may copy another value while the image is being persisted.

Capture `GetClipboardSequenceNumber` before reading the payload. Before rewrite:

1. compare the current sequence number to the captured value;
2. if changed, skip repair;
3. re-check that the clipboard is still a PNG-only pure image;
4. rewrite only if both checks still pass.

This prevents stale repair tasks from overwriting a newer clipboard value.

## Rewrite Contract

After history persistence succeeds:

1. read the stored PNG through `ImageStore`;
2. decode to `RustImageData`;
3. register the existing image `content_hash` with `WritebackGuard`;
4. use the existing Windows image writer, which publishes PNG plus `CF_BITMAP`;
5. do not add text unless the image already has Office fallback text;
6. log inspection/rewrite failures without failing ingestion.

The repair must not mutate pixels or stored history metadata.

## Settings

Add:

- Rust `Content.repair_pure_image_clipboard_format: bool = false`;
- TypeScript `Content.repairPureImageClipboardFormat: boolean`;
- preference schema entry only under the Windows branch;
- zh-CN/en-US copy explaining it is a Windows compatibility option;
- search keywords: screenshot, paste, image, clipboard, repair.

macOS still deserializes the field harmlessly but no code path reads it for repair and no UI entry is shown.

## Verification

Unit-test the pure classifier with a small format snapshot rather than requiring clipboard access:

- PNG + no bitmap formats + no text -> candidate;
- PNG + CF_DIB -> no repair;
- PNG + CF_BITMAP -> no repair;
- image + Office text -> no repair;
- image + HTML/RTF only -> no repair;
- files/HDROP -> no repair;
- setting off -> no inspection/rewrite.

Ignored desktop-session tests can construct a PNG-only clipboard format and assert rewrite adds a native bitmap format while the watcher guard suppresses the resulting event.

