# RFC-0002 — Platform Shell & Lifecycle (Tauri 2)

- **Status:** Draft for review
- **Author:** Dani
- **Parent:** [RFC-0001](RFC-0001-desktop-companion.md) (Desktop Companion)
- **Companion:** [Business PRD](../ruoxi_prd.md)
- **Covers (PRD):** F-01 (tray resident), F-06 (floating panel), UI side of F-09
- **ACs:** AC-01, AC-06, AC-12 · **Milestone:** M0 ("It lives")

---

## 1. Summary

The shell is the Tauri 2 application that hosts everything the user touches:
tray icon, global hotkeys (including push-to-talk with key-down/key-up), the
transparent capture overlay, the floating result panel, the onboarding
permission ritual, and the supervision of the Python brain sidecar. It owns the
**< 150 MB idle RAM budget** (RFC-0001 risk #4) and the process lifecycle.

## 2. Motivation

Every user-visible PRD promise ("always one keystroke away; never in the way")
lands in this component. The shell is also where four of the five open risks
from RFC-0001's amendment live: PTT key-up events (risk #1), Windows overlay
quirks (risk #2), panel vibrancy (risk #3), and fullscreen/game hotkey edge
cases (risk #5). Getting the shell seams right is the prerequisite for every
other component RFC.

## 3. Goals / Non-Goals

**Goals**
- Tray/menu-bar resident app with no Dock/taskbar clutter (AC-01).
- Global hotkeys incl. PTT with **key-down AND key-up** semantics.
- Transparent region-select overlay + frameless floating result panel.
- Guided onboarding permission ritual with a "why" line per permission (AC-12).
- Supervise the Python brain sidecar: start, health-check, restart, shutdown.
- Stay within the idle RAM envelope (< 150 MB, whisper loaded — shared with RFC-0004).

**Non-Goals**
- Any business logic (agent loop, memory, tools — RFC-0006/0007/0008).
- Capture mechanics themselves (RFC-0003) — the shell only hosts the overlay UI.
- OS-level machine control (v1, native adapter).

## 4. Detailed Design

### 4.1 Process architecture

```
┌────────────────────────── Tauri 2 app ──────────────────────────┐
│ Rust core                                                       │
│  • tray + global hotkey listener (raw event routes)             │
│  • capture overlay window mgmt · panel window mgmt              │
│  • PTT audio pipeline + whisper-rs (RFC-0004) · TTS (RFC-0005)  │
│  • sidecar supervisor (Python brain)                            │
│ WebView UI (system WebView: WKWebView / WebView2)               │
│  • panel content · onboarding · settings · timeline (RFC-0003)  │
└─────────────────────────────────────────────────────────────────┘
              │ IPC (§4.6)                │ native seams
              ▼                           ▼
        Python brain sidecar        macOS / Windows APIs
```

The Rust core is the **only** place platform APIs are touched (RFC-0001's
adapter seam). The WebView layer stays presentational.

### 4.2 Tray & lifecycle

- Menu-bar/tray icon from launch; activation policy set to accessory (no Dock
  icon, no taskbar button) → AC-01.
- Launch at login: opt-in during onboarding, via the registered platform
  mechanism (login-items / Run-key equivalent).
- Quit, pause captures, open settings, open timeline from the tray menu.
- Single-instance lock; second launch focuses the existing instance.

### 4.3 Global hotkeys & PTT (risk #1 — required spike)

| Hotkey (default, configurable) | Semantics |
|---|---|
| Hold-to-talk | key-down = mic open; key-up = mic close → STT (RFC-0004) |
| Region capture | press = show overlay, drag = select (RFC-0003) |
| Window capture | press = capture frontmost window |
| Fullscreen capture | press = capture display under cursor |
| `Esc` | dismiss panel, abort agent loop, stop TTS (AC-06, AC-11) |

Shortcuts registered via the platform global-shortcut mechanisms give
key-pressed events only; **key-up is not delivered by the standard shortcut
APIs on either OS**. Design: a small raw-event route in the Rust core —

- macOS: a key-monitor/event-tap style mechanism at the accessibility layer.
- Windows: a low-level keyboard hook style mechanism.

Both require Accessibility/input-monitoring permissions, which folds into the
onboarding ritual (§4.5). **Spike required before M0 sign-off**: prove key-up
delivery in fullscreen apps and games (risk #5), and define fallback (e.g.,
toggle-mode PTT) if a context blocks raw events.
*S1 verdict (macOS, 2026-09-19): PASS — listen-only session CGEventTap,
down+up delivered at 332–405 µs; findings in plans/m0-spike-plan.md.*

### 4.4 Windows: overlay & panel

- **Capture overlay:** a fullscreen transparent, always-on-top window used as
  the region-select surface. Windows risks: DWM composition/acrylic artifacts
  on transparent topmost windows, click-through behavior, multi-monitor
  geometry. Spike: validate on Win10 + Win11 before M0.
- **Result panel:** frameless, non-activating (must not steal focus from the
  user's app), positioned near the capture or cursor with screen-edge
  clamping. `Esc` dismisses with no side effects (AC-06).
- **Vibrancy/blur** (risk #3): WKWebView (macOS) and WebView2 (Windows) both
  have platform-specific backdrop behaviors; treat as polish item in M0,
  non-blocking fallback = semi-opaque solid background.

### 4.5 Onboarding permission ritual (J8, AC-12)

Ordered ritual; each step shows **one plain-language "why" line** before the
OS prompt:

1. Screen Recording → "so Ruòxī can see exactly what you point at."
2. Microphone → "so you can ask by voice; audio is deleted within a minute."
3. Accessibility / Input Monitoring → "so the talk button works while you hold
   it, in any app."
4. Optional: launch at login.

Then a guided **first demo capture** (box something, ask anything) to reach the
"it works!" moment. Permission-denied paths degrade gracefully with a
re-grant path from settings.

### 4.6 IPC: shell ↔ Python brain

| Option | Pros | Cons |
|---|---|---|
| **stdio JSON-RPC (recommended)** | no sockets, no ports, clean child supervision, packrizable | one request stream; streaming events need careful framing |
| localhost HTTP + SSE | familiar, browser-friendly dev tools | port allocation, firewall prompts on Windows, socket exposure |

**Decision: stdio JSON-RPC** (newline-delimited) with typed envelopes;
streaming (tokens, state changes) as JSON-RPC notifications. Rationale: no
local attack surface from sockets, simplest lifecycle binding to the child
process. Message categories:

```
req  session.ask        {transcript, capture_ids[], mode}
nfy  agent.token        {delta}           → panel streaming (RFC-0007)
nfy  agent.tool_step    {n_of_3, tool}    → progress affordance
req  session.abort      {}                → Esc (RFC-0007 §4e)
req  capture.take       {scope}           → routed to Rust capture (RFC-0003)
nfy  stt.final          {text, lang}      → key-up pipeline (RFC-0004)
req  memory.search/save {...}             → RFC-0006/0008
```

### 4.7 Sidecar supervision

- Supervisor in Rust core: spawn Python sidecar at app start (bundled
  runtime), health-check via periodic `ping`, exponential-backoff restart on
  crash (max N, then degraded-mode tray notice), graceful SIGTERM-style
  shutdown on app quit.
- Cold-start budget: brain must answer first `ping` within ~2s of spawn
  (target, validate in M0).

### 4.8 RAM budget strategy (risk #4)

Envelope < 150 MB idle (panel closed, whisper loaded):

| Resident | Target |
|---|---|
| Tauri shell + WebView (panel closed) | ≤ 60 MB |
| whisper model (small, quantized — RFC-0004) | ≤ 70 MB |
| Python brain sidecar (loaded, idle) | ≤ 60 MB (shared/overlapping accounting TBD) |

These overlap in the total: the binding constraint is measured **RSS sum**.
Policy: WebView preloaded but panel window not created; Python imports lazily;
whisper lazy-or-preload decided by RFC-0004 spike. Enforced by a CI-style
measurement script during milestones. Numbers are **targets to validate**, not
measurements.

### 4.9 Packaging & updates

- macOS: signed + notarized `.dmg`; Windows: signed `.msi`/installer.
- Bundles: Python runtime + app code, whisper model artifact (or first-run
  download — decision owned by RFC-0004), TTS assets (RFC-0005).
- Updates: manual-check in v0 (no silent network — RFC-0009); auto-update
  deferred.
- Installer size impact of bundled model: tens of MB scale, quantified in
  RFC-0004's model matrix.

## 5. Interfaces & Data Structures

- `HotkeyConfig {ptt, region, window, fullscreen}` — persisted settings.
- `PanelState {hidden, streaming, complete, error}` — drives WebView UI.
- IPC envelopes as in §4.6 (schema owned here; versioned `proto_ver` field).
- `OnboardingState` — machine-readable checklist for resume-after-restart.

## 6. Risks & Failure Modes

| Risk | Impact | Mitigation |
|---|---|---|
| No key-up for global shortcuts (risk #1) | PTT unusable | Raw-event spike; toggle-mode fallback |
| Windows transparent topmost quirks (risk #2) | Broken region select | DWM/acrylic spike; solid-color fallback |
| Panel vibrancy inconsistent (risk #3) | Visual polish only | Non-blocking; fallback background |
| RAM budget exceeded (risk #4) | "Always-resident" promise breaks | Measurement harness; lazy loading; downgrade whisper model |
| Fullscreen/game hotkey loss (risk #5) | Feature gap in games | Spike; document known-bad contexts |
| Sidecar crash loop | Dead assistant | Backoff + degraded mode + user-visible status |
| Focus stealing by panel | Breaks flow (core promise) | Non-activating window policy on both OSes; test in AC-06 suite |

## 7. Open Questions

1. macOS event-tap vs CGEvent-level route for key-up — which is reliable under
   modern macOS permission hardening? (spike)
2. Windows low-level hook: latency and AV-software interaction? (spike)
3. Non-activating panel on WebView2 — confirmed support for
   `WS_EX_NOACTIVATE`-style behavior with Tauri windows? (spike)
4. Exact RSS accounting method for the shared envelope (sum vs PSS)?
5. Toggle-PTT fallback: user-visible config or automatic on failure?

## 8. Dependencies

- **RFC-0003** — overlay drives region capture; capture windows hosted here.
- **RFC-0004 / RFC-0005** — PTT pipeline and TTS live in the Rust core this
  RFC defines; RAM allocations agreed jointly.
- **RFC-0007** — IPC contract consumed by the agent brain; `Esc` abort wiring.
- **RFC-0009** — no-silent-network rule applies to update checks; offline
  indicator shown in tray/panel chrome.

## 9. Acceptance Mapping

| AC | Mechanism |
|---|---|
| AC-01 | Accessory activation policy, tray icon, single instance (§4.2) |
| AC-06 | `Esc` handler: hide panel + `session.abort` + TTS stop (§4.4, §4.6) |
| AC-12 | Onboarding ritual with why-lines before each prompt (§4.5) |

## 10. Milestone Alignment

- **M0:** tray, hotkeys (post-spike), overlay + panel (fallback visuals),
  IPC v1, sidecar supervision, packaging skeleton.
- **M1+:** consumes RFC-0004/0007 streams; polish (vibrancy) after M0.
