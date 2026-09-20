import { useCallback, useEffect, useRef, useState } from "react";
import { getPermissionsStatus, requestPermission } from "../../permissions";
import type { PermissionKind, PermissionsStatus } from "../../permissions";
import { invokeTauriAsync } from "../../tauri";
import FirstCaptureStep from "./FirstCaptureStep";
import ModelsStep from "./ModelsStep";
import PermissionStep from "./PermissionStep";
import PreferencesStep from "./PreferencesStep";
import ReadyStep from "./ReadyStep";
import StepRail from "./StepRail";
import WelcomeStep from "./WelcomeStep";
import { CAPTION, GHOST_BUTTON_SM, PRIMARY_BUTTON, SECONDARY_BUTTON } from "./styles";
import { PTT_KEY_LABEL, RITUAL_STEPS } from "./types";
import type { PreferenceField, PreferenceValues } from "./PreferencesStep";
import type { RitualStep } from "./types";

type SettingsRecord = Record<string, unknown>;

function boolField(source: SettingsRecord, key: string, fallback: boolean): boolean {
  const value = source[key];
  return typeof value === "boolean" ? value : fallback;
}

const DEFAULT_PREFERENCES: PreferenceValues = { offline: true, launch_at_login: false, read_aloud: false };

export interface OnboardingRitualProps {
  initialStep: RitualStep;
  reducedMotion: boolean;
}

export default function OnboardingRitual({ initialStep, reducedMotion }: OnboardingRitualProps) {
  const [step, setStep] = useState<RitualStep>(initialStep);
  const [status, setStatus] = useState<PermissionsStatus>({
    screen_recording: false,
    microphone: "unknown",
    accessibility: false,
  });
  const [pendingKind, setPendingKind] = useState<PermissionKind | null>(null);
  const [demoDone, setDemoDone] = useState(false);
  const [preferences, setPreferences] = useState<PreferenceValues>(DEFAULT_PREFERENCES);
  const [skipped, setSkipped] = useState(false);
  const [pttHotkey, setPttHotkey] = useState(PTT_KEY_LABEL);

  const settingsRef = useRef<SettingsRecord | null>(null);
  const stepRegionRef = useRef<HTMLDivElement>(null);

  const isTauri = !!window.__TAURI_INTERNALS__?.invoke;

  const refresh = useCallback(async () => {
    setStatus(await getPermissionsStatus());
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  useEffect(() => {
    const onFocus = () => {
      void refresh();
    };
    const onVisibility = () => {
      if (document.visibilityState === "visible") void refresh();
    };
    window.addEventListener("focus", onFocus);
    document.addEventListener("visibilitychange", onVisibility);
    return () => {
      window.removeEventListener("focus", onFocus);
      document.removeEventListener("visibilitychange", onVisibility);
    };
  }, [refresh]);

  useEffect(() => {
    const pending = invokeTauriAsync("get_settings");
    if (!pending) return;
    let active = true;
    pending.then(
      (raw) => {
        if (!active || !raw || typeof raw !== "object") return;
        const record = raw as SettingsRecord;
        settingsRef.current = record;
        setPreferences({
          offline: boolField(record, "offline", DEFAULT_PREFERENCES.offline),
          launch_at_login: boolField(record, "launch_at_login", DEFAULT_PREFERENCES.launch_at_login),
          read_aloud: boolField(record, "read_aloud", DEFAULT_PREFERENCES.read_aloud),
        });
        const hotkey = typeof record.ptt_hotkey === "string" ? record.ptt_hotkey.trim() : "";
        setPttHotkey(hotkey.length > 0 ? hotkey : PTT_KEY_LABEL);
      },
      () => undefined,
    );
    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    stepRegionRef.current?.querySelector<HTMLElement>("[data-step-heading]")?.focus();
  }, [step]);

  const stepIndex = RITUAL_STEPS.indexOf(step);

  const go = (index: number) => {
    setStep(RITUAL_STEPS[Math.min(RITUAL_STEPS.length - 1, Math.max(0, index))]);
  };

  const handleRequest = async (kind: PermissionKind) => {
    setPendingKind(kind);
    try {
      setStatus(await requestPermission(kind));
    } finally {
      setPendingKind(null);
    }
  };

  const handleDemoDone = useCallback((done: boolean) => {
    setDemoDone(done);
  }, []);

  const handleToggle = (field: PreferenceField, next: boolean) => {
    setPreferences((prev) => ({ ...prev, [field]: next }));
    const current = settingsRef.current;
    if (!current) return;
    const updated = { ...current, [field]: next };
    settingsRef.current = updated;
    invokeTauriAsync("set_settings", { settings: updated })?.catch(() => undefined);
  };

  const handleContinue = () => {
    if (blocked) return;
    if (step === "ready") {
      window.setTimeout(() => invokeTauriAsync("hide_onboarding"), 900);
      return;
    }
    go(stepIndex + 1);
  };

  const handleSkip = () => {
    setSkipped(true);
    go(RITUAL_STEPS.indexOf("preferences"));
  };

  // Honest relaxation: macOS may not report the microphone, so its status is
  // never a blocker — only Screen Recording + Accessibility gate the ritual.
  // Outside Tauri the status is always all-false, so the browser preview is
  // exempt from the gate (it can continue) and says so explicitly.
  const capturePermissionsGranted = status.screen_recording && status.accessibility;
  const blocked =
    (step === "permissions" && isTauri && !capturePermissionsGranted) || (step === "first-capture" && !demoDone);

  const gateMessage = (): string => {
    if (step === "permissions") {
      if (!isTauri) return "Grant the permissions in the real app — this browser preview can continue.";
      if (!capturePermissionsGranted) return "Grant all three to continue.";
    }
    if (step === "first-capture" && !demoDone) return "Run the capture to continue.";
    if (skipped && step === "preferences") return "You can grant access later in Settings.";
    return "";
  };

  return (
    <div className="w-full overflow-hidden bg-card">
      <StepRail current={step} />

      <div
        ref={stepRegionRef}
        aria-live="polite"
        aria-atomic="true"
        className="min-h-[320px] px-[30px] py-7"
      >
        <div key={step} className="ritual-step">
          {step === "welcome" && <WelcomeStep />}
          {step === "permissions" && (
            <PermissionStep
              status={status}
              pendingKind={pendingKind}
              onRequest={(kind) => {
                void handleRequest(kind);
              }}
            />
          )}
          {step === "first-capture" && <FirstCaptureStep reducedMotion={reducedMotion} onDemoDone={handleDemoDone} />}
          {step === "preferences" && <PreferencesStep values={preferences} onToggle={handleToggle} />}
          {step === "models" && (
            <ModelsStep offline={preferences.offline} onTurnOffOffline={() => handleToggle("offline", false)} />
          )}
          {step === "ready" && <ReadyStep pttHotkey={pttHotkey} offline={preferences.offline} />}
        </div>
      </div>

      <div className="flex items-center gap-3 border-t border-border px-[18px] py-3.5">
        <button
          type="button"
          onClick={() => go(stepIndex - 1)}
          disabled={stepIndex === 0}
          className={GHOST_BUTTON_SM}
        >
          Back
        </button>
        <span aria-live="polite" className={`${CAPTION} min-w-0 flex-1`}>
          {gateMessage()}
        </span>
        <button type="button" onClick={handleSkip} className={GHOST_BUTTON_SM}>
          Skip
        </button>
        <button
          type="button"
          onClick={handleContinue}
          aria-disabled={blocked ? "true" : "false"}
          className={blocked ? SECONDARY_BUTTON : PRIMARY_BUTTON}
        >
          {step === "ready" ? "Finish" : "Continue"}
        </button>
      </div>
    </div>
  );
}

/*
 * Dev params:
 *   ?view=onboarding
 *   ?step=welcome | permissions | first-capture | preferences | models | ready
 *   legacy: screen | microphone | accessibility → permissions, launch → preferences, finale → ready
 * Missing or unknown `step` falls back to `welcome`.
 */
