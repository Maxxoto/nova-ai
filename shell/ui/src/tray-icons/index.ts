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
import capturesPaused from "./svg/tray-captures-paused.svg?raw";
import capturesPausedTemplate from "./svg/tray-captures-paused-template.svg?raw";

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

export type TrayStateId = "idle" | "listening" | "captures-paused";

export const TRAY_STATES: TrayStateDef[] = [
  {
    id: "idle",
    label: "ready",
    fullLabel: "ready · idle, offline, everything unlit",
    colorToken: "{colors.mute}",
    hexLight: "#64748c",
    hexDark: "#8a97ac",
    colorClass: "text-muted-foreground",
    svg: idle,
    templateSvg: idleTemplate,
  },
  {
    id: "listening",
    label: "listening",
    fullLabel: "listening · mic open",
    colorToken: "{colors.live}",
    hexLight: "#d16405",
    hexDark: "#f7a033",
    colorClass: "text-live",
    svg: listening,
    templateSvg: listeningTemplate,
    anim: "pulse",
    loopName: "pulse-ring",
  },
  {
    id: "captures-paused",
    label: "captures paused",
    colorToken: "{colors.mute}",
    hexLight: "#64748c",
    hexDark: "#8a97ac",
    colorClass: "text-muted-foreground",
    svg: capturesPaused,
    templateSvg: capturesPausedTemplate,
  },
];

/** Idle-mark exploration concepts (review board history — the v4 capture frame now ships as the resting mark). */
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
      "The orb itself shrunk to its essence: a blue disc with a soft inner-highlight crescent of first light. One-to-one with DESIGN.md's 'tray = dawn dot' and the panel avatar — strongest brand continuity.",
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
