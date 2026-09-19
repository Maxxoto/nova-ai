import CloudIndicator from "./CloudIndicator";
import Orb from "./Orb";
import type { NetState, PanelState } from "./types";

const LABELS: Record<PanelState, string> = {
  idle: "Ruòxī",
  listening: "Listening",
  transcribing: "Transcribing",
  thinking: "Thinking",
  speaking: "Speaking",
  streaming: "Streaming",
  complete: "Done",
  error: "Error",
  degraded: "Resting",
};

export default function PanelHeader({ state, net }: { state: PanelState; net: NetState }) {
  return (
    <header className="flex items-center gap-2.5 border-b border-border px-3.5 py-2.5">
      <Orb state={state} size={22} />
      <span className="min-w-0 truncate font-ui text-[13px] font-semibold text-foreground">{LABELS[state]}</span>
      <span className="min-w-2 flex-1" />
      <CloudIndicator net={net} />
      <kbd
        aria-label="Press Escape to dismiss"
        className="rounded-[5px] border border-border-strong border-b-2 bg-card px-1.5 py-0.5 font-mono text-[11px] leading-4 text-muted-foreground"
      >
        Esc
      </kbd>
    </header>
  );
}
