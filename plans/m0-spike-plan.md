# M0 Spike Plan — De-Risking the Tauri 2 Shell Before Build

- **Status:** Ready to run
- **Owner:** Dani
- **References:** [RFC-0001](../docs/rfc/RFC-0001-desktop-companion.md) amendment risks #1–#5 · [RFC-0002](../docs/rfc/RFC-0002-platform-shell.md) §7 · [RFC-0004](../docs/rfc/RFC-0004-voice-input-stt.md) §7
- **Purpose:** validate the five assumptions that could change the shell design **before** M0 implementation starts. Each spike is a small throwaway probe (hours, not days), with explicit pass/fail and a pre-agreed fallback.

---

## What a spike is (and is not)

A **spike** is a minimal throwaway experiment that answers one risky technical
question with a measurement, not a prototype you keep. Code quality doesn't
matter; the recorded result does. When a spike passes, its *finding* feeds the
RFC; the code is discarded or reduced to a reference snippet.

**Spike week rule:** one question per spike, one page of results per spike,
timeboxed. If a spike blows its box, we take the fallback and move on.

---

## Spike 1 — PTT key-up via raw event routes (architecture-breaking, run FIRST)

- **Question:** can we receive global key-down AND key-up for a chosen hotkey
  in any app, including fullscreen, via a raw-event mechanism in Rust?
  (macOS event-tap class route; Windows low-level keyboard hook class route.)
- **RFC:** risk #1 (RFC-0001), RFC-0002 §4.3.
- **Setup:** minimal Tauri 2 app + Rust crate registering the raw route;
  Accessibility/Input-Monitoring permission granted.
- **Procedure:** log both events with timestamps while: normal app focus,
  fullscreen video, one game (if available), terminal, and another
  virtual desktop/space.
- **Record:** event latency key-down→our handler (ms), missed events, whether
  key-up arrives in fullscreen contexts, permission-prompt behavior per OS.
- **PASS:** key-down + key-up delivered in ≥ all non-game contexts, latency
  imperceptible (< 50 ms, target).
- **FAIL → fallback:** toggle-mode PTT (press to start, press to stop) —
  documented as the shipped behavior; PRD F-02 unaffected in substance.

## Spike 2 — Idle RAM: shell + WebView + whisper model

- **Question:** does Tauri 2 (tray-only, panel closed) + system WebView +
  a loaded whisper quantized model fit the < 150 MB idle envelope?
  (RFC-0002 §4.8; RFC-0001 risk #4.)
- **Setup:** spike 1's app + `whisper-rs` with `base` q5, then `small` q5;
  tray resident; panel window not created.
- **Procedure:** measure RSS sum (shell process + WebView child + any audio
  process) after 5 min idle, model loaded; repeat panel-opened.
- **Record:** RSS table {model × panel-closed/opened}, load time on first
  use (lazy-load penalty), measurement method used.
- **PASS:** `base` ≤ 150 MB with panel closed → default = `base`;
  bonus PASS: `small` ≤ 150 MB → default = `small`.
- **FAIL → fallback:** `base` breaks envelope → tiny/model-downgrade path +
  revisit envelope in RFC-0002 (product call: accept higher floor or lighter
  model).

## Spike 3 — Windows transparent topmost overlay

- **Question:** does a fullscreen transparent, always-on-top, non-activating
  window render correctly for region-select drag on Win10 + Win11 (DWM
  composition, multi-monitor)? (Risk #2; RFC-0002 §4.4, RFC-0003 §4.2.)
- **Setup:** spike 1's app on Windows; overlay window with border
  highlight + crosshair cursor; two monitors with different scale factors.
- **Procedure:** draw drag rectangle; screenshot result; verify rectangle
  maps to the exact screen region on each monitor/scale combo.
- **Record:** visual artifacts (or none), click-through behavior, geometry
  correctness matrix (monitor × scale), any DWM fallback used.
- **PASS:** exact geometry on all combos, no blocking artifacts.
- **FAIL → fallback:** semi-opaque solid overlay background (cosmetic), or
  per-monitor native capture UI if geometry is unfixable (escalate to
  RFC-0003 revision).

## Spike 4 — Non-activating panel + Esc semantics

- **Question:** can the result panel appear without stealing focus from the
  user's app, receive `Esc` reliably, and not appear in Alt-Tab/cmd-tab on
  both OSes? (RFC-0002 §4.4; AC-01/AC-06 "never in the way".)
- **Setup:** spike 1's app; frameless panel shown near cursor over a focused
  text editor.
- **Procedure:** trigger panel while typing in the editor; keep typing
  (input must not drop); press Esc (panel hides); check app switcher lists.
- **Record:** focus retained (y/n), caret behavior in the underlying app,
  Esc delivery, switcher visibility, macOS vs Windows differences.
- **PASS:** focus retained + Esc works + not in switcher on both OSes.
- **FAIL → fallback:** brief focus grant then return (measure annoyance);
  document as known issue; vibrancy/focus polish deferred post-M0.

## Spike 5 — whisper latency + Chinese quality (decides STT default)

- **Question:** do `base`/`small` quantized meet ≤ 800 ms p50 for ≤ 10 s
  utterances, and is zh accuracy acceptable on real study speech?
  (RFC-0004 §4.5/4.6, §7 Q1/Q2.)
- **Setup:** spike 2's app; 20 recorded utterances (10 en, 10 zh, real
  study-style sentences, 3–10 s each) on the reference machines.
- **Procedure:** transcribe set per model; measure wall time; eyeball +
  score zh/en transcripts (WER tool optional; manual suffices for 20 clips).
- **Record:** latency p50/p95 per model per machine, qualitative accuracy
  notes, chosen default.
- **PASS:** any model meeting both latency + zh quality → becomes default;
  result written back into RFC-0004 §4.2 (replacing "approx").
- **FAIL → fallback:** nearest model wins on the more important axis
  (latency for M1, zh quality is the product ask — likely quality), or
  zh-specialized variant spike.

---

## Running order & budget

| Day | Spikes | Output
|---|---|---|
| 1 | S1 (macOS) + S2 start | PTT verdict or fallback decision
| 2 | S2 finish + S5 | STT default + RAM verdict → RFC-0004/0002 updated
| 3 | S3 (Windows) + S4 | Windows overlay + panel verdicts
| **Total** | **~3 days, one person** | all five RFC open risks closed

## Exit criteria (spike week is done when)

1. All five verdicts recorded in this file (PASS/FAIL + chosen fallback).
2. RFC-0002 §7 and RFC-0004 §7 open-question lists updated to decisions.
3. RAM + latency tables in RFC-0002 §4.8 / RFC-0004 §4.6 hold measured
   numbers instead of targets.
4. Then: M0 execution plan is drafted (`plans/m0-build-plan.md`) with
   nothing unvalidated left in it.

## Results log (fill in during spike week)

| Spike | Verdict | Key numbers | Decision | Date |
|---|---|---|---|---|
| S1 PTT key-up | | | | |
| S2 idle RAM | | | | |
| S3 Win overlay | | | | |
| S4 panel focus | | | | |
| S5 STT latency/zh | | | | |
