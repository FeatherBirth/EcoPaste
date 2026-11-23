# State Management

## Ownership

Valtio stores frontend UI state and mirrors, not durable business data.

Current stores:

- `settingsState`: mirror of Rust `SettingsStore`, updated only from
  `settings://updated`.
- `clipboardViewState`: transient filters for the clipboard window.
- `clipboardStatsState`: small shared display value for the current filtered
  total.
- `sourceAppsState` and `windowLifecycleState`: UI mirrors of command/event
  results.

Clipboard history, groups, apps, backups, settings persistence, and storage
usage remain backend-owned.

## Settings Flow

`src/stores/settings.ts` documents the one-way flow:

```text
component
  -> updateSettings(patch)
  -> Rust SettingsStore persists
  -> Rust emits settings://updated
  -> every WebView listener Object.assigns settingsState
```

Do not optimistically mutate `settingsState` in components or services. The
caller can use the returned snapshot from `updateSettings` if it needs immediate
control flow, but rendering waits for the event mirror.

## Clipboard View State

`clipboardViewState` contains only query/filter state used by the clipboard
window: `category`, `keyword`, `groupId`, and `range`. Keep pagination inside
`useClipboardItems`; do not add `limit` or `offset` to the store.

Be careful with names. `List` maps store fields into `ClipboardItemQuery`, so a
new UI field with the same name as a backend query field can accidentally become
a filter. The store comment calls out `pinned` as a known footgun.

## Server Data

Use command hooks or local component state for command results:

- `useClipboardItems` owns the current virtualized page list.
- Preference panels read the current settings snapshot and commit patches.
- Backup/storage modals fetch their own inspection or usage state.

Promote data to a store only when multiple distant components need the same
small UI mirror and Rust remains the source of truth.

## Clipboard Multi-Selection

The virtual list owns `selectedIndexes`, the anchor, and the focused item.
`selectedIndexes` insertion order is the paste order and the number displayed in
each selected card.
`clipboardSelectionState` mirrors only whether explicit multi-select mode is
active and the selected count for the header control.

- The header list-check button enters/leaves multi-select mode.
- In that mode, ordinary clicks add rows; Cmd/Ctrl-click toggles a row; Shift
  appends not-yet-selected rows from an inclusive range without reordering the
  existing selection.
- Repeating an ordinary click on a selected row removes it.
- Text rows are always batch-selectable. Image/file rows require the experimental
  mixed setting, and files rows whose stored paths no longer all exist are
  unavailable. Unavailable rows are muted and skipped by Shift ranges.
- Native context-menu `select` and the optional configured quick action both
  enter selection mode and add the current row.
- Modifier clicks outside explicit mode enter mode instead of running the
  configured single-click paste/copy action.
- Clear index selection whenever the query changes, the list reloads, a row is
  deleted, or the clipboard window hides/opens.
- Enter/double-click resolves indices in Rust against the current filter, then
  restores the frontend click order; React must not join cached summaries as the
  batch payload.

## Derived State

Prefer deriving display state from snapshots during render. Use `useEffect` only
when another store or an external system must be synchronized, such as updating
`clipboardStatsState.total` from `data.total`.

Do not cache derived copies of `ClipboardItem` arrays in stores. The list already
handles deferred reloads and mutation through the data hook.
