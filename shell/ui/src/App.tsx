import { useEffect, useState } from "react";
import { IDLE_CONCEPTS, TRAY_STATES } from "./tray-icons";
import ConceptCard from "./components/ConceptCard";
import MenuBar from "./components/MenuBar";
import Section from "./components/Section";
import StateSet from "./components/StateSet";
import TemplateCompare from "./components/TemplateCompare";
import TrayMark from "./components/TrayMark";

const SHIPPED_IDLE = IDLE_CONCEPTS[1]; /* B · Dawn Dot — matches src/tray-icons/svg/tray-idle.svg */

function ReducedMotionToggle({
  on,
  onChange,
}: {
  on: boolean;
  onChange: (next: boolean) => void;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={on}
      onClick={() => onChange(!on)}
      className="flex items-center gap-2.5 rounded-pill border border-border bg-card py-1.5 pl-3 pr-2.5 shadow-e1 transition-colors duration-200 hover:border-border-strong focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring"
    >
      <span className="font-ui text-[13px] font-semibold text-foreground">reduced motion</span>
      <span
        className={`relative inline-block rounded-pill transition-colors duration-200 ${
          on ? "bg-primary" : "bg-muted"
        }`}
        style={{ height: 18, width: 32 }}
      >
        <span
          className="absolute top-0.5 h-3.5 w-3.5 rounded-pill bg-card shadow-e1 transition-all duration-200"
          style={{ left: on ? 16 : 2 }}
        />
      </span>
    </button>
  );
}

export default function App() {
  const [reducedMotion, setReducedMotion] = useState(false);

  useEffect(() => {
    const mq = window.matchMedia("(prefers-reduced-motion: reduce)");
    if (mq.matches) setReducedMotion(true);
    const onChange = (e: MediaQueryListEvent) => setReducedMotion(e.matches);
    mq.addEventListener("change", onChange);
    return () => mq.removeEventListener("change", onChange);
  }, []);

  return (
    <div className={`min-h-screen${reducedMotion ? " reduced-motion rm-halve" : ""}`}>
      <div className="mx-auto flex max-w-[1120px] flex-col gap-12 px-6 pb-20 pt-10">
        <header className="flex flex-col gap-6">
          <div className="flex flex-wrap items-center justify-between gap-4">
            <div className="flex items-center gap-3">
              <TrayMark svg={TRAY_STATES[0].svg} size={28} anim={reducedMotion ? undefined : "breathe"} />
              <div className="flex flex-col">
                <span className="font-companion text-[15px] font-bold text-foreground">
                  Ruòxī · 若曦 — Tray Icon Review Board
                </span>
                <span className="font-mono text-[11px] text-muted-foreground">
                  docs/DESIGN.md · RFC-0002 §4.2 · tray icon only — menu-bar marks + honest states
                </span>
              </div>
            </div>
            <ReducedMotionToggle on={reducedMotion} onChange={setReducedMotion} />
          </div>

          <div className="flex flex-col gap-2 rounded-lg border border-border bg-card p-4 shadow-e1">
            <MenuBar theme="light">
              <span className="menubar-chip menubar-chip--light">
                <TrayMark svg={TRAY_STATES[1].svg} size={18} anim={reducedMotion ? undefined : "pulse"} />
                <span className="font-companion text-[13px] font-semibold">listening</span>
              </span>
            </MenuBar>
            <MenuBar theme="dark">
              <span className="menubar-chip menubar-chip--dark">
                <TrayMark svg={TRAY_STATES[1].svg} size={18} anim={reducedMotion ? undefined : "pulse"} />
                <span className="font-companion text-[13px] font-semibold">listening</span>
              </span>
            </MenuBar>
            <p className="font-mono text-[11px] text-muted-foreground">
              The bar it lives in — 18px mark, selected-item chrome, honest label next to the mark.
            </p>
          </div>
        </header>

        <Section
          eyebrow="§ 1 · idle mark"
          title="Three candidates for the resting mark"
          lede="The tray idle state is the dawn dot (DESIGN.md AI State System: “tray = dawn dot”). Each concept below is the resting Ruòxī at real menu-bar sizes, on both bars."
        >
          <div className="grid grid-cols-1 gap-4 lg:grid-cols-3">
            {IDLE_CONCEPTS.map((c) => (
              <ConceptCard key={c.id} concept={c} recommended={c.id === SHIPPED_IDLE.id} />
            ))}
          </div>
          <p className="rounded-md border border-border bg-muted p-3 text-[13px] leading-[1.45] text-foreground">
            <span className="font-semibold">Verdict — B · Dawn Dot ships.</span> It is the orb-avatar at tray
            scale, so menu bar and panel read as one companion; A and C stay on the board as fallbacks if the
            crescent ever proves noisy on Windows taskbars.
          </p>
        </Section>

        <Section
          eyebrow="§ 2 · honest states"
          title="The full state set — icon + label, exact state colors"
          lede="Ten states from the DESIGN.md honest-states table. Only genuinely-live conditions loop (breathe / pulse-ring / orbit / wave); everything else is static. The reduced-motion switch freezes loops and halves durations, per the motion law."
        >
          <StateSet reducedMotion={reducedMotion} />
          <p className="font-mono text-[11px] text-muted-foreground">
            calling-cloud mirrors listening’s ember pulse — egress is live egress; the cloud glyph carries the
            difference. Tray mirrors offline + paused (F-09, F-13) so the tray alone tells the truth.
          </p>
        </Section>

        <Section
          eyebrow="§ 3 · macOS template"
          title="Colored vs template, side by side"
          lede="Template images are pure black + alpha; macOS tints them for light/dark menus. The tinted column simulates that tint with {colors.dark-ink} — the shipped PNG stays black."
        >
          <TemplateCompare />
        </Section>

        <footer className="flex flex-col gap-1 border-t border-border pt-5">
          <p className="font-mono text-[11px] text-muted-foreground">
            Sources: shell/ui/src/tray-icons/svg/ · Export: pnpm export:tray-icons → shell/src-tauri/icons/tray/ ·
            Wiring: shell/ui/README.md (no .rs edits in this change)
          </p>
        </footer>
      </div>
    </div>
  );
}
