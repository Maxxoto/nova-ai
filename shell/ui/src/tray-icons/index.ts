// Registry of Ruòxī tray icon sources.
// SVGs are single-source: colored (light palette, per docs/DESIGN.md state table)
// and -template (pure black + alpha for macOS). The export script renders both
// to the PNG matrix in shell/src-tauri/icons/tray/.
import idle from "./svg/tray-idle.svg?raw";
import idleTemplate from "./svg/tray-idle-template.svg?raw";
import idleDawnrise from "./svg/tray-idle-dawnrise.svg?raw";
import idleDawndot from "./svg/tray-idle-dawndot.svg?raw";
import idleCompanion from "./svg/tray-idle-companion.svg?raw";
import listening from "./svg/tray-listening.svg?raw";
import listeningTemplate from "./svg/tray-listening-template.svg?raw";
import thinking from "./svg/tray-thinking.svg?raw";
import thinkingTemplate from "./svg/tray-thinking-template.svg?raw";
import speaking from "./svg/tray-speaking.svg?raw";
import speakingTemplate from "./svg/tray-speaking-template.svg?raw";
import error from "./svg/tray-error.svg?raw";
import errorTemplate from "./svg/tray-error-template.svg?raw";
import degraded from "./svg/tray-degraded.svg?raw";
import degradedTemplate from "./svg/tray-degraded-template.svg?raw";
import capturesPaused from "./svg/tray-captures-paused.svg?raw";
import capturesPausedTemplate from "./svg/tray-captures-paused-template.svg?raw";
import offline from "./svg/tray-offline.svg?raw";
import offlineTemplate from "./svg/tray-offline-template.svg?raw";
import localOnly from "./svg/tray-local-only.svg?raw";
import localOnlyTemplate from "./svg/tray-local-only-template.svg?raw";
import callingCloud from "./svg/tray-calling-cloud.svg?raw";
import callingCloudTemplate from "./svg/tray-calling-cloud-template.svg?raw";

export type TrayAnim = "breathe" | "pulse" | "orbit" | "wave";

export interface TrayStateDef {
  id: TrayStateId;
  /** Tray-menu / panel label — Ruòxī's voice (Nunito on the board). */
  label: string;
  /** Full honest-state label where DESIGN.md specifies one. */
  fullLabel?: string;
  /** Light-theme state color token, per the DESIGN.md honest-states table. */
  colorToken: string;
  /** Resolved light/dark hexes for the board's color readouts (from DESIGN.md front matter). */
  hexLight: string;
  hexDark: string;
  /** Tailwind text class carrying that token. */
  colorClass: string;
  svg: string;
  templateSvg: string;
  /** The loop this state owns when it is genuinely live (DESIGN.md motion law). */
  anim?: TrayAnim;
  /** Motion-law name of that loop, shown on the board. */
  loopName?: string;
}

export type TrayStateId =
  | "idle"
  | "listening"
  | "thinking"
  | "speaking"
  | "error"
  | "degraded"
  | "captures-paused"
  | "offline"
  | "local-only"
  | "calling-cloud";

export const TRAY_STATES: TrayStateDef[] = [
  {
    id: "idle",
    label: "idle",
    colorToken: "{colors.mute}",
    hexLight: "#74655a",
    hexDark: "#a39386",
    colorClass: "text-muted-foreground",
    svg: idle,
    templateSvg: idleTemplate,
    anim: "breathe",
    loopName: "motion.breathe · 3s",
  },
  {
    id: "listening",
    label: "listening",
    colorToken: "{colors.live}",
    hexLight: "#d16405",
    hexDark: "#f7a033",
    colorClass: "text-live",
    svg: listening,
    templateSvg: listeningTemplate,
    anim: "pulse",
    loopName: "motion.pulse-ring · 1.2s",
  },
  {
    id: "thinking",
    label: "thinking",
    colorToken: "{colors.thinking-text}",
    hexLight: "#35526e",
    hexDark: "#7fb3d5",
    colorClass: "text-thinking-text",
    svg: thinking,
    templateSvg: thinkingTemplate,
    anim: "orbit",
    loopName: "motion.orbit · 1.6s",
  },
  {
    id: "speaking",
    label: "speaking",
    colorToken: "{colors.primary}",
    hexLight: "#c55026",
    hexDark: "#e07a4e",
    colorClass: "text-primary",
    svg: speaking,
    templateSvg: speakingTemplate,
    anim: "wave",
    loopName: "motion.wave · 0.9s (mic level)",
  },
  {
    id: "calling-cloud",
    label: "sending to cloud",
    fullLabel: "sending to cloud",
    colorToken: "{colors.live} (pulse)",
    hexLight: "#d16405",
    hexDark: "#f7a033",
    colorClass: "text-live",
    svg: callingCloud,
    templateSvg: callingCloudTemplate,
    anim: "pulse",
    loopName: "motion.pulse-ring · 1.2s",
  },
  {
    id: "offline",
    label: "offline",
    fullLabel: "offline · nothing leaves this Mac",
    colorToken: "{colors.mute}",
    hexLight: "#74655a",
    hexDark: "#a39386",
    colorClass: "text-muted-foreground",
    svg: offline,
    templateSvg: offlineTemplate,
  },
  {
    id: "local-only",
    label: "local only",
    colorToken: "{colors.success}",
    hexLight: "#267326",
    hexDark: "#4d9e5d",
    colorClass: "text-success",
    svg: localOnly,
    templateSvg: localOnlyTemplate,
  },
  {
    id: "error",
    label: "error",
    colorToken: "{colors.destructive}",
    hexLight: "#bb2b1f",
    hexDark: "#ee6a63",
    colorClass: "text-destructive",
    svg: error,
    templateSvg: errorTemplate,
  },
  {
    id: "degraded",
    label: "Ruòxī is resting…",
    colorToken: "{colors.warning}",
    hexLight: "#9a4a04",
    hexDark: "#f97d10",
    colorClass: "text-warning",
    svg: degraded,
    templateSvg: degradedTemplate,
  },
  {
    id: "captures-paused",
    label: "captures paused",
    colorToken: "{colors.mute}",
    hexLight: "#74655a",
    hexDark: "#a39386",
    colorClass: "text-muted-foreground",
    svg: capturesPaused,
    templateSvg: capturesPausedTemplate,
  },
];

/** Idle-mark exploration concepts (review board only — B is the shipped idle mark). */
export interface IdleConceptDef {
  id: "dawnrise" | "dawndot" | "companion";
  name: string;
  rationale: string;
  svg: string;
}

export const IDLE_CONCEPTS: IdleConceptDef[] = [
  {
    id: "dawnrise",
    name: "A · Dawn Rise",
    rationale:
      "The sun clearing the horizon — 若曦 as dawn breaking. The horizon line anchors the disc so the mark reads as a scene, not a stray dot, even at 16px.",
    svg: idleDawnrise,
  },
  {
    id: "dawndot",
    name: "B · Dawn Dot",
    rationale:
      "The orb itself shrunk to its essence: a coral disc with a soft inner-highlight crescent of first light. One-to-one with DESIGN.md's 'tray = dawn dot' and the panel avatar — strongest brand continuity.",
    svg: idleDawndot,
  },
  {
    id: "companion",
    name: "C · Companion Cradle",
    rationale:
      "A gentle arc cradles the dot — kawaii by shape alone, no face. Reads as Ruòxī resting quietly in the menu bar, but the second element costs contrast at 16px.",
    svg: idleCompanion,
  },
];
