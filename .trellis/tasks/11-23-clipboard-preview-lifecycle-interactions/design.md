# Preview Lifecycle and Interaction Design

## Architecture and Boundaries

React owns:

- current preview intent and item/session state;
- the actual visible panel rect;
- text selection and image zoom state;
- interaction enter/leave events rendered by the preview page.

Rust owns:

- request ordering and stale-show rejection;
- overlay placement and visibility;
- global cursor observation while a preview is visible;
- temporarily enabling mouse input for the real panel only;
- platform focus activation/restoration.

## Request Ordering Contract

Add a frontend-generated monotonic `requestId` to both show and close commands:

```text
show_clipboard_preview(itemId, anchor, requestId)
close_clipboard_preview(requestId)
```

Rust stores the highest request id observed. A command whose id is lower than the stored value is a no-op. This makes concurrent IPC safe:

- show(new) then close(newer) -> preview opens and closes;
- close(newer) arrives before show(old) -> show(old) is rejected and cannot revive the preview;
- close(newer) then show(same/newer) follows numeric ordering deterministically.

The frontend continues to increment the same request id before every open/close intent. `PreviewSession` also carries `requestId`, so interaction events from an obsolete session are ignored.

## Residual Preview Fix

Two independent protections are added:

1. Rust request-id gating prevents an in-flight show from reopening after a newer close.
2. The list area observes pointer position over card boundaries. When the pointer is over list background or a card that should not preview, the current hover session enters the normal hide buffer instead of waiting for another preview trigger.

The existing 240ms item-to-item buffer remains. `handleItemPointerEnter` still cancels it when a new previewable item is entered.

## Interactive Panel Activation

The preview overlay remains full-monitor sized and click-through by default.

The preview page reports its actual dynamic panel rect to Rust whenever the panel rect changes:

```text
set_clipboard_preview_panel_rect(requestId, rect)
```

While the preview session is visible, Rust polls the global cursor at animation-frame cadence:

- cursor inside physical panel rect -> set `ignore_cursor_events(false)` and activate preview interaction;
- cursor outside panel rect -> set `ignore_cursor_events(true)` and deactivate interaction.

The poll is bounded to the current preview request/session and stops when the preview closes or a newer request replaces it. The actual panel rect avoids treating the full 480px placement allowance as interactive.

Rust emits:

```text
preview://interaction { sessionId, active }
```

The clipboard-window preview controller listens to this event:

- `active=true` cancels/suspends hover hide timers;
- `active=false` schedules the normal hover hide;
- events with a non-current session id are ignored.

While `active=true`, the controller also suppresses preview closure caused by clipboard-window blur from the focus handoff. Preview closure while the panel is active is allowed only through explicit actions such as Escape, list/window hide, clipboard mutation, or a newer preview request.

## Focus and Copy

On entering the active panel, Rust makes the preview window focusable but does not permanently change the target application:

- Windows stores the previous foreground HWND, focuses the preview WebView, and restores the previous foreground when interaction ends or preview closes.
- macOS changes the preview panel to allow key-window status and uses nonactivating panel focus so the underlying app is not brought forward.

Text selection needs native keyboard focus; image scrolling and wheel zoom work once the panel receives mouse events. If focus restoration is rejected by Windows, log the condition and keep the preview close path intact.

## Text Viewer

Replace the virtual-row text viewer with one read-only `<textarea>`:

- native scrolling, selection, select-all, and copy;
- `select-text`, no spellcheck, and soft wrapping;
- transparent styling using Ant Design tokens;
- height follows the estimated wrapped content, with the clipboard window height
  as the upper bound;
- `overflow-auto` so the scrollbar appears only when content actually exceeds the panel.

This intentionally trades virtual DOM for stable native selection. The existing
capture size setting remains the performance guard. The 32-character row
estimate is used only for layout bounds; short content shrinks the panel and
long content caps at the clipboard-window height before scrolling.

## Image Viewer

Keep zoom state keyed by payload id in the preview page:

- default scale is fit-to-panel (`1.0` in this local scale space);
- wheel changes scale around the cursor, clamped to `0.25..=4.0`;
- panel width/height and the image grow or shrink together, clamped by the
  clipboard-window-sized panel limits;
- payload/session change resets zoom to fit;
- scrolling the image container must not close the preview.

Image zoom uses the full monitor margin-inset overlay as its outer limit rather
than the clipboard-window width, and applies one uniform scale to both axes.
The visible motion panel must not keep a fixed `max-width` or `max-height`;
only the hidden measurement layer retains fallback dimensions.

## Windows Outside-Click Exclusion

The clipboard window auto-hide hook runs independently of the preview WebView
focus path. A click inside the current preview panel looks like an outside click
relative to the clipboard window. The mouse hook must check the reported preview
panel physical rect first and leave the clipboard window visible for that click.

For left/right placement, the connector source is the clipboard-window outer
edge at the item's vertical center, not the virtual card edge. This keeps the
dot outside the card and independent of the card's horizontal list padding.

The source rect sent by React now uses the full-width list row for `left` and
`width`, while retaining the card rect's vertical center. Rust still maps that
viewport rect into overlay coordinates, but connector geometry uses only the
mapped row rect. Each endpoint dot is offset outward by its radius, so both
sides are tangent to the visual borders and do not overlap either surface.
The right list source point receives an additional 18px inward compensation for
the overlay scrollbar and border; the confirmed left anchor remains unchanged.

## Text First Paint

The textarea starts with `overflow: hidden` while the panel spring is still
moving from its fallback size to the measured height. The unmeasured estimate
includes a 10% safety factor. After the preview transition settles, overflow
switches to `auto`; only genuinely overflowing text then shows a scrollbar.

## Windows Precision Touchpad Gesture

The preview WebView and its child HWND tree request `GID_ZOOM`. A window
subclass consumes zoom gestures only while the gesture location is inside the
current preview panel, then emits `preview://gesture` with a per-frame factor.
React applies that factor to the existing whole-panel zoom state. Pointer events
for two physical contacts remain a fallback, while `Ctrl + wheel` remains
available for device/WebView2 paths that already translate pinch to wheel.

No Rust clipboard contract is needed for zoom because it only affects preview presentation.

## Cross-Platform and Compatibility

- No persisted data changes.
- Windows and macOS both use `set_ignore_cursor_events`/panel-equivalent toggling.
- Keyboard Space preview remains click-through unless the pointer enters the panel.
- Existing placement, connector, first-layout, and exit-animation contracts remain unchanged.
- If global cursor polling cannot start, preview remains usable in the existing click-through mode; interaction enhancement degrades without blocking the lifecycle fix.
- If focus activation fails, mouse scrolling and selection remain available; keyboard copy is the only capability that may require the user to click the panel again.

## Key Risks and Mitigations

- Focus stealing: activate only after pointer enters the actual panel and restore the previous foreground on exit.
- Full-window click interception while active: poll and disable input immediately when the cursor leaves the panel.
- Large text responsiveness: preserve the configured max text capture size and avoid creating HTML DOM.
- Request-id reset: ids are process-local and used only for in-flight ordering, not persisted identity.

