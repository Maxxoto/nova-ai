import type { ReactElement } from "react";

const FOCUS_RING =
  "focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring";

type GlyphProps = { className?: string };

function AlertGlyph({ className }: GlyphProps): ReactElement {
  return (
    <svg
      viewBox="0 0 24 24"
      className={className}
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
      aria-hidden="true"
    >
      <path d="M12 8v5m0 3h.01" />
      <circle cx="12" cy="12" r="9" />
    </svg>
  );
}

function RestGlyph({ className }: GlyphProps): ReactElement {
  return (
    <svg
      viewBox="0 0 24 24"
      className={className}
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
    </svg>
  );
}

type PanelBannerProps =
  | { kind: "error"; capture?: { id: string; time: string }; offline: boolean; onAction?: () => void }
  | { kind: "degraded"; onAction?: () => void };

export default function PanelBanner(props: PanelBannerProps) {
  if (props.kind === "error") {
    const { capture, offline, onAction } = props;
    const base = capture
      ? `Last attempt ${capture.time} · capture ${capture.id} kept on disk`
      : "The capture stays in your local store";
    const detail = offline ? `${base} · offline mode is on, so nothing was sent.` : `${base}.`;
    return (
      <div className="panel-error-banner flex flex-col">
        <div
          role="status"
          className="flex items-center gap-3 rounded-[10px] border border-destructive bg-primary-soft px-3.5 py-2.5"
        >
          <AlertGlyph className="h-4 w-4 flex-none text-destructive" />
          <span className="flex-1 font-ui text-[13px] leading-[1.4] text-destructive">
            Couldn’t reach the model. Your capture is safe locally.
          </span>
          <button
            type="button"
            onClick={onAction}
            className={`shrink-0 px-1 font-ui text-[11px] font-semibold text-destructive underline underline-offset-2 ${FOCUS_RING}`}
          >
            Retry
          </button>
        </div>
        <p className="pl-[42px] pr-3.5 pt-2 font-mono text-[11px] leading-[1.4] text-muted-foreground">{detail}</p>
      </div>
    );
  }

  return (
    <div
      role="status"
      className="flex items-center gap-3 rounded-[10px] border border-warning bg-primary-soft px-3.5 py-2.5"
    >
      <RestGlyph className="h-4 w-4 flex-none text-warning" />
      <span className="flex-1 font-ui text-[13px] leading-[1.4] text-warning">
        Ruòxī is resting — the assistant core is restarting.
      </span>
      <button
        type="button"
        onClick={props.onAction}
        className={`shrink-0 px-1 font-ui text-[11px] font-semibold text-warning underline underline-offset-2 ${FOCUS_RING}`}
      >
        Restart
      </button>
    </div>
  );
}
