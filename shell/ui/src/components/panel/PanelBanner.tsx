import type { ReactElement } from "react";

function AlertGlyph(): ReactElement {
  return (
    <svg
      viewBox="0 0 24 24"
      className="panel-banner-glyph"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d="M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
      <line x1="12" y1="9" x2="12" y2="13" />
      <line x1="12" y1="17" x2="12.01" y2="17" />
    </svg>
  );
}

function RestGlyph(): ReactElement {
  return (
    <svg
      viewBox="0 0 24 24"
      className="panel-banner-glyph"
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

export default function PanelBanner({
  kind,
  onAction,
}: {
  kind: "error" | "degraded";
  onAction?: () => void;
}) {
  if (kind === "error") {
    return (
      <div role="alert" className="panel-banner panel-banner--error flex items-start gap-2.5">
        <AlertGlyph />
        <div className="flex flex-1 flex-col items-start gap-2">
          <p className="font-ui text-[13px] leading-[1.4]">
            She couldn’t reach her brain — the sidecar stopped responding.
          </p>
          <button type="button" onClick={onAction} className="panel-action">
            Retry
          </button>
        </div>
      </div>
    );
  }
  return (
    <div role="status" className="panel-banner panel-banner--degraded flex items-start gap-2.5">
      <RestGlyph />
      <div className="flex flex-1 flex-col items-start gap-2">
        <p className="font-companion text-[15px] font-semibold leading-[1.3]">Ruòxī is resting…</p>
        <p className="font-ui text-[13px] leading-[1.4] text-muted-foreground">
          The sidecar restarted too many times.
        </p>
        <button type="button" onClick={onAction} className="panel-action">
          Restart
        </button>
      </div>
    </div>
  );
}
