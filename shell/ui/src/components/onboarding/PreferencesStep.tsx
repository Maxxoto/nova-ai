import { useId } from "react";
import type { ReactNode } from "react";
import { BODY, EYEBROW, FOCUS_RING, STEP_HEADING, TAG } from "./styles";

export type PreferenceField = "offline" | "launch_at_login" | "read_aloud";

export interface PreferenceValues {
  offline: boolean;
  launch_at_login: boolean;
  read_aloud: boolean;
}

/** Design `.switch` — 34×20 track, 16px thumb. */
function Toggle({
  labelledBy,
  describedBy,
  on,
  onChange,
}: {
  labelledBy: string;
  describedBy: string;
  on: boolean;
  onChange: (next: boolean) => void;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={on}
      aria-labelledby={labelledBy}
      aria-describedby={describedBy}
      onClick={() => onChange(!on)}
      className={`relative h-5 w-[34px] flex-none rounded-pill transition-colors duration-[80ms] ${
        on ? "bg-primary" : "bg-border-strong"
      } hover:shadow-e2 ${FOCUS_RING}`}
    >
      <span
        aria-hidden
        className="absolute top-0.5 left-0.5 h-4 w-4 rounded-pill bg-card shadow-e1 transition-transform duration-[80ms]"
        style={{ transform: on ? "translateX(14px)" : "none" }}
      />
    </button>
  );
}

function SetRow({
  label,
  help,
  side,
  control,
}: {
  label: string;
  help: string;
  side?: ReactNode;
  control?: { on: boolean; onChange: (next: boolean) => void };
}) {
  const labelId = useId();
  const helpId = useId();
  return (
    <div className="flex items-start gap-4 border-t border-border py-3.5 last:border-b">
      <div className="min-w-0 flex-1">
        <span id={labelId} className="block font-ui text-[14px] font-semibold leading-[1.4] text-foreground">
          {label}
        </span>
        <p id={helpId} className="mt-[3px] max-w-[70ch] font-ui text-[13px] leading-[1.45] text-body">
          {help}
        </p>
      </div>
      <div className="flex flex-none items-center gap-2.5">
        {control ? (
          <Toggle labelledBy={labelId} describedBy={helpId} on={control.on} onChange={control.onChange} />
        ) : (
          side
        )}
      </div>
    </div>
  );
}

export default function PreferencesStep({
  values,
  onToggle,
}: {
  values: PreferenceValues;
  onToggle: (field: PreferenceField, next: boolean) => void;
}) {
  return (
    <div className="flex flex-col">
      <span className={EYEBROW}>Step 4 of 6</span>
      <h2 data-step-heading tabIndex={-1} className={STEP_HEADING}>
        Defaults.
      </h2>
      <p className={`${BODY} mt-2 max-w-[50ch]`}>All four are also in Settings.</p>

      <div className="mt-4">
        <SetRow
          label="Offline mode"
          help="Nothing leaves this Mac. Answers that need the cloud ask first."
          control={{ on: values.offline, onChange: (next) => onToggle("offline", next) }}
        />
        <SetRow
          label="Launch at login"
          help="Ruoxi waits in the menu bar."
          control={{ on: values.launch_at_login, onChange: (next) => onToggle("launch_at_login", next) }}
        />
        <SetRow
          label="Read answers aloud"
          help="Esc stops the audio."
          control={{ on: values.read_aloud, onChange: (next) => onToggle("read_aloud", next) }}
        />
        <SetRow
          label="Automatic capture"
          help="Off for every app. Turn it on per app in Settings."
          side={<span className={TAG}>Off</span>}
        />
      </div>
    </div>
  );
}
