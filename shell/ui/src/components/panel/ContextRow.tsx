import type { CaptureScope } from "./types";

export const SCOPE_LABELS: Record<CaptureScope, string> = {
  region: "Region",
  window: "Window",
  screen: "Whole screen",
};

export default function ContextRow({
  scope,
  width,
  height,
}: {
  scope: CaptureScope;
  width: number;
  height: number;
}) {
  return (
    <div className="ctx">
      <svg
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth={1.7}
        strokeLinecap="round"
        aria-hidden="true"
      >
        <path d="M4 8V5h3M20 8V5h-3M4 16v3h3M20 16v3h-3" />
      </svg>
      <span className="ctx-label">Context · {SCOPE_LABELS[scope]}</span>
      <span className="ctx-dim">
        {width} × {height} px
      </span>
    </div>
  );
}
