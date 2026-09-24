import { useEffect, useRef, useState } from "react";
import { invokeTauriAsync } from "../../tauri";
import AnswerStream from "./AnswerStream";
import PanelHeader from "./PanelHeader";
import ReadAloudButton from "./ReadAloudButton";
import SaveMemoryButton from "./SaveMemoryButton";
import type { CaptureInfo, NetState, PanelState } from "./types";

export type ResultPanelProps = {
  state: PanelState;
  net: NetState;
  answer?: string;
  transcript?: string;
  capture?: CaptureInfo;
  hotkeyLabel?: string;
  reducedMotion?: boolean;
  onDismiss?: () => void;
  onEscape?: () => boolean;
  onCollapse?: () => void;
  onSaveMemory?: () => void;
  onRetry?: () => void;
  onPttStart?: () => void;
  onPttStop?: () => void;
  onPttCancel?: () => void;
};

export default function ResultPanel({
  state,
  net,
  answer,
  transcript,
  capture,
  hotkeyLabel,
  reducedMotion,
  onDismiss,
  onEscape,
  onCollapse,
  onSaveMemory,
  onRetry,
  onPttStart,
  onPttStop,
  onPttCancel,
}: ResultPanelProps) {
  const [reading, setReading] = useState(false);
  const readingRef = useRef(false);
  const readTimerRef = useRef<number | null>(null);

  const clearReadTimer = () => {
    if (readTimerRef.current !== null) {
      window.clearTimeout(readTimerRef.current);
      readTimerRef.current = null;
    }
  };

  const stopReading = () => {
    clearReadTimer();
    if (!readingRef.current) return;
    readingRef.current = false;
    setReading(false);
    invokeTauriAsync("tts_stop");
  };

  const startReading = () => {
    if (!answer || !answer.trim()) return;
    invokeTauriAsync("tts_speak_text", { text: answer });
    readingRef.current = true;
    setReading(true);
    clearReadTimer();
    const seconds = Math.min(120, Math.max(1.5, answer.length / 5.5 + 0.6));
    readTimerRef.current = window.setTimeout(() => {
      readTimerRef.current = null;
      readingRef.current = false;
      setReading(false);
    }, seconds * 1000);
  };

  useEffect(() => {
    if (state !== "complete") stopReading();
  }, [state]);

  useEffect(() => {
    if (!onDismiss) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== "Escape") return;
      if (onEscape?.()) return;
      // Esc stops the read-aloud only; the dismiss hotkey hides the panel.
      stopReading();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onDismiss, onEscape]);

  useEffect(
    () => () => {
      clearReadTimer();
      if (readingRef.current) invokeTauriAsync("tts_stop");
    },
    [],
  );

  const caption = (
    <span className="font-ui text-[11px] font-medium leading-[1.4] text-muted-foreground">
      Three steps maximum · <span className="font-mono">Esc</span> aborts
    </span>
  );
  const inFlight =
    state === "ask" || state === "listening" || state === "transcribing" || state === "thinking" || state === "streaming";

  return (
    <section
      role="group"
      aria-label="Ruòxī result panel"
      data-state={state}
      style={{ transitionDuration: "120ms" }}
      className={`panel-surface w-[400px] max-w-full overflow-hidden rounded-[14px] border border-border shadow-e3 transition-opacity opacity-100${
        reducedMotion ? " reduced-motion rm-halve" : ""
      }`}
    >
      <PanelHeader state={state} net={net} onCollapse={onCollapse} />
      <div className="panel-scroll max-h-[60vh] overflow-y-auto px-4 py-3.5">
        <AnswerStream
          state={state}
          answer={answer}
          transcript={transcript}
          capture={capture}
          net={net}
          hotkeyLabel={hotkeyLabel}
          onPttStart={onPttStart}
          onPttStop={onPttStop}
          onPttCancel={onPttCancel}
          onRetry={onRetry}
        />
      </div>
      {inFlight ? (
        <div className="flex items-center border-t border-border px-3 py-2.5">{caption}</div>
      ) : state === "complete" ? (
        <div className="flex items-center justify-between gap-2 border-t border-border px-3 py-2.5">
          {onSaveMemory ? (
            <SaveMemoryButton onSaved={onSaveMemory} />
          ) : (
            <span className="min-w-0 flex-1 font-mono text-[11px] leading-[1.4] text-muted-foreground">
              Saving to memory lands with M2 — nothing is saved yet.
            </span>
          )}
          <ReadAloudButton reading={reading} onRead={startReading} onStop={stopReading} />
        </div>
      ) : null}
    </section>
  );
}
