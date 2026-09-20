import type { PanelState } from "./types";

function discClass(state: PanelState): string {
  if (state === "thinking" || state === "transcribing") return "orb-disc orb-disc--thinking";
  if (state === "degraded") return "orb-disc orb-disc--degraded";
  if (state === "error") return "orb-disc orb-disc--error";
  return "orb-disc";
}

export default function Orb({ state, size = 22 }: { state: PanelState; size?: number }) {
  const motion = state === "speaking" ? " orb-speaking" : " orb-breathe";

  return (
    <span className={`orb${motion}`} style={{ width: size, height: size }} aria-hidden="true">
      <span className={discClass(state)} />
      {state === "listening" ? <span className="orb-ring" /> : null}
      {state === "thinking" || state === "transcribing" ? (
        <span className="orb-orbit">
          <i />
          <i />
          <i />
        </span>
      ) : null}
    </span>
  );
}
