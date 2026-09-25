import { useEffect, useRef, useState } from "react";
import type { ReactNode } from "react";
import { IDLE_CONCEPTS, TRAY_STATES } from "./tray-icons";
import ConceptCard from "./components/ConceptCard";
import MenuBar from "./components/MenuBar";
import OnboardingRitual from "./components/onboarding/OnboardingRitual";
import { parseRitualStep } from "./components/onboarding/types";
import type { RitualStep } from "./components/onboarding/types";
import CaptureOverlay from "./components/overlay/CaptureOverlay";
import type { OverlaySelection } from "./components/overlay/CaptureOverlay";
import MiniMark from "./components/panel/MiniMark";
import ResultPanel from "./components/panel/ResultPanel";
import { isNetState, isPanelState, NET_STATES, PANEL_STATES } from "./components/panel/types";
import type { CaptureInfo, CaptureScope, NetState, PanelState } from "./components/panel/types";
import Section from "./components/Section";
import SettingsWindow from "./components/settings/SettingsWindow";
import StateSet from "./components/StateSet";
import TimelineView from "./components/timeline/TimelineView";
import TemplateCompare from "./components/TemplateCompare";
import TrayMark from "./components/TrayMark";
import { invokeTauri, invokeTauriAsync, listenTauri } from "./tauri";
import type { TauriUnlisten } from "./tauri";

const SHIPPED_IDLE = IDLE_CONCEPTS[1]; /* board record — the tray's resting mark is the v4 capture frame */

type CapturePayload = {
  id: string;
  at_ms?: number;
  time?: string;
  scope?: string;
  width?: number;
  height?: number;
};
type TranscriptPayload = { text: string };
type TokenPayload = { delta: string };
type CompletePayload = { answer: string };
type ErrorPayload = { message: string };

function formatHHMM(atMs: number): string {
  const d = new Date(atMs);
  return `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`;
}

const SCOPE_DIM: Record<CaptureScope, { width: number; height: number }> = {
  region: { width: 412, height: 96 },
  window: { width: 1440, height: 900 },
  screen: { width: 2560, height: 1600 },
};

const HOTKEY_MODIFIER_GLYPHS: Record<string, string> = {
  alt: "⌥",
  option: "⌥",
  shift: "⇧",
  ctrl: "⌃",
  control: "⌃",
  cmd: "⌘",
  command: "⌘",
  meta: "⌘",
  super: "⌘",
};

function formatHotkey(accelerator: unknown): string {
  const raw = typeof accelerator === "string" ? accelerator.trim() : "";
  const parts = raw.split("+").map((part) => part.trim()).filter((part) => part.length > 0);
  if (parts.length === 0) return "⌥⇧V";
  return parts.map((part) => HOTKEY_MODIFIER_GLYPHS[part.toLowerCase()] ?? part).join("");
}

function readScope(raw: unknown): CaptureScope {
  if (raw === "window") return "window";
  if (raw === "screen" || raw === "fullscreen") return "screen";
  return "region";
}

function parseDimension(raw: string | null, fallback: number): number {
  if (raw === null) return fallback;
  const value = Number(raw);
  return Number.isFinite(value) && value > 0 ? Math.round(value) : fallback;
}

const THEMES = ["system", "dawn", "night"] as const;
type ThemeName = (typeof THEMES)[number];

/** Cross-window event: the Settings window fires it so the root theme effect
 *  can re-bind its `prefers-color-scheme` listener after a manual pick. */
const THEME_EVENT = "ruoxi:theme";

function readTheme(raw: unknown): ThemeName {
  if (raw && typeof raw === "object") {
    const value = (raw as { theme?: unknown }).theme;
    if (typeof value === "string" && (THEMES as readonly string[]).includes(value)) return value as ThemeName;
  }
  return "system";
}

/** Dev QA static selection for `?view=overlay&sel=x,y,w,h`; undefined when absent/invalid. */
function parseOverlaySelection(raw: string | null): OverlaySelection | undefined {
  if (!raw) return undefined;
  const parts = raw.split(",").map((n) => Number(n));
  if (parts.length !== 4 || parts.some((n) => !Number.isFinite(n))) return undefined;
  const [x, y, w, h] = parts;
  return { x, y, w, h };
}

/** Panel presentation: the full surface, or the collapsed 44px disc. */
type PanelMode = "panel" | "mini";

function readPanelMode(raw: string | null): PanelMode {
  return raw === "mini" ? "mini" : "panel";
}

const DEMO_ANSWER =
  "Here's what the capture shows: the test suite is failing on the **sidecar handshake** — the JSON-RPC read loop times out before the first ping returns. Same class as the earlier respawn failure, which points at the supervisor's **restart backoff** rather than the protocol itself.";

const DEMO_CAPTURE: CaptureInfo = {
  id: "cap_20250923140200",
  time: "14:02",
  scope: "region",
  width: 412,
  height: 96,
};

const DEMO_TRANSCRIPT = "explain this simply";

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
      className={`rounded-pill border px-3 py-1.5 font-mono text-[11px] transition-colors duration-[120ms] ease-[cubic-bezier(.4,0,.2,1)] focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring ${
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
        className="rounded-pill border border-border bg-card px-3 py-1.5 font-ui text-[13px] font-semibold transition-colors duration-[120ms] ease-[cubic-bezier(.4,0,.2,1)] hover:bg-muted focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring"
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
      className="flex items-center gap-2.5 rounded-pill border border-border bg-card py-1.5 pl-3 pr-2.5 shadow-e1 transition-colors duration-[120ms] ease-[cubic-bezier(.4,0,.2,1)] hover:border-border-strong focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring"
    >
      <span className="font-ui text-[13px] font-semibold text-foreground">reduced motion</span>
      <span
        className={`relative inline-block rounded-pill transition-colors duration-[80ms] ${
          on ? "bg-primary" : "bg-muted"
        }`}
        style={{ height: 18, width: 32 }}
      >
        <span
          className="absolute top-0.5 h-3.5 w-3.5 rounded-pill bg-card shadow-e1 transition-all duration-[80ms]"
          style={{ left: on ? 16 : 2 }}
        />
      </span>
    </button>
  );
}

export default function App() {
  const params = new URLSearchParams(window.location.search);
  const [reducedMotion, setReducedMotion] = useState(false);
  const [mode, setMode] = useState<PanelMode>(() => readPanelMode(params.get("mode")));
  const [panelState, setPanelState] = useState<PanelState>("thinking");
  const [panelNet, setPanelNet] = useState<NetState>("local_only");
  const [panelVisible, setPanelVisible] = useState(true);
  const [liveState, setLiveState] = useState<PanelState>("idle");
  const [liveAnswer, setLiveAnswer] = useState("");
  const [liveTranscript, setLiveTranscript] = useState("");
  const [liveCapture, setLiveCapture] = useState<CaptureInfo | undefined>(undefined);
  const [liveNet, setLiveNet] = useState<NetState>("local_only");
  const [hotkeyLabel, setHotkeyLabel] = useState("⌥⇧V");
  const lastSpokenQuestionRef = useRef("");
  const pttHeldRef = useRef(false);

  const panelView = params.get("view") === "panel";
  const isTauri = !!window.__TAURI_INTERNALS__?.invoke;
  const onboardingView = params.get("view") === "onboarding";
  const overlayView = params.get("view") === "overlay";
  const settingsView = params.get("view") === "settings";
  const timelineView = params.get("view") === "timeline";
  const timelineDemo = timelineView && params.get("demo") === "1";
  const overlaySelection = parseOverlaySelection(params.get("sel"));
  const rawStep = params.get("step");
  const onboardingStep: RitualStep = parseRitualStep(rawStep);
  const rawState = params.get("state");
  const rawNet = params.get("net");
  const panelViewState: PanelState = isPanelState(rawState) ? rawState : "streaming";
  const panelViewNet: NetState = isNetState(rawNet) ? rawNet : "local_only";
  const panelViewCapture: CaptureInfo = (() => {
    const rawScope = params.get("scope");
    const scope = rawScope !== null ? readScope(rawScope) : DEMO_CAPTURE.scope;
    const dim = SCOPE_DIM[scope];
    return {
      ...DEMO_CAPTURE,
      scope,
      width: parseDimension(params.get("w"), dim.width),
      height: parseDimension(params.get("h"), dim.height),
    };
  })();

  useEffect(() => {
    const mq = window.matchMedia("(prefers-reduced-motion: reduce)");
    if (mq.matches) setReducedMotion(true);
    const onChange = (e: MediaQueryListEvent) => setReducedMotion(e.matches);
    mq.addEventListener("change", onChange);
    return () => mq.removeEventListener("change", onChange);
  }, []);

  useEffect(() => {
    const root = document.documentElement;
    if (!panelView && !onboardingView && !overlayView && !settingsView) {
      root.classList.remove("panel-view", "overlay-view");
      return;
    }
    if (panelView) root.classList.add("panel-view");
    if (overlayView) root.classList.add("overlay-view");
    return () => {
      root.classList.remove("panel-view", "overlay-view");
    };
  }, [panelView, onboardingView, overlayView, settingsView]);

  /* Single source of truth for the root `.dark` class. Product views follow
     the persisted `theme`; the dev board (`/`) stays on dawn, as before. */
  useEffect(() => {
    if (!panelView && !onboardingView && !overlayView && !settingsView && !timelineView) {
      document.documentElement.classList.remove("dark");
      return;
    }
    const root = document.documentElement;
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    let detachMq: (() => void) | null = null;
    let active = true;

    const applyTheme = (theme: string) => {
      if (detachMq) {
        detachMq();
        detachMq = null;
      }
      if (theme === "night") root.classList.add("dark");
      else if (theme === "dawn") root.classList.remove("dark");
      else {
        const sync = () => root.classList.toggle("dark", mq.matches);
        sync();
        mq.addEventListener("change", sync);
        detachMq = () => mq.removeEventListener("change", sync);
      }
    };

    const onThemeEvent = (event: Event) => {
      applyTheme(readTheme({ theme: (event as CustomEvent<unknown>).detail }));
    };

    applyTheme("system");
    window.addEventListener(THEME_EVENT, onThemeEvent);
    if (isTauri) {
      invokeTauriAsync("get_settings")?.then(
        (raw) => {
          if (active) applyTheme(readTheme(raw));
        },
        () => undefined,
      );
    }
    return () => {
      active = false;
      window.removeEventListener(THEME_EVENT, onThemeEvent);
      if (detachMq) detachMq();
      root.classList.remove("dark");
    };
  }, [panelView, onboardingView, overlayView, settingsView, timelineView, isTauri]);

  useEffect(() => {
    if (!panelView || !isTauri) return;
    let cancelled = false;
    const unlisteners: TauriUnlisten[] = [];
    const track = (off: TauriUnlisten) => {
      if (cancelled) off();
      else unlisteners.push(off);
    };
    const start = async () => {
      track(
        await listenTauri("panel:transcribing", () => {
          setLiveAnswer("");
          setLiveState("transcribing");
        }),
      );
      track(
        await listenTauri<CapturePayload>("panel:capture", (p) => {
          const scope = readScope(p.scope);
          const dim = SCOPE_DIM[scope];
          setLiveCapture({
            id: p.id,
            time: typeof p.time === "string" && p.time.trim().length > 0 ? p.time : formatHHMM(p.at_ms ?? Date.now()),
            scope,
            width: typeof p.width === "number" && p.width > 0 ? p.width : dim.width,
            height: typeof p.height === "number" && p.height > 0 ? p.height : dim.height,
          });
          setLiveAnswer("");
          setLiveTranscript("");
          lastSpokenQuestionRef.current = "";
          setLiveState("ask");
        }),
      );
      track(
        await listenTauri("panel:listening", () => {
          setLiveAnswer("");
          setLiveTranscript("");
          lastSpokenQuestionRef.current = "";
          setLiveState("listening");
        }),
      );
      track(
        await listenTauri<TranscriptPayload>("panel:transcript", (p) => {
          setLiveTranscript(p.text);
          lastSpokenQuestionRef.current = p.text;
        }),
      );
      track(
        await listenTauri("panel:ask_cancelled", () => {
          setLiveTranscript("");
          lastSpokenQuestionRef.current = "";
          setLiveState("ask");
        }),
      );
      track(
        await listenTauri<TokenPayload>("panel:token", (p) => {
          setLiveAnswer((prev) => prev + p.delta);
          setLiveState("streaming");
        }),
      );
      track(
        await listenTauri<CompletePayload>("panel:complete", (p) => {
          setLiveAnswer(p.answer);
          setLiveState("complete");
        }),
      );
      track(await listenTauri<ErrorPayload>("panel:error", () => setLiveState("error")));
      const pendingSettings = invokeTauriAsync("get_settings");
      pendingSettings?.then(
        (raw) => {
          const record = raw && typeof raw === "object" ? (raw as { offline?: unknown; ptt_hotkey?: unknown }) : {};
          setLiveNet(record.offline ? "offline" : "local_only");
          setHotkeyLabel(formatHotkey(record.ptt_hotkey));
        },
        () => undefined,
      );
      track(
        await listenTauri<{ offline?: boolean }>("settings:changed", (s) =>
          setLiveNet(s?.offline ? "offline" : "local_only"),
        ),
      );
      track(
        await listenTauri<{ mode?: string }>("panel:mode", (p) => {
          setMode(readPanelMode(p?.mode ?? null));
        }),
      );
    };
    void start();
    return () => {
      cancelled = true;
      for (const off of unlisteners) off();
      unlisteners.length = 0;
    };
  }, [panelView, isTauri]);

  const pttStart = () => {
    pttHeldRef.current = true;
    invokeTauriAsync("voice_ask_start");
  };
  const pttStop = () => {
    pttHeldRef.current = false;
    invokeTauriAsync("voice_ask_stop");
  };
  const pttCancel = () => {
    pttHeldRef.current = false;
    invokeTauriAsync("voice_ask_cancel");
  };

  /* Adopt the mode locally first (so the disc swaps instantly), then ask the
     shell to resize the window. The command may not exist yet — swallow. */
  const setPanelMode = (next: PanelMode) => {
    setMode(next);
    invokeTauriAsync("set_panel_mode", { mode: next })?.catch(() => undefined);
  };

  const wizardRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (!isTauri || !onboardingView) return;
    const wizard = wizardRef.current;
    if (!wizard) return;

    let lastSent = -1;
    let timer: number | null = null;

    const send = () => {
      const target = wizard.getBoundingClientRect().height;
      if (lastSent >= 0 && Math.abs(target - lastSent) <= 2) return;
      lastSent = target;
      invokeTauriAsync("resize_onboarding", { height: target, contentHeight: window.innerHeight });
    };

    const observer = new ResizeObserver(() => {
      if (timer !== null) window.clearTimeout(timer);
      timer = window.setTimeout(send, 80);
    });
    observer.observe(wizard);

    return () => {
      observer.disconnect();
      if (timer !== null) window.clearTimeout(timer);
    };
  }, [isTauri, onboardingView]);

  if (overlayView) {
    return <CaptureOverlay selection={overlaySelection} reducedMotion={reducedMotion} />;
  }

  if (panelView) {
    if (mode === "mini") {
      return (
        <MiniMark
          state={isTauri ? liveState : panelViewState}
          onOpen={() => setPanelMode("panel")}
          reducedMotion={reducedMotion}
        />
      );
    }
    return (
      <div
        className={`flex min-h-screen items-start justify-center p-3 text-[14px] leading-[1.45]${reducedMotion ? " reduced-motion rm-halve" : ""}`}
      >
        <ResultPanel
          state={isTauri ? liveState : panelViewState}
          net={isTauri ? liveNet : panelViewNet}
          answer={isTauri ? liveAnswer : DEMO_ANSWER}
          transcript={isTauri ? liveTranscript : DEMO_TRANSCRIPT}
          capture={isTauri ? liveCapture : panelViewCapture}
          hotkeyLabel={hotkeyLabel}
          reducedMotion={reducedMotion}
          onCollapse={() => setPanelMode("mini")}
          onPttStart={isTauri ? pttStart : undefined}
          onPttStop={isTauri ? pttStop : undefined}
          onPttCancel={isTauri ? pttCancel : undefined}
          onEscape={
            isTauri
              ? () => {
                  if (liveState === "listening" || liveState === "transcribing" || pttHeldRef.current) {
                    pttCancel();
                    return true;
                  }
                  return false;
                }
              : undefined
          }
          onRetry={
            isTauri
              ? () => {
                  setLiveState("thinking");
                  invokeTauriAsync("session_ask", {
                    transcript: lastSpokenQuestionRef.current || "what is this?",
                    captureIds: liveCapture ? [liveCapture.id] : [],
                  })?.catch(() => undefined);
                }
              : undefined
          }
          onDismiss={
            isTauri
              ? () => {
                  invokeTauri("hide_panel");
                  if (liveState === "thinking" || liveState === "streaming") invokeTauri("session_abort");
                }
              : () => invokeTauri("hide_panel")
          }
        />
      </div>
    );
  }

  if (onboardingView) {
    return (
      <div className={`min-h-screen bg-background${reducedMotion ? " reduced-motion rm-halve" : ""}`}>
        <div ref={wizardRef} className="w-full">
          <OnboardingRitual initialStep={onboardingStep} reducedMotion={reducedMotion} />
        </div>
      </div>
    );
  }

  if (settingsView) {
    return (
      <div className="min-h-screen bg-background">
        <SettingsWindow reducedMotion={reducedMotion} />
      </div>
    );
  }

  if (timelineView) {
    return (
      <div className="min-h-screen bg-background">
        <TimelineView demo={timelineDemo} reducedMotion={reducedMotion} />
      </div>
    );
  }

  return (
    <div className={`min-h-screen${reducedMotion ? " reduced-motion rm-halve" : ""}`}>
      <div className="mx-auto flex max-w-[1120px] flex-col gap-12 px-6 pb-20 pt-10">
        <header className="flex flex-col gap-6">
          <div className="flex flex-wrap items-center justify-between gap-4">
            <div className="flex items-center gap-3">
              <TrayMark svg={TRAY_STATES[0].svg} size={28} />
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
                <TrayMark svg={TRAY_STATES[1].svg} size={18} />
                <span className="font-companion text-[13px] font-semibold">listening</span>
              </span>
            </MenuBar>
            <MenuBar theme="dark">
              <span className="menubar-chip menubar-chip--dark">
                <TrayMark svg={TRAY_STATES[1].svg} size={18} />
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
            <span className="font-semibold">Verdict — kept as history.</span> The v4 design replaced the
            resting mark with the capture-frame glyph (§ 2); B · Dawn Dot lives on as the panel avatar's
            tray-scale ancestor, and A / C stay on the board as idle-mark history.
          </p>
        </Section>

        <Section
          eyebrow="§ 2 · tray glyph"
          title="One frame, three shapes"
          lede="The v4 design carries tray state by shape alone: ready, listening, captures paused. Offline and every other state are confirmed in words (tray menu + tooltip); colour stays in the panel, not the bar. The reduced-motion switch still freezes board loops, per the motion law."
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
                  transcript={DEMO_TRANSCRIPT}
                  reducedMotion={reducedMotion}
                  onSaveMemory={() => undefined}
                  onRetry={() => setPanelState("thinking")}
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
                  transcript={DEMO_TRANSCRIPT}
                  reducedMotion={reducedMotion}
                  onSaveMemory={() => undefined}
                  onRetry={() => setPanelState("thinking")}
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
                    transcript={DEMO_TRANSCRIPT}
                    reducedMotion={reducedMotion}
                    onSaveMemory={() => undefined}
                    onRetry={() => undefined}
                  />
                </PanelStage>,
                <PanelStage key={`night-${state}`} label={`${state} · ${net} · night`} dark>
                  <ResultPanel
                    state={state}
                    net={net}
                    answer={DEMO_ANSWER}
                    capture={DEMO_CAPTURE}
                    transcript={DEMO_TRANSCRIPT}
                    reducedMotion={reducedMotion}
                    onSaveMemory={() => undefined}
                    onRetry={() => undefined}
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
