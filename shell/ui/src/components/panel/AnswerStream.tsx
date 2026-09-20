import type { ReactNode } from "react";
import { invokeTauriAsync } from "../../tauri";
import ContextRow, { SCOPE_LABELS } from "./ContextRow";
import PanelBanner from "./PanelBanner";
import PttPill from "./PttPill";
import ToolStepIndicator from "./ToolStepIndicator";
import type { CaptureInfo, NetState, PanelState } from "./types";

/** Renders the answer's inline markdown (**bold**) as real <strong> runs. */
function renderInline(text: string): ReactNode[] {
  return text.split(/(\*\*[^*]+\*\*)/g).map((part, index) =>
    part.startsWith("**") && part.endsWith("**") && part.length > 4 ? (
      <strong key={index} className="font-semibold">
        {part.slice(2, -2)}
      </strong>
    ) : (
      <span key={index}>{part}</span>
    ),
  );
}

function AnswerText({ text, caret = false }: { text: string; caret?: boolean }) {
  const paragraphs = text
    .split(/\n+/)
    .map((line) => line.trim())
    .filter((line) => line.length > 0);
  return (
    <>
      {paragraphs.map((line, index) => (
        <p
          key={index}
          className={`font-ui text-[14px] leading-[1.5] text-foreground${index > 0 ? " mt-2.5" : ""}`}
        >
          {renderInline(line)}
          {caret && index === paragraphs.length - 1 ? (
            <span className="panel-caret" aria-hidden="true" />
          ) : null}
        </p>
      ))}
    </>
  );
}

const CAPTION = "flex flex-wrap items-center gap-2 font-ui text-[11px] font-medium leading-[1.4] text-muted-foreground";

function CiteChip({ capture }: { capture: { id: string; time: string } }) {
  return (
    <span className="group relative inline-flex">
      <button
        type="button"
        onClick={() => invokeTauriAsync("show_timeline")?.catch(() => undefined)}
        className="inline-flex items-center gap-1.5 rounded-[6px] border border-primary/25 bg-primary-soft px-2 py-[3px] font-mono text-[11px] leading-[1.4] text-primary-active transition-colors duration-200 hover:border-primary focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring"
      >
        {capture.id} · {capture.time}
      </button>
      <span
        aria-hidden="true"
        className="pointer-events-none invisible absolute bottom-[calc(100%+8px)] left-0 z-[6] w-[190px] translate-y-1.5 rounded-[10px] border border-border bg-card p-1.5 opacity-0 shadow-e2 transition-all duration-150 group-hover:visible group-hover:translate-y-0 group-hover:opacity-100 group-focus-within:visible group-focus-within:translate-y-0 group-focus-within:opacity-100"
      >
        <span className="grid h-[58px] place-items-center rounded-[6px] border border-border bg-muted px-1 text-center font-mono text-[11px] leading-[1.4] text-muted-foreground">
          screenshot · capture
        </span>
        <span className="mt-1.5 flex justify-between gap-1.5 font-mono text-[11px] text-muted-foreground">
          <span>{capture.id}</span>
          <span>{capture.time}</span>
        </span>
      </span>
    </span>
  );
}

export default function AnswerStream({
  state,
  answer,
  transcript,
  capture,
  net,
  hotkeyLabel = "⌥⇧V",
  onPttStart,
  onPttStop,
  onPttCancel,
  onRetry,
}: {
  state: PanelState;
  answer?: string;
  transcript?: string;
  capture?: CaptureInfo;
  net: NetState;
  hotkeyLabel?: string;
  onPttStart?: () => void;
  onPttStop?: () => void;
  onPttCancel?: () => void;
  onRetry?: () => void;
}) {
  const start = onPttStart ?? (() => undefined);
  const stop = onPttStop ?? (() => undefined);
  const cancel = onPttCancel ?? (() => undefined);

  if (state === "error") {
    return <PanelBanner kind="error" capture={capture} offline={net === "offline"} onAction={onRetry} />;
  }
  if (state === "degraded") return <PanelBanner kind="degraded" />;

  const context = capture ? (
    <ContextRow scope={capture.scope} width={capture.width} height={capture.height} />
  ) : null;

  if (state === "ask") {
    return (
      <>
        {context}
        <div className="ask-stage">
          <PttPill label={`Hold ${hotkeyLabel} to ask`} onPressStart={start} onRelease={stop} onCancel={cancel} />
        </div>
        <p className="ask-note font-ui text-[11px] font-medium leading-[1.4] text-muted-foreground">
          Speak the question about what you captured. Release to send.
        </p>
      </>
    );
  }

  if (state === "listening") {
    return (
      <>
        {context}
        <div className="ask-stage">
          <PttPill live label="Listening…" onPressStart={start} onRelease={stop} onCancel={cancel} />
        </div>
        <p className="ask-note font-ui text-[11px] font-medium leading-[1.4] text-muted-foreground">
          Release to send · Esc cancels.
        </p>
      </>
    );
  }

  if (state === "transcribing") {
    return (
      <>
        {context}
        <div className="transcript">
          {transcript && transcript.trim() ? <p className="text-[13px] text-body">{transcript}</p> : null}
        </div>
        <span className="font-ui text-[11px] font-medium leading-[1.4] text-muted-foreground">Transcribing…</span>
      </>
    );
  }

  if (state === "thinking") {
    const scopeWord = capture ? SCOPE_LABELS[capture.scope].toLowerCase() : "capture";
    return (
      <>
        {context}
        <span className={`${CAPTION} mb-2.5`}>
          <ToolStepIndicator active={1} />
          <span>reading the {scopeWord}</span>
        </span>
        <div className="flex flex-col gap-2" aria-hidden="true">
          <span className="panel-skeleton h-[9px] w-[94%] rounded-[3px]" />
          <span className="panel-skeleton h-[9px] w-[86%] rounded-[3px]" />
          <span className="panel-skeleton h-[9px] w-[62%] rounded-[3px]" />
        </div>
      </>
    );
  }

  if (state === "idle") {
    return <p className="font-ui text-[13px] text-muted-foreground">no answer yet</p>;
  }

  if (state === "streaming") {
    return (
      <>
        {context}
        {answer ? (
          <AnswerText text={answer} caret />
        ) : (
          <p className="font-ui text-[14px] leading-[1.5] text-foreground">
            <span className="panel-caret" aria-hidden="true" />
          </p>
        )}
        <span className={`${CAPTION} mt-2.5`}>
          <ToolStepIndicator active={2} />
          <span>writing the answer</span>
        </span>
      </>
    );
  }

  if (!answer) return null;

  return (
    <>
      {context}
      <AnswerText text={answer} />
      <span className="mt-3 flex flex-wrap items-center gap-2">
        {capture ? <CiteChip capture={capture} /> : null}
        <ToolStepIndicator active={3} />
      </span>
    </>
  );
}
