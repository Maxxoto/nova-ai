import type { ReactElement } from "react";
import PanelBanner from "./PanelBanner";
import ToolStepIndicator from "./ToolStepIndicator";
import type { PanelState } from "./types";

const QUIET: Partial<Record<PanelState, { text: string; className: string }>> = {
  idle: { text: "no answer yet", className: "font-ui text-[13px] text-muted-foreground" },
  listening: { text: "listening…", className: "font-companion text-[13px] font-semibold text-muted-foreground" },
  transcribing: { text: "transcribing…", className: "font-ui text-[13px] text-muted-foreground" },
};

function ThumbGlyph(): ReactElement {
  return (
    <svg
      viewBox="0 0 24 24"
      className="h-4 w-4"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.6"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <rect x="3" y="3" width="18" height="18" rx="2" />
      <circle cx="8.5" cy="8.5" r="1.5" />
      <polyline points="21 15 16 10 5 21" />
    </svg>
  );
}

export default function AnswerStream({
  state,
  answer,
  capture,
}: {
  state: PanelState;
  answer?: string;
  capture?: { id: string; time: string };
}) {
  if (state === "thinking") {
    return (
      <div className="flex flex-col gap-4">
        <ToolStepIndicator active={1} />
        <div className="flex flex-col gap-2.5" aria-hidden="true">
          <span className="panel-skeleton h-2.5 w-full rounded-pill" />
          <span className="panel-skeleton h-2.5 w-[86%] rounded-pill" />
          <span className="panel-skeleton h-2.5 w-[58%] rounded-pill" />
        </div>
      </div>
    );
  }
  if (state === "error") return <PanelBanner kind="error" />;
  if (state === "degraded") return <PanelBanner kind="degraded" />;

  const quiet = QUIET[state];
  if (quiet) return <p className={quiet.className}>{quiet.text}</p>;
  if (!answer) return null;

  return (
    <div className="flex flex-col gap-3">
      <p className="font-ui text-[14px] leading-[1.45] text-foreground">
        {answer}
        {state === "streaming" && <span className="panel-caret" aria-hidden="true" />}
      </p>
      {capture && (
        <div className="flex items-center gap-2.5 rounded-md border border-border bg-card p-2 shadow-e1 transition-shadow duration-200 hover:shadow-e2">
          <span
            className="flex h-11 w-16 flex-none items-center justify-center rounded-sm border border-border bg-muted text-muted-foreground"
            aria-hidden="true"
          >
            <ThumbGlyph />
          </span>
          <span className="flex min-w-0 flex-col gap-0.5">
            <span className="truncate font-mono text-[11px] text-muted-foreground">{capture.id}</span>
            <span className="font-mono text-[11px] text-muted-foreground">{capture.time} · capture</span>
          </span>
        </div>
      )}
    </div>
  );
}
