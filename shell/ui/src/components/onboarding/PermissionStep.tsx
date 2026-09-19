import type { ReactElement } from "react";
import type { PermissionKind, PermissionsStatus } from "../../permissions";
import {
  AccessibilityGlyph,
  CheckGlyph,
  DenyGlyph,
  DotGlyph,
  HelpGlyph,
  MicGlyph,
  ScreenGlyph,
} from "./Glyphs";
import { PERMISSION_STEPS } from "./types";
import type { PermissionStepSpec } from "./types";
import { BODY, CAPTION, CHIP, EYEBROW, SECONDARY_BUTTON, STEP_HEADING, WHY_LINE } from "./styles";

type ViewState = "granted" | "denied" | "not_allowed" | "not_asked" | "unknown";

const VIEWS: Record<ViewState, { label: string; tone: string; Glyph: () => ReactElement }> = {
  granted: { label: "granted", tone: "text-success", Glyph: () => <CheckGlyph className="h-3 w-3" /> },
  denied: { label: "denied", tone: "text-destructive", Glyph: () => <DenyGlyph className="h-3 w-3" /> },
  not_allowed: {
    label: "not granted",
    tone: "text-muted-foreground",
    Glyph: () => <DotGlyph className="h-3 w-3" />,
  },
  not_asked: {
    label: "not asked yet",
    tone: "text-muted-foreground",
    Glyph: () => <DotGlyph className="h-3 w-3" />,
  },
  unknown: { label: "unknown", tone: "text-muted-foreground", Glyph: () => <HelpGlyph className="h-3 w-3" /> },
};

const PERMISSION_GLYPHS: Record<PermissionKind, () => ReactElement> = {
  screen_recording: () => <ScreenGlyph />,
  microphone: () => <MicGlyph />,
  accessibility: () => <AccessibilityGlyph />,
};

function viewState(kind: PermissionKind, status: PermissionsStatus): ViewState {
  if (kind === "screen_recording") return status.screen_recording ? "granted" : "not_allowed";
  if (kind === "accessibility") return status.accessibility ? "granted" : "not_allowed";
  switch (status.microphone) {
    case "granted":
      return "granted";
    case "denied":
      return "denied";
    case "not_determined":
      return "not_asked";
    default:
      return "unknown";
  }
}

function isGranted(kind: PermissionKind, status: PermissionsStatus): boolean {
  return viewState(kind, status) === "granted";
}

function StatusChip({ kind, status }: { kind: PermissionKind; status: PermissionsStatus }) {
  const view = VIEWS[viewState(kind, status)];
  return (
    <span className={`${CHIP} ${view.tone}`}>
      <view.Glyph />
      {view.label}
    </span>
  );
}

function PermissionCard({
  spec,
  status,
  pending,
  onRequest,
}: {
  spec: PermissionStepSpec;
  status: PermissionsStatus;
  pending: boolean;
  onRequest: (kind: PermissionKind) => void;
}) {
  const granted = isGranted(spec.kind, status);
  const Glyph = PERMISSION_GLYPHS[spec.kind];

  return (
    <div
      className={`flex items-start gap-3 rounded-lg border bg-card px-4 py-3.5 transition-colors duration-200 ${
        granted ? "border-success" : "border-border"
      }`}
    >
      <span
        aria-hidden
        className={`grid h-8 w-8 flex-none place-items-center rounded-md border ${
          granted ? "border-success text-success" : "border-border bg-muted text-muted-foreground"
        }`}
      >
        <Glyph />
      </span>

      <div className="flex min-w-0 flex-1 flex-col gap-1.5">
        <span className="font-ui text-[14px] font-semibold leading-[1.3] text-foreground">{spec.name}</span>
        <p className={WHY_LINE}>{spec.why}</p>
        <span className={CAPTION}>{spec.path}</span>
        <div className="mt-0.5 flex items-start gap-2 rounded-md border border-dashed border-border bg-muted px-3 py-2">
          <span className="mt-px text-muted-foreground">
            <HelpGlyph className="h-3.5 w-3.5" />
          </span>
          <span className="font-ui text-[12px] leading-[1.4] text-muted-foreground">
            {spec.dialog} <em className="not-italic text-foreground/70">{spec.dialogNote}</em>
          </span>
        </div>
      </div>

      <div className="flex flex-none flex-col items-end gap-2">
        <StatusChip kind={spec.kind} status={status} />
        <button
          type="button"
          aria-busy={pending}
          disabled={granted || pending}
          onClick={() => onRequest(spec.kind)}
          className={SECONDARY_BUTTON}
        >
          {granted ? "Granted" : pending ? "Waiting…" : "Grant"}
        </button>
      </div>
    </div>
  );
}

export interface PermissionStepProps {
  status: PermissionsStatus;
  pendingKind: PermissionKind | null;
  onRequest: (kind: PermissionKind) => void;
}

export default function PermissionStep({ status, pendingKind, onRequest }: PermissionStepProps) {
  const grantedCount = PERMISSION_STEPS.filter((spec) => isGranted(spec.kind, status)).length;
  const captureReady = status.screen_recording && status.accessibility;
  const micUnreadable = status.microphone === "unknown";
  const counter = !captureReady
    ? `${grantedCount} of 3 granted — the guided capture unlocks when Screen Recording and Accessibility are on.`
    : micUnreadable
      ? `${grantedCount} of 3 granted — macOS doesn't report the microphone's status to Ruòxī; grant it in System Settings for voice.`
      : `${grantedCount} of 3 granted — the guided capture is unlocked.`;

  return (
    <div className="flex flex-col gap-5">
      <div className="flex flex-col gap-3">
        <span className={EYEBROW}>Step 2 of 5 · AC-12</span>
        <h2 data-step-heading tabIndex={-1} className={STEP_HEADING}>
          Three permissions, three reasons.
        </h2>
        <p className={BODY}>
          Each one is preceded by the reason it exists. If a reason doesn&rsquo;t convince you, don&rsquo;t grant it —
          the app will tell you exactly which features stop working.
        </p>
      </div>

      <div className="flex flex-col gap-2.5">
        {PERMISSION_STEPS.map((spec) => (
          <PermissionCard
            key={spec.kind}
            spec={spec}
            status={status}
            pending={pendingKind === spec.kind}
            onRequest={onRequest}
          />
        ))}
      </div>

      <p aria-live="polite" className={CAPTION}>
        {counter}
      </p>
    </div>
  );
}
