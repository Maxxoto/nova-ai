import type { PermissionKind } from "../../permissions";

/**
 * The 6-step onboarding ritual (J8 · AC-12), matching the new OpenDesign flow:
 * welcome → permissions → first-capture → preferences → models → ready.
 */
export const RITUAL_STEPS = [
  "welcome",
  "permissions",
  "first-capture",
  "preferences",
  "models",
  "ready",
] as const;

export type RitualStep = (typeof RITUAL_STEPS)[number];

/**
 * Legacy `?step=` slugs from the pre-refresh D-pad flow. They resolve to the
 * step that now carries the same responsibility so old dev links keep working.
 * `models` needs no alias — it is now a first-class step.
 */
const LEGACY_STEP_ALIASES: Record<string, RitualStep> = {
  screen: "permissions",
  microphone: "permissions",
  accessibility: "permissions",
  launch: "preferences",
  finale: "ready",
};

export function isRitualStep(value: string | null): value is RitualStep {
  return value !== null && (RITUAL_STEPS as readonly string[]).includes(value);
}

/** Canonical parse for the `?step=` dev param; unknown/missing → `welcome`. */
export function parseRitualStep(value: string | null): RitualStep {
  if (isRitualStep(value)) return value;
  if (value !== null && value in LEGACY_STEP_ALIASES) return LEGACY_STEP_ALIASES[value];
  return "welcome";
}

export interface PermissionStepSpec {
  kind: PermissionKind;
  name: string;
  /** Her voice — the reason, shown before the OS dialog. Inter 14/600. */
  why: string;
  /** macOS path hint. Mono caption. */
  path: string;
  /** The OS-dialog quote the user is about to see. */
  dialog: string;
}

/**
 * Permission copy verbatim from the new design, with one honesty fix: the
 * design's Accessibility why-line names the mock ⌥⇧R, and that one IS real —
 * the region accelerator is Alt+Shift+R (shell/src-tauri/src/hotkeys.rs ·
 * ALL_ACCELERATORS), which macOS renders as ⌥ ⇧ R.
 *
 * The quoted app name stays `Ruòxī`: that is the name macOS prints in the
 * system dialog, so the preview shows exactly what the user will see.
 */
export const PERMISSION_STEPS: PermissionStepSpec[] = [
  {
    kind: "screen_recording",
    name: "Screen Recording",
    why: "So Ruoxi can see the part of the screen you point at.",
    path: "System Settings → Privacy & Security → Screen Recording",
    dialog: "“Ruòxī” would like to record this computer's screen.",
  },
  {
    kind: "microphone",
    name: "Microphone",
    why: "So Ruoxi can hear the question while you hold the key.",
    path: "System Settings → Privacy & Security → Microphone",
    dialog: "“Ruòxī” would like to access the microphone.",
  },
  {
    kind: "accessibility",
    name: "Accessibility",
    why: "So ⌥⇧R works while you are inside another app.",
    path: "System Settings → Privacy & Security → Accessibility",
    dialog: "“Ruòxī” would like to control this computer using accessibility features.",
  },
];

/**
 * The accelerators this build actually registers
 * (shell/src-tauri/src/hotkeys.rs · ALL_ACCELERATORS). Kept for reference and
 * parity with the Settings surface.
 */
export const REGISTERED_CAPTURE_ACCELERATORS = [
  { keys: "Alt+Shift+R", intent: "capture a region" },
  { keys: "Alt+Shift+W", intent: "capture a window" },
  { keys: "Alt+Shift+F", intent: "capture the full screen" },
] as const;

/**
 * Push-to-talk accelerator. Matches the shell default
 * (`shell/src-tauri/src/settings.rs` · `default_ptt_hotkey` = `Alt+Shift+V`),
 * which the Ready step renders mac-style as `⌥ ⇧ V`.
 */
export const PTT_KEY_LABEL = "Alt+Shift+V";
