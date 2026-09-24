import { FOCUS_RING } from "../settings/primitives";
import type { PanelState } from "./types";

type MiniVisual = "ready" | "listening" | "working";

/**
 * The mark's word per capture phase (design `LABEL`, lower-cased). The visual
 * collapses transcribing/thinking/streaming into one `working` loop, but the
 * word stays the real phase so the state remains legible under reduced motion.
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

/** Design `MINI_STATE`: listening and the busy phases carry a marker ring; everything else rests on the mark alone. */
function miniVisual(state: PanelState): MiniVisual {
  if (state === "listening") return "listening";
  if (state === "transcribing" || state === "thinking" || state === "streaming" || state === "speaking") {
    return "working";
  }
  return "ready";
}

export default function MiniMark({
  state,
  onOpen,
  reducedMotion = false,
}: {
  state: PanelState;
  onOpen: () => void;
  reducedMotion?: boolean;
}) {
  const visual = miniVisual(state);
  const word = WORD[state];
  const complete = state === "complete";
  const title = `Ruoxi · ${word}${complete ? " — open to read" : ""}`;

  return (
    <button
      type="button"
      data-state={visual}
      onClick={onOpen}
      aria-label={`Ruoxi — ${word}. Open the panel.`}
      title={title}
      className={`group relative grid h-11 w-11 place-items-center border-0 bg-transparent p-0 text-primary-foreground ${FOCUS_RING}${
        reducedMotion ? " reduced-motion rm-halve" : ""
      }`}
    >
      {visual === "working" ? (
        <span
          aria-hidden="true"
          className={`absolute inset-0 m-auto h-[38px] w-[38px] rounded-full border-2 border-transparent border-t-primary motion-reduce:animate-none${
            reducedMotion ? "" : " animate-orbit-fast"
          }`}
        />
      ) : null}
      {visual === "listening" ? (
        <span
          aria-hidden="true"
          className={`absolute inset-0 m-auto h-[30px] w-[30px] rounded-full border-2 border-primary motion-reduce:animate-none${
            reducedMotion ? "" : " animate-pulse-ring"
          }`}
        />
      ) : null}
      <span
        aria-hidden="true"
        className="grid h-[30px] w-[30px] place-items-center rounded-full bg-primary text-primary-foreground shadow-e3 transition-colors duration-[120ms] ease-in-out group-hover:bg-primary-active"
      >
        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth={2.6} className="h-4 w-4">
          <circle cx="8" cy="8" r="5.4" />
        </svg>
      </span>
      {complete ? (
        <span
          aria-hidden="true"
          className="absolute right-[7px] top-[7px] h-[9px] w-[9px] rounded-full bg-primary-foreground ring-[1.5px] ring-primary"
        />
      ) : null}
    </button>
  );
}
