import { proxy } from "valtio";

/**
 * Cross-component multi-selection UI state. Selected row indices remain local
 * to the virtual list; this mirror only drives the header control.
 */
export const clipboardSelectionState = proxy({
  active: false,
  count: 0,
});

/**
 * Enter or leave explicit multi-selection mode.
 */
export function setClipboardSelectionMode(active: boolean) {
  clipboardSelectionState.active = active;
  if (!active) {
    clipboardSelectionState.count = 0;
  }
}

/**
 * Keep the header selection count aligned with the list-owned index set.
 */
export function setClipboardSelectionCount(count: number) {
  clipboardSelectionState.count = count;
}
