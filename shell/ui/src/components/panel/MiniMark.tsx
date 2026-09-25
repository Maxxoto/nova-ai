import { useRef } from "react";
import { FOCUS_RING } from "../settings/primitives";
import { invokeTauriAsync } from "../../tauri";
import { canonicalStateOf, type PanelState } from "./types";

/**
 * The mark's word per capture phase (design `LABEL`, lower-cased). The canonical
 * state collapses transcribing/thinking/streaming into one `thinking` loop, but
 * the word stays the real phase so the state remains legible under reduced motion.
 */
const WORD: Record<PanelState, string> = {
  idle: "ready",
  ask: "ready",
  listening: "listening",
  transcribing: "transcribing",
  thinking: "thinking",
  speaking: "speaking",
  streaming: "writing",
  complete: "complete",
  error: "error",
  degraded: "resting",
};

export default function MiniMark({
  state,
  onOpen,
  reducedMotion = false,
}: {
  state: PanelState;
  onOpen: () => void;
  reducedMotion?: boolean;
}) {
  const canonical = canonicalStateOf(state);
  const word = WORD[state];
  const answered = canonical === "answered";
  const title = `Ruoxi · ${word}${answered ? " — open to read" : ""}`;

  const pressRef = useRef<{ x: number; y: number } | null>(null);

  /* The root must not be a `<button>`: Tauri's drag region ignores interactive
     elements, so a button can never start a native window drag. A drag-region
     press can swallow `click`, so release expands only when the pointer moved
     < 4px; `onClick` stays as the fallback (idempotent if both fire). */
  return (
    <div
      role="button"
      tabIndex={0}
      data-state={canonical}
      data-phase={state}
      data-icon="circle"
      data-tauri-drag-region
      onMouseDown={(e) => {
        pressRef.current = { x: e.screenX, y: e.screenY };
        invokeTauriAsync("begin_panel_drag")?.catch(() => undefined);
      }}
      onMouseUp={(e) => {
        const press = pressRef.current;
        pressRef.current = null;
        if (press && Math.hypot(e.screenX - press.x, e.screenY - press.y) < 4) onOpen();
      }}
      onMouseLeave={() => {
        pressRef.current = null;
      }}
      onClick={onOpen}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          onOpen();
        }
      }}
      aria-label={`Ruoxi — ${word}. Open the panel.`}
      title={title}
      className={`mini grid h-11 w-11 place-items-center border-0 bg-transparent p-0 text-primary-foreground ${FOCUS_RING}${
        reducedMotion ? " reduced-motion" : ""
      }`}
    >
      {canonical === "thinking" ? (
        <span className="mini-spin" aria-hidden="true" data-tauri-drag-region />
      ) : null}
      <span className="mini-mark" aria-hidden="true" data-tauri-drag-region>
        <span className="mini-glyph">
          <svg
            data-glyph="circle"
            data-tauri-drag-region
            viewBox="0 0 16 16"
            fill="none"
            stroke="currentColor"
            strokeWidth={2.6}
            className="h-4 w-4"
          >
            <circle cx="8" cy="8" r="5.4" />
          </svg>
        </span>
      </span>
      {canonical === "thinking" ? (
        <span className="mini-dots" aria-hidden="true" data-tauri-drag-region>
          <i />
          <i />
          <i />
        </span>
      ) : null}
      {canonical === "listening" ? (
        <span className="mini-bars" aria-hidden="true" data-tauri-drag-region>
          <i />
          <i />
          <i />
        </span>
      ) : null}
      {answered ? (
        <span className="mini-badge" aria-hidden="true" data-tauri-drag-region>
          <svg viewBox="0 0 16 16" aria-hidden="true">
            <path
              d="M8 1.6a4.6 4.6 0 0 0-2.7 8.3c.4.3.6.7.6 1.2v.3h4.2v-.3c0-.5.2-.9.6-1.2A4.6 4.6 0 0 0 8 1.6Z"
              fill="currentColor"
            />
            <path
              d="M6.4 12.9h3.2M7 14.5h2"
              stroke="currentColor"
              strokeWidth={1.3}
              strokeLinecap="round"
            />
          </svg>
        </span>
      ) : null}
    </div>
  );
}
