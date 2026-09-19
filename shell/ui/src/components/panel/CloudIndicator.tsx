import type { ReactElement } from "react";
import type { NetState } from "./types";

type GlyphProps = { className?: string };

function CloudHollow({ className }: GlyphProps): ReactElement {
  return (
    <svg
      viewBox="0 0 24 24"
      className={`cloud-glyph ${className ?? ""}`}
      fill="none"
      stroke="currentColor"
      strokeWidth="1.7"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d="M7 18h9a4 4 0 0 0 .6-8A5.5 5.5 0 0 0 6 9.4 3.8 3.8 0 0 0 7 18Z" />
    </svg>
  );
}

function CloudOff({ className }: GlyphProps): ReactElement {
  return (
    <svg
      viewBox="0 0 24 24"
      className={`cloud-glyph ${className ?? ""}`}
      fill="none"
      stroke="currentColor"
      strokeWidth="1.7"
      strokeLinecap="round"
      aria-hidden="true"
    >
      <path d="M4 4l16 16" />
      <path d="M7 18h9a4 4 0 0 0 2-7.5" />
      <path d="M6 9.4A3.8 3.8 0 0 0 7 18" />
    </svg>
  );
}

function CloudUpload({ className }: GlyphProps): ReactElement {
  return (
    <svg
      viewBox="0 0 24 24"
      className={`cloud-glyph ${className ?? ""}`}
      fill="none"
      stroke="currentColor"
      strokeWidth="1.7"
      strokeLinecap="round"
      aria-hidden="true"
    >
      <path d="M7 18h9a4 4 0 0 0 .6-8A5.5 5.5 0 0 0 6 9.4 3.8 3.8 0 0 0 7 18Z" />
      <path d="M12 11v5m0-5-2 2m2-2 2 2" />
    </svg>
  );
}

const META: Record<
  NetState,
  { label: string; iconClass: string; dotClass: string; icon: (props: GlyphProps) => ReactElement }
> = {
  offline: {
    label: "Offline",
    iconClass: "text-muted-foreground",
    dotClass: "bg-muted-foreground",
    icon: CloudOff,
  },
  local_only: {
    label: "Local Only",
    iconClass: "text-success",
    dotClass: "bg-success",
    icon: CloudHollow,
  },
  calling_cloud: {
    label: "Sending to Cloud",
    iconClass: "text-live",
    dotClass: "bg-live animate-pulse-ring",
    icon: CloudUpload,
  },
};

export default function CloudIndicator({ net }: { net: NetState }) {
  const meta = META[net];
  return (
    <span
      role="status"
      className="inline-flex flex-none items-center gap-1.5 whitespace-nowrap rounded-pill border border-border bg-card px-2.5 py-1 font-ui text-[11px] font-medium text-muted-foreground"
    >
      <meta.icon className={`h-[13px] w-[13px] ${meta.iconClass}`} />
      <span className={`h-[7px] w-[7px] flex-none rounded-full ${meta.dotClass}`} aria-hidden="true" />
      <span>{meta.label}</span>
    </span>
  );
}
