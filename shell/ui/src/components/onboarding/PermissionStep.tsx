import type { ReactElement } from "react";
import type { PermissionKind, PermissionsStatus } from "../../permissions";
import { AccessibilityGlyph, MicGlyph, ScreenGlyph } from "./Glyphs";
import { PERMISSION_STEPS } from "./types";
import type { PermissionStepSpec } from "./types";
import { BODY, CAPTION, EYEBROW, SECONDARY_BUTTON_SM, STEP_HEADING, STRONG, TAG, TAG_OK, TECH } from "./styles";

type ViewState = "granted" | "denied" | "not_allowed" | "not_asked" | "unknown";

const VIEWS: Record<ViewState, { label: string; tone: string }> = {
  granted: { label: "granted", tone: "" },
  denied: { label: "denied", tone: "border-destructive text-destructive" },
  not_allowed: { label: "not granted", tone: "" },
  not_asked: { label: "not asked", tone: "" },
  unknown: { label: "unknown", tone: "" },
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

/** The macOS dialog mark — design `.os-prompt .op-mark`. */
function OsPrompt({ quote }: { quote: string }) {
  return (
    <div className="mt-2.5 flex items-center gap-2.5 rounded-md border border-dashed border-border-strong bg-muted px-3 py-[9px]">
      <span aria-hidden className="grid h-[18px] w-[18px] flex-none place-items-center rounded-[5px] bg-muted-foreground">
        <svg
          viewBox="0 0 24 24"
          className="h-[11px] w-[11px] text-background"
          fill="none"
          stroke="currentColor"
          strokeWidth="2.2"
          strokeLinecap="round"
        >
          <path d="M12 4v8" />
          <path d="M6 7a8 8 0 1 0 12 0" />
        </svg>
      </span>
      <span className="font-ui text-[11px] leading-[1.4] text-body">{quote}</span>
    </div>
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
  const state = viewState(spec.kind, status);
  const granted = state === "granted";
  const Glyph = PERMISSION_GLYPHS[spec.kind];
  const view = VIEWS[state];

  return (
    <div
      className={`flex items-start gap-3 rounded-md border bg-card p-3.5 transition-colors duration-200 ${
        granted ? "border-success" : "border-border"
      }`}
    >
      <span
        aria-hidden
        className={`grid h-7 w-7 flex-none place-items-center rounded-[8px] border ${
          granted ? "border-success text-success" : "border-border bg-muted text-body"
        }`}
      >
        <Glyph />
      </span>

      <div className="min-w-0 flex-1">
        <span className={STRONG}>{spec.name}</span>
        <p className="mt-1 font-ui text-[14px] font-semibold leading-[1.4] text-foreground">{spec.why}</p>
        <span className={`${TECH} mt-1.5 block`}>{spec.path}</span>
        {(granted || pending) && <OsPrompt quote={spec.dialog} />}
      </div>

      <div className="flex flex-none items-center gap-2">
        <span className={`${granted ? TAG_OK : TAG} ${view.tone}`}>{view.label}</span>
        {!granted && (
          <button
            type="button"
            aria-busy={pending}
            disabled={pending}
            onClick={() => onRequest(spec.kind)}
            className={SECONDARY_BUTTON_SM}
          >
            {pending ? "Waiting…" : "Grant"}
          </button>
        )}
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
  const micUnreadable = status.microphone === "unknown";
  const counter = `${grantedCount} of 3 granted.${
    micUnreadable
      ? " macOS doesn't report the microphone's status to Ruòxī — grant it in System Settings for voice."
      : ""
  }`;

  return (
    <div className="flex flex-col">
      <span className={EYEBROW}>Step 2 of 6</span>
      <h2 data-step-heading tabIndex={-1} className={STEP_HEADING}>
        Three permissions.
      </h2>
      <p className={`${BODY} mt-2 max-w-[50ch]`}>Each one is followed by the reason it exists.</p>

      <div className="mt-5 flex flex-col gap-2.5">
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

      <p aria-live="polite" className={`${CAPTION} mt-4`}>
        {counter}
      </p>
    </div>
  );
}
