import { useEffect, useState } from "react";
import { invokeTauriAsync } from "../../tauri";
import { useId } from "react";
import type { ReactNode } from "react";
import { BODY, CAPTION, EYEBROW, FOCUS_RING, STEP_HEADING } from "./styles";

export type PreferenceField = "offline" | "launch_at_login" | "read_aloud";

export interface PreferenceValues {
  offline: boolean;
  launch_at_login: boolean;
  read_aloud: boolean;
}

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
      className={`flex flex-none items-center rounded-pill border border-border bg-card p-[3px] shadow-e1 transition-colors duration-200 hover:bg-muted ${FOCUS_RING}`}
    >
      <span
        aria-hidden
        className={`relative inline-block rounded-pill transition-colors duration-200 ${on ? "bg-primary" : "bg-muted"}`}
        style={{ height: 20, width: 36 }}
      >
        <span
          className="absolute top-0.5 h-4 w-4 rounded-pill bg-card shadow-e1 transition-all duration-200"
          style={{ left: on ? 18 : 2 }}
        />
      </span>
    </button>
  );
}

function Tag({ children }: { children: ReactNode }) {
  return (
    <span className="inline-flex w-fit flex-none items-center rounded-pill border border-border bg-muted px-2.5 py-1 font-ui text-[11px] font-medium text-muted-foreground">
      {children}
    </span>
  );
}

function ToggleRow({
  label,
  help,
  on,
  onChange,
}: {
  label: string;
  help: string;
  on: boolean;
  onChange: (next: boolean) => void;
}) {
  const labelId = useId();
  const helpId = useId();
  return (
    <div className="flex items-start justify-between gap-6 border-b border-border px-1 py-3.5 last:border-b-0">
      <div className="flex min-w-0 flex-col gap-0.5">
        <span id={labelId} className="font-ui text-[14px] font-semibold leading-[1.4] text-foreground">
          {label}
        </span>
        <p id={helpId} className="max-w-[70ch] font-ui text-[13px] leading-[1.45] text-muted-foreground">
          {help}
        </p>
      </div>
      <Toggle labelledBy={labelId} describedBy={helpId} on={on} onChange={onChange} />
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
    <div className="flex flex-col gap-5">
      <div className="flex flex-col gap-3">
        <span className={EYEBROW}>Step 4 of 5</span>
        <h2 data-step-heading tabIndex={-1} className={STEP_HEADING}>
          Four defaults, all reversible.
        </h2>
        <p className={BODY}>Every switch here also lives in Settings. Nothing is buried.</p>
      </div>

      <div className="rounded-lg border border-border bg-card px-4">
        <ToggleRow
          label="Offline mode"
          help="Start with nothing leaving this Mac. Answers that need the cloud will ask first, and say what they would send."
          on={values.offline}
          onChange={(next) => onToggle("offline", next)}
        />
        <ToggleRow
          label="Launch at login"
          help="Ruòxī waits in the menu bar, out of the Dock and out of your way."
          on={values.launch_at_login}
          onChange={(next) => onToggle("launch_at_login", next)}
        />
        <ToggleRow
          label="Read answers aloud"
          help="Off by default. When on, the panel shows the live speaking waveform and Esc always stops it."
          on={values.read_aloud}
          onChange={(next) => onToggle("read_aloud", next)}
        />
        <div className="flex items-start justify-between gap-6 border-t border-border px-1 py-3.5">
          <div className="flex min-w-0 flex-col gap-0.5">
            <span className="font-ui text-[14px] font-semibold leading-[1.4] text-foreground">
              Auto-capture in other apps
            </span>
            <p className="max-w-[70ch] font-ui text-[13px] leading-[1.45] text-muted-foreground">
              Off for every app. Turn it on per app in Settings, where each row states what is stored and when it
              captures.
            </p>
          </div>
          <Tag>Off</Tag>
        </div>
      </div>

      <SetupSurfaceRow />

      <p className={CAPTION}>Nothing is buried — every switch here also lives in Settings.</p>
    </div>
  );
}

function SetupSurfaceRow() {
  const [summary, setSummary] = useState("");
  useEffect(() => {
    let active = true;
    void (async () => {
      const models = (await invokeTauriAsync("stt_catalog")) as
        | Array<{ id: string; downloaded: boolean; selected: boolean }>
        | null;
      const settings = (await invokeTauriAsync("get_settings")) as
        | { llm?: { base_url?: string; api_key_set?: boolean } }
        | null;
      if (!active) return;
      const stt = models?.find((m) => m.selected) ?? models?.find((m) => m.downloaded);
      const llmReady = Boolean(settings?.llm?.base_url) && Boolean(settings?.llm?.api_key_set);
      setSummary(
        `Voice model: ${stt ? stt.id : "not downloaded"} · Brain (LLM): ${llmReady ? "configured" : "no key yet"}`,
      );
    })();
    return () => {
      active = false;
    };
  }, []);
  return (
    <div className="flex items-start justify-between gap-6 border-t border-border px-1 py-3.5">
      <div className="flex min-w-0 flex-col gap-0.5">
        <span className="font-ui text-[14px] font-semibold leading-[1.4] text-foreground">
          Models &amp; brain setup
        </span>
        <p className="max-w-[70ch] font-ui text-[13px] leading-[1.45] text-muted-foreground">
          {summary || "Download a voice model, pick the read-aloud voice, and add your LLM key in Settings."}
        </p>
      </div>
      <button
        type="button"
        onClick={() => invokeTauriAsync("show_settings")}
        className="rounded-pill border border-border bg-card px-3 py-1 font-ui text-[12px] font-semibold text-foreground transition-colors duration-200 hover:bg-muted focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring"
      >
        Open Settings
      </button>
    </div>
  );
}
