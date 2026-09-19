import type { ReactElement } from "react";
import type { NetState } from "./types";

function CloudHollow(): ReactElement {
  return (
    <svg
      viewBox="0 0 24 24"
      className="cloud-glyph"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d="M18 10h-1.26A8 8 0 1 0 9 20h9a5 5 0 0 0 0-10z" />
    </svg>
  );
}

function CloudOff(): ReactElement {
  return (
    <svg
      viewBox="0 0 24 24"
      className="cloud-glyph"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d="M22.61 16.95A5 5 0 0 0 18 10h-1.26a8 8 0 0 0-7.05-6M5 5a8 8 0 0 0 4 15h9a5 5 0 0 0 1.7-.3" />
      <line x1="1" y1="1" x2="23" y2="23" />
    </svg>
  );
}

function CloudUpload(): ReactElement {
  return (
    <svg
      viewBox="0 0 24 24"
      className="cloud-glyph"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d="M20 16.58A5 5 0 0 0 18 7h-1.26A8 8 0 1 0 4 15.25" />
      <polyline points="16 16 12 12 8 16" />
      <line x1="12" y1="12" x2="12" y2="21" />
    </svg>
  );
}

const META: Record<NetState, { label: string; tone: string; dot: string; icon: () => ReactElement }> = {
  offline: {
    label: "offline · nothing leaves this Mac",
    tone: "text-muted-foreground",
    dot: "bg-muted-foreground",
    icon: CloudOff,
  },
  local_only: {
    label: "local only",
    tone: "text-success",
    dot: "bg-success",
    icon: CloudHollow,
  },
  calling_cloud: {
    label: "sending to cloud",
    tone: "text-warning",
    dot: "bg-live animate-pulse-ring",
    icon: CloudUpload,
  },
};

export default function CloudIndicator({ net }: { net: NetState }) {
  const meta = META[net];
  return (
    <span
      role="status"
      className={`inline-flex flex-none items-center gap-1.5 whitespace-nowrap rounded-pill border border-border bg-muted px-2 py-[3px] font-ui text-[11px] font-medium ${meta.tone}`}
    >
      <meta.icon />
      <span>{meta.label}</span>
      <span className={`h-1.5 w-1.5 rounded-full ${meta.dot}`} aria-hidden="true" />
    </span>
  );
}
