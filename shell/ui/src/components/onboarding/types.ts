import type { PermissionKind } from "../../permissions";

/**
 * The 5-step onboarding ritual (J8 · AC-12), refreshed to the new OpenDesign
 * flow: welcome → permissions → first-capture → preferences → ready.
 */
export const RITUAL_STEPS = [
  "welcome",
  "permissions",
  "first-capture",
  "preferences",
  "ready",
] as const;

export type RitualStep = (typeof RITUAL_STEPS)[number];

/**
 * Legacy `?step=` slugs from the pre-refresh D-pad flow. They resolve to the
 * step that now carries the same responsibility so old dev links keep working.
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
  /** Her voice — the reason, shown before the OS dialog. Nunito. */
  why: string;
  /** macOS path hint. Mono caption. */
  path: string;
  /** The OS-dialog quote the user is about to see. */
  dialog: string;
  /** The quiet reassurance appended to the dialog quote. */
  dialogNote: string;
}

/**
 * Permission copy verbatim from the new design, with one honesty fix: the
 * design's Accessibility why-line names the mock ⌘⇧4, which is NOT a
 * registered accelerator. The real region accelerator is Alt+Shift+R
 * (shell/src-tauri/src/hotkeys.rs · ALL_ACCELERATORS).
 */
export const PERMISSION_STEPS: PermissionStepSpec[] = [
  {
    kind: "screen_recording",
    name: "Screen Recording",
    why: "So I can see the part of your screen you point at — and only that part.",
    path: "macOS · System Settings → Privacy & Security → Screen Recording",
    dialog: "“Ruòxī” would like to record this computer's screen.",
    dialogNote: "Nothing is captured until you ask.",
  },
  {
    kind: "microphone",
    name: "Microphone",
    why: "So I can hear the question you speak — while you hold the key, and not a moment longer.",
    path: "macOS · Privacy & Security → Microphone · audio deleted ≈1 min after release",
    dialog: "“Ruòxī” would like to access the microphone.",
    dialogNote: "Push-to-talk only.",
  },
  {
    kind: "accessibility",
    name: "Accessibility",
    why: "So Alt+Shift+R works while you're inside another app — that's all I use it for.",
    path: "macOS · Privacy & Security → Accessibility · global hotkey registration only",
    dialog: "“Ruòxī” would like to control this computer using accessibility features.",
    dialogNote: "Hotkeys only — never clicks or typing.",
  },
];

/**
 * The accelerators this build actually registers
 * (shell/src-tauri/src/hotkeys.rs · ALL_ACCELERATORS). The design mock shows
 * ⌘⇧4 / ⌘⇧Space, which are not registered — the UI prints these instead.
 */
export const REGISTERED_CAPTURE_ACCELERATORS = [
  { keys: "Alt+Shift+R", intent: "capture a region" },
  { keys: "Alt+Shift+W", intent: "capture a window" },
  { keys: "Alt+Shift+F", intent: "capture the full screen" },
] as const;

/**
 * Push-to-talk key. `DEFAULT_PTT_KEYCODE = 100` (shell/src-tauri/src/ptt.rs)
 * is macOS virtual keycode kVK_F8 — hold to talk.
 */
export const PTT_KEY_LABEL = "F8";
