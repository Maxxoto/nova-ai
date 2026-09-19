import type { PanelState } from "./types";

function discClass(state: PanelState): string {
  if (state === "thinking") return "orb-disc bg-thinking-text";
  if (state === "degraded") return "orb-disc bg-warning";
  if (state === "error") return "orb-disc bg-destructive";
  if (state === "transcribing") return "orb-disc bg-live";
  return "orb-disc bg-primary";
}

export default function Orb({ state, size = 22 }: { state: PanelState; size?: number }) {
  const motion = state === "speaking" ? " orb-speaking" : " orb-breathe";

  return (
    <span className={`orb${motion}`} style={{ width: size, height: size }} aria-hidden="true">
      <span className={discClass(state)} />
      {state === "thinking" && (
        <span className="orb-orbit">
          <i />
          <i />
          <i />
        </span>
      )}
      {state === "listening" && <span className="orb-ring" />}
      {state === "transcribing" && (
        <span className="orb-dots">
          <i />
          <i />
          <i />
        </span>
      )}
      {state === "error" && <span className="orb-ring orb-ring--error" />}
    </span>
  );
}
