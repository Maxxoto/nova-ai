import type { ReactElement } from "react";
import type { NetState } from "./types";

const LABEL: Record<NetState, string> = {
  online: "Online",
  local_only: "Local Only",
  offline: "Offline",
};

/* The design's `data-cloud` vocabulary is `online | local | offline`
   (OD capture-and-ask.frag:265, CLOUD_ORDER). Our `NetState` spells the middle
   state `local_only`, so map it here and the ported `.cloud` CSS stays verbatim. */
const CLOUD_ATTR: Record<NetState, string> = {
  online: "online",
  local_only: "local",
  offline: "offline",
};

function CloudGlyph(): ReactElement {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.7"
      strokeLinecap="round"
      aria-hidden="true"
    >
      <path className="cloud-slash" d="M4 4l16 16" />
      <path d="M7 18h9a4 4 0 0 0 .6-8A5.5 5.5 0 0 0 6 9.4 3.8 3.8 0 0 0 7 18Z" />
    </svg>
  );
}

export default function CloudIndicator({
  net,
  inflight = false,
}: {
  net: NetState;
  inflight?: boolean;
}) {
  return (
    <span role="status" className="cloud" data-cloud={CLOUD_ATTR[net]} data-inflight={inflight ? "true" : "false"}>
      <CloudGlyph />
      <span className="cdot" aria-hidden="true" />
      <span>{LABEL[net]}</span>
    </span>
  );
}
