import { useCallback, useEffect, useRef, useState } from "react";
import { getPermissionsStatus, requestPermission } from "../../permissions";
import type { PermissionKind, PermissionsStatus } from "../../permissions";
import { invokeTauriAsync } from "../../tauri";
import FirstCaptureStep from "./FirstCaptureStep";
import PermissionStep from "./PermissionStep";
import PreferencesStep from "./PreferencesStep";
import ReadyStep from "./ReadyStep";
import StepRail from "./StepRail";
import WelcomeStep from "./WelcomeStep";
import { CAPTION, GHOST_BUTTON, PRIMARY_BUTTON } from "./styles";
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
  const [captureDone, setCaptureDone] = useState(false);
  const [preferences, setPreferences] = useState<PreferenceValues>(DEFAULT_PREFERENCES);
  const [completed, setCompleted] = useState(false);
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

  const handleCaptureChange = useCallback((captured: boolean) => {
    setCaptureDone(captured);
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
    if (step === "ready") {
      setCompleted(true);
      window.setTimeout(() => invokeTauriAsync("hide_onboarding"), 900);
      return;
    }
    go(stepIndex + 1);
  };

  const handleSkip = () => {
    setSkipped(true);
    go(RITUAL_STEPS.indexOf("preferences"));
  };

  const capturePermissionsGranted = status.screen_recording && status.accessibility;
  const micUnreadable = status.microphone === "unknown";
  const blocked = step === "permissions" && isTauri && !capturePermissionsGranted;

  const gateMessage = (): string => {
    if (completed) return "Setup saved. The tray is in your menu bar.";
    if (skipped && step === "preferences") {
      return "Skipped the ritual — permissions can be granted later from Settings → Permissions.";
    }
    if (step === "permissions") {
      if (capturePermissionsGranted && !micUnreadable) return "All three granted. Ready when you are.";
      if (capturePermissionsGranted) {
        return "Screen Recording and Accessibility are on — macOS doesn't report the microphone's status to Ruòxī; grant it in System Settings for voice.";
      }
      return isTauri
        ? "Grant Screen Recording and Accessibility to continue — or skip and finish later."
        : "Grant the permissions in the real app — this browser preview can continue.";
    }
    if (step === "first-capture" && !captureDone) return "Box the paragraph to see it work — Continue stays open.";
    if (step === "ready") return "You can re-run setup from Settings → Permissions.";
    return "";
  };

  return (
    <section
      aria-label="Ruòxī onboarding"
      className={`w-full max-w-[680px]${reducedMotion ? " reduced-motion" : ""}`}
    >
      <div className="overflow-hidden rounded-xl border border-border bg-card shadow-e1">
        <StepRail current={step} />

        <div ref={stepRegionRef} aria-live="polite" aria-atomic="true" className="px-7 py-7">
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
            {step === "first-capture" && (
              <FirstCaptureStep permissionsStatus={status} onCaptureChange={handleCaptureChange} />
            )}
            {step === "preferences" && <PreferencesStep values={preferences} onToggle={handleToggle} />}
            {step === "ready" && <ReadyStep pttHotkey={pttHotkey} />}
          </div>
        </div>

        <div className="flex items-center gap-3 border-t border-border px-5 py-3">
          <button type="button" onClick={() => go(stepIndex - 1)} disabled={stepIndex === 0} className={GHOST_BUTTON}>
            Back
          </button>
          <span aria-live="polite" className={`${CAPTION} min-w-0 flex-1`}>
            {gateMessage()}
          </span>
          <button type="button" onClick={handleSkip} className={GHOST_BUTTON}>
            Skip setup
          </button>
          <button
            type="button"
            onClick={handleContinue}
            disabled={blocked}
            aria-disabled={blocked ? "true" : "false"}
            className={blocked ? `${GHOST_BUTTON} border border-border` : PRIMARY_BUTTON}
          >
            {step === "ready" ? "Finish setup" : "Continue"}
          </button>
        </div>
      </div>
    </section>
  );
}

/*
 * Dev params:
 *   ?view=onboarding
 *   ?step=welcome | permissions | first-capture | preferences | ready
 *   legacy: screen | microphone | accessibility → permissions, launch → preferences, finale → ready
 * Missing or unknown `step` falls back to `welcome`.
 */
