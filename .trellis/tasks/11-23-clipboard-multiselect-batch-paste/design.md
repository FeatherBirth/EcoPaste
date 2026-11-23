# Multi-Select and Batch Paste Design

## Architecture and Boundaries

React owns transient selection intent:

- focus/active index;
- explicit `selectionMode` from the header control;
- selection anchor index;
- selected index set;
- modifier-click interpretation;
- selected-state rendering.

Rust owns durable query semantics and clipboard operations:

- resolving selected list indices against the current filter/sort;
- reading complete item content;
- constructing single or sequential clipboard payloads;
- hiding/focus handoff and paste simulation;
- reuse counters and writeback suppression.

Selection indices are never treated as durable row identifiers. They are only valid for the current query snapshot and are cleared whenever the list query changes.

## Selection State

Replace the single multi-selection decision with:

```ts
selectedId: string | null;        // current focus item
selectionAnchorIndex: number | null;
selectedIndexes: Set<number>;     // all explicitly selected rows
```

`selectedId` remains the keyboard focus and single-item action target. `selectedIndexes` expresses multi-selection:

- outside selection mode, normal click clears the set and keeps historic
  single-click paste/copy;
- in selection mode, normal click adds the row and does not paste;
- a repeated normal click on a selected row toggles it off;
- Ctrl/Cmd+click: toggle target index, set focus and anchor to target;
- Shift+click: replace selection with anchor..target when no Ctrl/Cmd;
- Ctrl/Cmd+Shift+click: add anchor..target to the existing set;
- modifier clicks never run single-click paste/copy;
- no visible selection anchor uses the current focused/visible item as fallback.

When `selectedIndexes.size > 1`, Enter and double-click invoke batch paste. A normal click on an item already inside a multi-selection uses a short single-click defer so the second click can become a batch double-click; if no double-click follows, the original single-click action executes after the defer.

Selection is cleared on:

- query/filter/category/group/sort change;
- window open reset where the list itself is reset;
- deleted items;
- leaving the clipboard window (fresh session on reopen).

Availability:

- text rows are always batch-selectable;
- image/file rows require the experimental mixed setting;
- a files row is selectable only when every stored path exists;
- unavailable rows are visually muted and skipped by Shift ranges.

The native context menu and optional configured quick action both expose
`select`, which enters selection mode and adds the target row.

## Rendering

Keep the existing `bg-ant-blue-1` and `border-ant-primary` selected treatment.

`selectedId` remains a stronger focus state:

- selected range: blue background/border;
- focus item: retain existing border/primary treatment;
- non-focused selected items must still be clearly distinguishable.

No checkbox or separate selection-mode UI is added.

## Batch Query Contract

Add a Rust command:

```text
paste_clipboard_items(query, selectedIndexes, mixed)
```

The command returns a compact result:

```ts
interface BatchPasteResult {
  completed: number;
  total: number;
  failedIndex: number | null;
  failedItemId: string | null;
}
```

`query` is the same filter/sort object used by `useClipboardItems`, with `limit`/`offset` ignored by the batch path. The frontend sends ordered unique selected indices.

Repository support:

- compress selected indices into contiguous ranges;
- for each range, query complete `SELECT_ITEM` rows with the same filter and sort using the range's offset/limit;
- verify the returned row's zero-based position corresponds to each requested index;
- concatenate in display order.

This avoids requiring all virtualized rows to be loaded in React.

## Single Payload Path

If all selected rows are text-family (`ClipboardKind::Text`):

- use `search_text` first, falling back to `content`;
- trim? Do not trim content; preserve the stored plain-text representation;
- join with exactly `\n`;
- register the joined hash with `WritebackGuard`;
- write one Unicode text clipboard payload;
- hide/resign the clipboard window once;
- simulate one paste;
- update reuse counters for each item only when the existing setting is enabled.

This path is used regardless of the experimental mixed setting.

## Mixed Sequential Path

If the selection contains image/files and `experimentalMixedBatchPaste = true`:

1. Segment ordered items into maximal contiguous text runs and individual non-text items.
2. Text runs use the single-payload join algorithm.
3. Image/files use the existing `write_to_clipboard` semantics, including `pastePlain` and `pasteFilesAsPath`.
4. Write and simulate paste once per segment in display order.
5. Wait a fixed short interval (initial recommendation: 180ms) between segments for the foreground app to consume the paste.
6. Stop immediately on a write/paste/update failure and return `completed`, `total`, and the failed segment's lead item.
7. Do not roll back already pasted external-app content.

The command owns the full sequence. React does not issue multiple invokes, which avoids IPC reordering and enables structured partial-failure reporting.

Validation errors such as an empty/out-of-range selection reject the command. A mixed sequence that starts successfully and later fails returns `Ok(BatchPasteResult)` with `failedIndex`/`failedItemId`, allowing the wrapper to show a localized partial-failure message.

## Experimental Setting

Add:

- Rust `Content.experimental_mixed_batch_paste: bool = false`;
- TypeScript `Content.experimentalMixedBatchPaste: boolean`;
- preference control in the Paste section with `status: "experimental"`;
- zh-CN/en-US title, description, and search keywords.

The setting controls only mixed paste. Text-only batch remains available when the switch is off.

## Compatibility

- No SQLite schema changes.
- `selectedId` continues to drive preview and existing single-item shortcuts.
- Existing single-click/copy/paste, double-click, middle-click, menu, drag, favorite, pin, note, and delete paths remain unchanged.
- Multi-select copy (Ctrl/Cmd+C) supports text-only selections by writing the same joined text without simulating paste; mixed copy is not required in MVP.

## Risks

- Sequential mixed paste can interleave with user input; the feature is explicitly experimental and stops on the first failure.
- Index-based selection becomes invalid if the list changes; query-change and deletion handlers must clear it.
- The delayed single click for multi-selected items must be canceled by double-click, Escape, query reset, and component unmount.
- Batch text can exceed clipboard/application limits; use the existing text-size limits at capture time and return a clear failure if writeback fails.

