import type { PanelState } from "./types";

function discClass(state: PanelState): string {
  if (state === "listening" || state === "transcribing") return "orb-disc bg-live";
  if (state === "degraded") return "orb-disc bg-warning";
  return "orb-disc bg-primary";
}

function MoonGlyph() {
  return (
    <svg
      viewBox="0 0 24 24"
      className="orb-glyph"
      fill="none"
      stroke="currentColor"
      strokeWidth="2.4"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
    </svg>
  );
}

export default function Orb({ state, size = 20 }: { state: PanelState; size?: number }) {
  const breathe = state === "idle" || state === "complete" ? " orb-breathe" : "";

  return (
    <span className={`orb${breathe}`} style={{ width: size, height: size }} aria-hidden="true">
      {state === "thinking" ? (
        <span className="orb-orbit">
          <i />
          <i />
          <i />
        </span>
      ) : (
        <span className={discClass(state)} />
      )}
      {state === "listening" && <span className="orb-ring" />}
      {state === "speaking" && (
        <>
          <span className="orb-wave" />
          <span className="orb-wave orb-wave-2" />
        </>
      )}
      {state === "transcribing" && (
        <span className="orb-dots">
          <i />
          <i />
          <i />
        </span>
      )}
      {state === "error" && <span className="orb-ring orb-ring--error" />}
      {state === "degraded" && <MoonGlyph />}
    </span>
  );
}
