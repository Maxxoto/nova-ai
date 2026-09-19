import CloudIndicator from "./CloudIndicator";
import Orb from "./Orb";
import type { NetState, PanelState } from "./types";

const LABELS: Record<PanelState, string> = {
  idle: "Ruòxī",
  listening: "listening",
  transcribing: "transcribing",
  thinking: "thinking",
  speaking: "speaking",
  streaming: "streaming",
  complete: "done",
  error: "error",
  degraded: "resting",
};

export default function PanelHeader({ state, net }: { state: PanelState; net: NetState }) {
  return (
    <header className="flex h-11 items-center gap-2.5 border-b border-border px-4">
      <Orb state={state} />
      <span className="min-w-0 truncate font-companion text-[13px] font-semibold text-foreground">{LABELS[state]}</span>
      <span className="min-w-2 flex-1" />
      <CloudIndicator net={net} />
      <kbd
        aria-label="Press Escape to dismiss"
        className="rounded-sm border border-border bg-muted px-1.5 py-px font-mono text-[11px] leading-none text-muted-foreground"
      >
        esc
      </kbd>
    </header>
  );
}
