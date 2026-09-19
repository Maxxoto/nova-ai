import { invokeTauriAsync } from "../../tauri";
import PanelBanner from "./PanelBanner";
import ToolStepIndicator from "./ToolStepIndicator";
import type { NetState, PanelState } from "./types";

const CAPTION = "inline-flex flex-wrap items-center gap-2 font-ui text-[11px] font-medium leading-[1.4] text-muted-foreground";

const QUIET: Partial<Record<PanelState, { text: string; className: string }>> = {
  idle: { text: "no answer yet", className: "font-ui text-[13px] text-muted-foreground" },
  listening: { text: "listening…", className: "font-companion text-[13px] font-semibold text-muted-foreground" },
  transcribing: { text: "transcribing…", className: "font-ui text-[13px] text-muted-foreground" },
};

function CiteChip({ capture }: { capture: { id: string; time: string } }) {
  return (
    <span className="group relative inline-flex">
      <button
        type="button"
        onClick={() => invokeTauriAsync("show_timeline")?.catch(() => undefined)}
        className="inline-flex items-center gap-1.5 rounded-sm border border-primary/25 bg-primary-soft px-2 py-[3px] font-mono text-[11px] leading-[1.4] text-primary transition-colors duration-200 hover:border-primary focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring"
      >
        {capture.id} · {capture.time}
      </button>
      <span
        aria-hidden="true"
        className="pointer-events-none invisible absolute bottom-[calc(100%+8px)] left-0 z-[6] w-[190px] translate-y-1.5 rounded-lg border border-border bg-card p-1.5 opacity-0 shadow-e2 transition-all duration-150 group-hover:visible group-hover:translate-y-0 group-hover:opacity-100 group-focus-within:visible group-focus-within:translate-y-0 group-focus-within:opacity-100"
      >
        <span className="grid h-[58px] place-items-center rounded-sm border border-border bg-muted px-1 text-center font-mono text-[11px] leading-[1.4] text-muted-foreground">
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
  capture,
  net,
  onRetry,
}: {
  state: PanelState;
  answer?: string;
  capture?: { id: string; time: string };
  net: NetState;
  onRetry?: () => void;
}) {
  if (state === "thinking") {
    return (
      <div className="flex flex-col gap-2.5">
        <span className={CAPTION}>
          <ToolStepIndicator active={1} />
          <span>reading the {capture ? "region" : "transcript"}</span>
        </span>
        <div className="flex flex-col gap-2" aria-hidden="true">
          <span className="panel-skeleton h-[9px] w-[94%] rounded-[3px]" />
          <span className="panel-skeleton h-[9px] w-[86%] rounded-[3px]" />
          <span className="panel-skeleton h-[9px] w-[62%] rounded-[3px]" />
        </div>
      </div>
    );
  }
  if (state === "error") {
    return <PanelBanner kind="error" capture={capture} offline={net === "offline"} onAction={onRetry} />;
  }
  if (state === "degraded") return <PanelBanner kind="degraded" />;

  const quiet = QUIET[state];
  if (quiet) return <p className={quiet.className}>{quiet.text}</p>;
  if (!answer) return null;

  return (
    <div className="flex flex-col gap-3">
      <p className="font-ui text-[14px] leading-[1.5] text-foreground">
        {answer}
        {state === "streaming" ? <span className="panel-caret" aria-hidden="true" /> : null}
      </p>
      {state === "streaming" ? (
        <span className={CAPTION}>
          <ToolStepIndicator active={2} />
          <span>writing the answer</span>
        </span>
      ) : (
        <span className="flex flex-wrap items-center gap-2">
          {capture ? <CiteChip capture={capture} /> : null}
          <ToolStepIndicator active={3} />
        </span>
      )}
    </div>
  );
}
