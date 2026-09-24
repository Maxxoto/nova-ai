import { FOCUS_RING } from "../settings/primitives";
import { invokeTauriAsync } from "../../tauri";
import CloudIndicator from "./CloudIndicator";
import Orb from "./Orb";
import type { NetState, PanelState } from "./types";

const LABELS: Record<PanelState, string> = {
  idle: "Ruòxī",
  ask: "Ready",
  listening: "Listening",
  transcribing: "Transcribing",
  thinking: "Thinking",
  speaking: "Speaking",
  streaming: "Writing",
  complete: "Complete",
  error: "Error",
  degraded: "Resting",
};

export default function PanelHeader({
  state,
  net,
  onCollapse,
}: {
  state: PanelState;
  net: NetState;
  onCollapse?: () => void;
}) {
  return (
    <header
    data-tauri-drag-region
    onMouseDown={(e) => {
      if (e.target === e.currentTarget) invokeTauriAsync("begin_panel_drag")?.catch(() => undefined);
    }}
    className="flex cursor-default items-center gap-2.5 border-b border-border px-3.5 py-2.5"
  >
      <Orb state={state} size={22} />
      <span className="min-w-0 truncate font-ui text-[13px] font-semibold text-foreground">{LABELS[state]}</span>
      <span className="min-w-2 flex-1" />
      <CloudIndicator net={net} />
      {onCollapse ? (
        <button
          type="button"
          onClick={onCollapse}
          aria-label="Collapse to mini"
          title="Collapse to mini"
          className={`inline-flex h-[26px] flex-none items-center justify-center rounded px-2.5 text-body transition-colors duration-200 hover:bg-muted hover:text-foreground ${FOCUS_RING}`}
        >
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth={2}
            strokeLinecap="round"
            className="h-[15px] w-[15px]"
            aria-hidden="true"
          >
            <path d="M6 12h12" />
          </svg>
        </button>
      ) : null}
      <kbd
        aria-label="Press Escape to dismiss"
        className="rounded-[5px] border border-border-strong border-b-2 bg-card px-1.5 py-0.5 font-mono text-[11px] leading-4 text-muted-foreground"
      >
        Esc
      </kbd>
    </header>
  );
}
