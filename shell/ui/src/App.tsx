import { useEffect, useState } from "react";
import type { ReactNode } from "react";
import { IDLE_CONCEPTS, TRAY_STATES } from "./tray-icons";
import ConceptCard from "./components/ConceptCard";
import MenuBar from "./components/MenuBar";
import ResultPanel from "./components/panel/ResultPanel";
import { isNetState, isPanelState, NET_STATES, PANEL_STATES } from "./components/panel/types";
import type { NetState, PanelState } from "./components/panel/types";
import Section from "./components/Section";
import StateSet from "./components/StateSet";
import TemplateCompare from "./components/TemplateCompare";
import TrayMark from "./components/TrayMark";
import { invokeTauri } from "./tauri";

const SHIPPED_IDLE = IDLE_CONCEPTS[1]; /* B · Dawn Dot — matches src/tray-icons/svg/tray-idle.svg */

const DEMO_ANSWER =
  "Here's what the capture shows: the test suite is failing on the sidecar handshake — the JSON-RPC read loop times out before the first ping returns. Same class as the earlier respawn failure, which points at the supervisor's restart backoff rather than the protocol itself.";

const DEMO_CAPTURE = { id: "cap_01J8ZF5Q2W9R3T4", time: "14:02" };

const GALLERY: { state: PanelState; net: NetState }[] = [
  { state: "thinking", net: "local_only" },
  { state: "streaming", net: "calling_cloud" },
  { state: "complete", net: "local_only" },
  { state: "speaking", net: "local_only" },
  { state: "error", net: "offline" },
  { state: "degraded", net: "offline" },
];

function PanelStage({
  label,
  dark = false,
  children,
}: {
  label: string;
  dark?: boolean;
  children: ReactNode;
}) {
  return (
    <div className={`flex flex-col gap-2 rounded-xl border border-border p-4 ${dark ? "dark bg-background" : "bg-muted"}`}>
      <span className="font-mono text-[11px] text-muted-foreground">{label}</span>
      <div className="flex flex-1 justify-center py-4">{children}</div>
    </div>
  );
}

function DemoChip({
  active = false,
  onClick,
  children,
}: {
  active?: boolean;
  onClick: () => void;
  children: ReactNode;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`rounded-pill border px-3 py-1.5 font-mono text-[11px] transition-colors duration-200 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring ${
        active ? "border-primary bg-primary-soft text-foreground" : "border-border bg-card text-muted-foreground hover:bg-muted"
      }`}
    >
      {children}
    </button>
  );
}

function DismissedStage({ onShow }: { onShow: () => void }) {
  return (
    <div className="flex min-h-[200px] flex-col items-center justify-center gap-3">
      <span className="font-companion text-[13px] font-semibold text-muted-foreground">panel dismissed — Esc works</span>
      <button
        type="button"
        onClick={onShow}
        className="rounded-pill border border-border bg-card px-3 py-1.5 font-ui text-[13px] font-semibold transition-colors duration-200 hover:bg-muted focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring"
      >
        show again
      </button>
    </div>
  );
}

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
  const [panelState, setPanelState] = useState<PanelState>("thinking");
  const [panelNet, setPanelNet] = useState<NetState>("local_only");
  const [panelVisible, setPanelVisible] = useState(true);

  const params = new URLSearchParams(window.location.search);
  const panelView = params.get("view") === "panel";
  const rawState = params.get("state");
  const rawNet = params.get("net");
  const panelViewState: PanelState = isPanelState(rawState) ? rawState : "streaming";
  const panelViewNet: NetState = isNetState(rawNet) ? rawNet : "local_only";

  useEffect(() => {
    const mq = window.matchMedia("(prefers-reduced-motion: reduce)");
    if (mq.matches) setReducedMotion(true);
    const onChange = (e: MediaQueryListEvent) => setReducedMotion(e.matches);
    mq.addEventListener("change", onChange);
    return () => mq.removeEventListener("change", onChange);
  }, []);

  useEffect(() => {
    const root = document.documentElement;
    if (!panelView) {
      root.classList.remove("panel-view", "dark");
      return;
    }
    root.classList.add("panel-view");
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const sync = () => root.classList.toggle("dark", mq.matches);
    sync();
    mq.addEventListener("change", sync);
    return () => {
      root.classList.remove("panel-view", "dark");
      mq.removeEventListener("change", sync);
    };
  }, [panelView]);

  if (panelView) {
    return (
      <div className={`flex min-h-screen justify-center p-3${reducedMotion ? " reduced-motion rm-halve" : ""}`}>
        <ResultPanel
          state={panelViewState}
          net={panelViewNet}
          answer={DEMO_ANSWER}
          capture={DEMO_CAPTURE}
          reducedMotion={reducedMotion}
          onDismiss={() => invokeTauri("hide_panel")}
        />
      </div>
    );
  }

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
                  docs/DESIGN.md · RFC-0002 §4.2 + §4.4 · menu-bar marks, honest states, and the W4 result panel
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

        <Section
          eyebrow="§ 4 · result panel"
          title="The floating panel — W4"
          lede="DESIGN.md panel-shell: 400px, radius 14, e3 shadow, vibrancy at 78% over the user's work with a 97% fallback. Header = orb + her state label + cloud chip + esc hint; Esc dismisses in 120ms with no side effects (AC-06). M0 content is placeholder — the live pipe lands with the IPC wiring."
        >
          <div className="flex flex-wrap items-center gap-2">
            {PANEL_STATES.map((s) => (
              <DemoChip
                key={s}
                active={s === panelState}
                onClick={() => {
                  setPanelState(s);
                  setPanelVisible(true);
                }}
              >
                {s}
              </DemoChip>
            ))}
            <span className="mx-1 h-5 w-px bg-border" />
            {NET_STATES.map((n) => (
              <DemoChip key={n} active={n === panelNet} onClick={() => setPanelNet(n)}>
                {n}
              </DemoChip>
            ))}
            <DemoChip onClick={() => setPanelVisible((v) => !v)}>
              {panelVisible ? "dismiss (esc works too)" : "show panel"}
            </DemoChip>
          </div>

          <div className="grid grid-cols-1 gap-6 xl:grid-cols-2">
            <PanelStage label="interactive · dawn">
              {panelVisible ? (
                <ResultPanel
                  state={panelState}
                  net={panelNet}
                  answer={DEMO_ANSWER}
                  capture={DEMO_CAPTURE}
                  reducedMotion={reducedMotion}
                  onDismiss={() => setPanelVisible(false)}
                />
              ) : (
                <DismissedStage onShow={() => setPanelVisible(true)} />
              )}
            </PanelStage>
            <PanelStage label="interactive · night before dawn" dark>
              {panelVisible ? (
                <ResultPanel
                  state={panelState}
                  net={panelNet}
                  answer={DEMO_ANSWER}
                  capture={DEMO_CAPTURE}
                  reducedMotion={reducedMotion}
                  onDismiss={() => setPanelVisible(false)}
                />
              ) : (
                <DismissedStage onShow={() => setPanelVisible(true)} />
              )}
            </PanelStage>
          </div>

          <div className="flex flex-col gap-4">
            <span className="font-mono text-[11px] text-muted-foreground">state gallery — dawn / night</span>
            <div className="grid grid-cols-1 gap-6 xl:grid-cols-2">
              {GALLERY.flatMap(({ state, net }) => [
                <PanelStage key={`dawn-${state}`} label={`${state} · ${net} · dawn`}>
                  <ResultPanel
                    state={state}
                    net={net}
                    answer={DEMO_ANSWER}
                    capture={DEMO_CAPTURE}
                    reducedMotion={reducedMotion}
                  />
                </PanelStage>,
                <PanelStage key={`night-${state}`} label={`${state} · ${net} · night`} dark>
                  <ResultPanel
                    state={state}
                    net={net}
                    answer={DEMO_ANSWER}
                    capture={DEMO_CAPTURE}
                    reducedMotion={reducedMotion}
                  />
                </PanelStage>,
              ])}
            </div>
          </div>
        </Section>

        <footer className="flex flex-col gap-1 border-t border-border pt-5">
          <p className="font-mono text-[11px] text-muted-foreground">
            Sources: shell/ui/src/tray-icons/svg/ + src/components/panel/ · Export: pnpm export:tray-icons →
            shell/src-tauri/icons/tray/ · Panel window: shell/src-tauri label “panel” loads ?view=panel
          </p>
        </footer>
      </div>
    </div>
  );
}
