# M0 Build Plan — "It Lives" (v0.1)

- **Status:** Draft — spike-gated (see gates below)
- **Owner:** Dani
- **References:** [PRD §11](../docs/ruoxi_prd.md) (v0.1 scope) · [RFC-0002](../docs/rfc/RFC-0002-platform-shell.md) · [RFC-0003](../docs/rfc/RFC-0003-screen-capture.md) · [Spike plan](m0-spike-plan.md)
- **Definition of done (product):** the app lives in the tray; hotkeys work; all three capture scopes capture exactly; the panel shows results and `Esc` dismisses; the Python brain sidecar runs supervised behind IPC v1. No STT, no LLM, no memory yet — M0 is the body, not the mind.

---

## 0. Repo layout (ratify before W1 starts)

Proposal — monorepo, existing brain untouched:

```
nova-ai/
  shell/                 # NEW — Tauri 2 app (Rust core + webview UI)
    src-tauri/           #   Rust core: tray, hotkeys, capture, sidecar supervisor
    src/                 #   WebView UI: panel, overlay chrome, onboarding, settings
  src/app/               # EXISTING — Python brain (agent loop, tools, memory)
  docs/  plans/  tests/  # unchanged
```

Rationale: one repo, one README story, PRs can touch both sides of the IPC
contract atomically. Alternative rejected: separate `nova-shell` repo (contract
drift, two CI pipelines, MIT fork story gets muddy).

## 1. Workstreams

### W1 — Tauri 2 scaffold + tray + lifecycle  *(no spike dependency — start day 1)*
- `shell/` project init (Tauri 2), accessory activation policy (no Dock icon), tray icon + menu (quit, pause captures, settings, about), single-instance lock, launch-at-login opt-in.
- Settings persistence v0 (JSON in app data dir).
- **Done when:** AC-01 holds on macOS (tray present, no Dock window, quit works).

### W2 — Global hotkeys + PTT  *(GATE: S1 verdict)*
- Region/window/fullscreen hotkeys via platform shortcut APIs; PTT via the S1-validated raw-event route (or toggle fallback if S1 fails — decided by spike, not here).
- Hotkey config + conflict detection (in-use combos warn).
- **Done when:** all four triggers fire in any app; PTT key-down/key-up logged with S1-verified fidelity (mic itself is M1).

### W3 — Overlay + three capture scopes + store v0  *(GATE: S3 for Windows; macOS start anytime)*
- Transparent overlay drag-select (post-S3 fallback: solid background); window scope with frontmost-excluding-self; fullscreen = display under cursor; HiDPI scale-factor math per RFC-0003 §4.2.
- Capture store v0: date-sharded files + SQLite index with the RFC-0003 §4.3 schema (sha256 dedupe included); retention = manual-delete only in M0.
- **Done when:** AC-03/04/05 hold on macOS (Windows after S3): dragged region exact, frontmost window not desktop, native-res display under cursor.

### W4 — Result panel  *(GATE: S4 informs focus policy)*
- Frameless, non-activating panel near capture point; `Esc` dismiss with no side effects; minimal content surface (M0: placeholder answer area + capture thumbnail).
- Onboarding permission ritual v0 (RFC-0002 §4.5): Screen Recording, Microphone, Accessibility prompts with why-lines — AC-12.
- **Done when:** AC-06 + AC-12 hold; underlying app keeps focus when panel shows (S4-verified or documented fallback).

### W5 — Brain sidecar + IPC v1  *(no spike dependency — start day 1, parallel with W1)*
- Python: stdio JSON-RPC server loop around the existing brain entry; implement `ping`, health, graceful shutdown; `session.ask`/`session.abort`/`agent.token` envelopes stubbed to a canned response (real loop lands M1).
- Rust: sidecar supervisor (spawn, health-check, backoff restart, shutdown) per RFC-0002 §4.7.
- **Done when:** killing the brain process → supervisor restarts it → tray shows status; `session.ask` returns the stub through the panel end-to-end.

### W6 — Packaging skeleton  *(after W1–W4 integrate)*
- Unsigned `.dmg` + Windows installer via Tauri bundling; bundle Python runtime + brain code; no model artifacts yet (M1 concern).
- **Done when:** a clean machine installs and runs M0 without dev tooling.

## 2. Sequencing

```
Day 1..        W1 ──┐                W5 ────────────┐   (parallel, spike-free)
After S1/S4 →  W2, W4 (macOS)                       │
After S3  →    W3 (macOS start; Windows gate) ──────┤
Last:          W6 (integration) ←───────────────────┘
```

Spike verdicts only *unlock* W2/W3-Win/W4 details; W1, W5, and W3-macOS can
begin during spike week itself.

## 3. Acceptance checklist (M0 exit)

| PRD AC | Check |
|---|---|
| AC-01 | Tray resident, no Dock/taskbar clutter |
| AC-03 | Region capture = exactly the dragged rect |
| AC-04 | Window capture = frontmost non-self window |
| AC-05 | Fullscreen = display under cursor, native res |
| AC-06 | `Esc` dismisses panel, no side effects |
| AC-12 | Permission prompts preceded by why-lines |

Plus: sidecar supervision demonstrable (kill/restart), store writes indexed
and rebuildable, `git tag v0.1` on a green checklist.

## 4. Explicitly NOT in M0

STT (M1) · LLM answers (M1) · memory grounding (M2) · Save/TTS/brief (M3) ·
timeline search UI (M2) · auto-capture F-13 · signed/notarized builds (M3
hardening) · Windows feature parity beyond what S3 validates.

## 5. Risks carried into build

| Risk | Owner | Mitigation in plan |
|---|---|---|
| S1 fails → toggle PTT | W2 | Fallback pre-agreed in spike plan; hotkey layer abstracted from trigger mode |
| RAM envelope unknown until S2 | W1/W5 | Measurement harness included in W1 (tray-only baseline) so S2 has real numbers |
| IPC contract churn M1→M2 | W5 | Envelopes versioned (`proto_ver`) from day one |
| Two-platform drift | W3 | Per-AC test matrix per platform in the checklist |
