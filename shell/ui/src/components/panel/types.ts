export type PanelState =
  | "idle"
  | "ask"
  | "listening"
  | "transcribing"
  | "thinking"
  | "speaking"
  | "streaming"
  | "complete"
  | "error"
  | "degraded";

export type NetState = "offline" | "local_only" | "calling_cloud";

export const PANEL_STATES: PanelState[] = [
  "idle",
  "ask",
  "listening",
  "transcribing",
  "thinking",
  "speaking",
  "streaming",
  "complete",
  "error",
  "degraded",
];

export type CaptureScope = "region" | "window" | "screen";

export type CaptureInfo = {
  id: string;
  time: string;
  scope: CaptureScope;
  width: number;
  height: number;
};

export const NET_STATES: NetState[] = ["offline", "local_only", "calling_cloud"];

export function isPanelState(v: string | null): v is PanelState {
  return v !== null && (PANEL_STATES as string[]).includes(v);
}

export function isNetState(v: string | null): v is NetState {
  return v !== null && (NET_STATES as string[]).includes(v);
}
