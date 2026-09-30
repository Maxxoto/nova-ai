import type { ReactElement } from "react";

import { FOCUS_RING } from "../settings/primitives";

/** The OD bookmark mark (`capture-and-ask.frag`), sized by `.btn svg`. */
function BookmarkGlyph(): ReactElement {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.7"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d="M6 4h12v16l-6-4-6 4Z" />
    </svg>
  );
}

/**
 * "Save to memory" — the panel-footer button from the OD capture-and-ask
 * screen (`.btn .btn-secondary .btn-sm`, core.css:154-175).
 *
 * UI only: the memory write is not wired yet (M2), so the button carries the
 * design's shape and both its states, but pressing it persists nothing.
 */
export default function SaveMemoryButton({ onSaved }: { onSaved?: () => void }) {
  return (
    <button
      type="button"
      title="Not wired yet — saving to memory lands with M2"
      onClick={onSaved}
      className={`panel-save-memory ${FOCUS_RING}`}
    >
      <BookmarkGlyph />
      <span>Save to memory</span>
    </button>
  );
}
