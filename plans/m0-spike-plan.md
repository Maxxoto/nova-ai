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
| S1 PTT key-up | **PASS** (macOS 26.0.1; Windows pending) | key-down+key-up delivered, latency 332–405 µs p50 (target <50 ms) | raw route = listen-only `CGEventTap` (Session, default-mode runloop source); toggle fallback not needed | 2026-09-19 |
| S2 idle RAM | **PASS** (macOS 26.0.1, M-series, release build) | idle 147.6 MB (shell 96.1 + sidecar 51.5); panel-open +3.6 MB; base q5_1 per-call: load 29 ms, peak +219 MB released to ~37 MB; small q5_1: load 58 ms, peak +458 MB | default STT = `base q5_1`; keep per-call load/drop (no resident cache — 29 ms penalty imperceptible); `small` stays opt-in (decode ~1.4 s/s audio, 458 MB transient) | 2026-09-20 |
| S3 Win overlay | | | | |
| S4 panel focus | **PASS** (macOS 26.0.1, release build, automated) | focus retained (TextEdit frontmost while panel visible); typing unbroken ("abc"+panel+"def" → "abcdef"); Esc pass-through hides panel; accessory+nonactivating+ignoresCycle keeps it out of cmd-tab | ship non-activating panel (style mask bit 7) + listen-only Esc tap; drop `set_focus()` from `panel::show` | 2026-09-20 |
| S5 STT latency/zh | | | | |

### S2 findings (macOS, probe: `shell/src-tauri/examples/ram_probe.rs` + release app)

Method: `ps -o rss=` on the release binary (tray resident, panel closed,
45 s settle) summed with the spawned python sidecar; probe loads the model
with the exact `voice.rs` params (`use_gpu = true`, Greedy best-of-1) and
samples RSS around load / 1 s & 5 s synthetic passes / drop. whisper.cpp
prints a 96 MB CPU compute buffer at state init; Metal-resident weights are
partially invisible to RSS (undercount noted).

| Measurement | base q5_1 (59.7 MB) | small q5_1 (190 MB) |
|---|---|---|
| load time (lazy) | 29 ms | 58 ms |
| RSS after load | 69.6 MB | 194.1 MB |
| peak during decode | 219 MB | 458 MB |
| after drop | 37 MB | 54 MB |
| decode, 1 s audio | ~460 ms | ~1.4 s |

App envelope (release, no model resident — `voice.rs` loads per utterance):

| State | shell | sidecar | sum |
|---|---|---|---|
| idle, panel closed | 96.1 MB | 51.5 MB | **147.6 MB** |
| idle, panel open | 99.7 MB | 51.5 MB | 151.2 MB |

- Idle 147.6 MB ≤ 150 MB target → PASS; default STT = `base q5_1`.
- Panel-open 151.2 MB is a transient state by design (AC-01); accepted.
- The spike's "model loaded while idle" premise is obsolete: per-call
  load/drop means idle never holds weights, and the 29 ms lazy-load
  penalty removes any need for a resident cache. RFC-0001 risk #4
  (memory floor) closes for macOS.
- Decode-speed preview for S5: base ≈ 0.46 RTF vs small ≈ 1.4 RTF on
  synthetic noise — small likely too slow for PTT UX; confirm with real
  speech in S5.
- Parakeet not measurable this pass (download incomplete); revisit after
  the S5 corpus exists.

### S4 findings (macOS, automated harness: release app + TextEdit + System Events)

`panel::show` no longer calls `set_focus()`. At setup the panel window gets
`NSWindowStyleMaskNonactivatingPanel` plus
`canJoinAllSpaces | ignoresCycle | fullScreenAuxiliary`; a second listen-only
CGEventTap (same S1 pattern) hides the panel on bare Esc (keycode 53) while
`PANEL_VISIBLE` is set, never consuming the key.

| Check | Result |
|---|---|
| frontmost app while panel visible | TextEdit (never Ruoxi) |
| typing during panel-open | "abc" before + "def" after → "abcdef", no drops |
| Esc (bare) | panel hides; underlying app stays frontmost |
| Esc consumption | pass-through (ListenOnly tap) — editor's own Esc unaffected |
| cmd-tab / window cycle | excluded by construction (accessory policy + nonactivating + ignoresCycle); manual spot-check pending |
| fullscreen spaces | `fullScreenAuxiliary` allows overlay; not yet exercised |

- The webview's own Esc handler remains as a fallback for when the panel
  does hold focus (e.g., after a click inside it).
- Clicks/scrolling inside the non-activating panel work without activating
  the app (standard NSPanel behavior); text-selection inside the webview
  while the owner app is inactive is the one path to watch post-M0.

### S1 findings (macOS, probe: `spikes/s1-ptt`)

1. **Route validated**: listen-only `CGEventTap` at `kCGSessionEventTap`, KeyDown+KeyUp,
   built via core-graphics 0.25 (`CGEventTap::new(...).expect`, callback returns `Keep`).
   Latency (event timestamp → handler) 332–405 µs, real keys, foreground contexts.
2. **Runloop gotcha**: the tap's runloop source must attach to `kCFRunLoopDefaultMode` —
   `kCFRunLoopCommonModes` is an empty set on a bare thread runloop and the tap silently
   never fires (runloop exits immediately).
3. **Permission behavior (feeds §4.5 ritual)**:
   - No OS prompt appears on denial — tap creation just fails (`CGEventTap::new` → `Err`).
     The app must self-check (`AXIsProcessTrusted`) and guide the user to Settings.
   - TCC attributes trust to the *responsible process*: run from a granted host
     (Terminal.app), all spawned binaries inherit trust. Granting a debug binary directly
     works but **invalidates on every rebuild** (ad-hoc signature changes) — dev builds
     need a stable self-signed identity (W6 packaging) or the host-app grant pattern.
   - `launchd`-spawned processes are their own responsible process (useful for testing).
4. **Avoid synthetic-event testing**: posting CGEvents back into HID from the same process
   is flaky (intermittent hangs in WindowServer round-trips). Real-key validation only.
5. Production PTT keys should use an **active filter tap** (return `Drop` for the PTT key)
   so the key does not leak to the focused app; listen-only was sufficient for the spike.
6. **GUI apps are their own responsible process**: once the tray app registers as a
   UIElement, it does NOT inherit the launching terminal's Accessibility grant (CLI
   tools do). The app binary needs its own grant — confirmed on nova-shell
   (2026-09-19: terminal-granted context still reported untrusted; direct binary
   grant fixed it). The §4.5 onboarding ritual must grant Ruòxī itself.

