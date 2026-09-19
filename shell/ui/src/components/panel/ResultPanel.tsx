import { useEffect, useState } from "react";
import AnswerStream from "./AnswerStream";
import PanelHeader from "./PanelHeader";
import SaveMemoryButton from "./SaveMemoryButton";
import type { NetState, PanelState } from "./types";

export type ResultPanelProps = {
  state: PanelState;
  net: NetState;
  answer?: string;
  capture?: { id: string; time: string };
  reducedMotion?: boolean;
  onDismiss?: () => void;
  onSaveMemory?: () => void;
};

export default function ResultPanel({
  state,
  net,
  answer,
  capture,
  reducedMotion,
  onDismiss,
  onSaveMemory,
}: ResultPanelProps) {
  const [closing, setClosing] = useState(false);

  useEffect(() => {
    if (!onDismiss) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== "Escape") return;
      setClosing(true);
      window.setTimeout(() => onDismiss(), 120);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onDismiss]);

  return (
    <section
      role="group"
      aria-label="Ruòxī result panel"
      data-state={state}
      style={{ transitionDuration: "120ms" }}
      className={`panel-surface w-[400px] max-w-full overflow-hidden rounded-lg border border-border shadow-e3 transition-opacity ${
        closing ? "opacity-0" : "opacity-100"
      }${reducedMotion ? " reduced-motion rm-halve" : ""}`}
    >
      <PanelHeader state={state} net={net} />
      <div className="panel-scroll max-h-[60vh] overflow-y-auto px-5 py-4">
        <AnswerStream state={state} answer={answer} capture={capture} />
      </div>
      {state === "complete" && (
        <div className="flex items-center justify-end border-t border-border px-4 py-3">
          <SaveMemoryButton onSaved={onSaveMemory} />
        </div>
      )}
    </section>
  );
}
