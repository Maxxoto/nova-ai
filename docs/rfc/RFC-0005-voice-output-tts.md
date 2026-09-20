# RFC-0005 — Voice Output (Local TTS)

- **Status:** Draft for review
- **Author:** Dani
- **Parent:** [RFC-0001](RFC-0001-desktop-companion.md) (Desktop Companion)
- **Companion:** [Business PRD](../ruoxi_prd.md)
- **Covers (PRD):** F-11 (TTS read-back, toggle — SHOULD priority; journey J2)
- **ACs:** none exclusively (supports AC-06 spirit: Esc stops speech) · **Milestone:** M3 ("It feels")

---

## 1. Summary

Local text-to-speech read-back of answers, off-cloud by construction:
cross-platform neural voices as the primary path (Piper-class engine in the
Rust core) with per-platform system voices as the always-available fallback.
Sentence-chunked streaming so speech begins while the answer is still
generating; interruptible by Esc, new answers, and PTT.

## 2. Motivation

F-11 is the "companion feel" layer: hear explanations while you read (J2).
It arrives in M3, after the core loop is trustworthy. TTS must be fully local
(offline kill-switch, AC-09, leaves zero room for cloud TTS) and cheap
enough to fit the shell's RAM envelope.

## 3. Goals / Non-Goals

**Goals**
- Fully local synthesis; zero network by construction.
- Bilingual zh/en readback with automatic per-sentence voice/script switching.
- Starts on first sentence while generation continues (RFC-0007 streaming).
- Interruption semantics: Esc, new answer, PTT key-down all stop speech.
- Graceful degradation: engine failure → silent text-only + indicator.

**Non-Goals**
- Voice cloning, custom voice training.
- Reading anything other than panel answers (no screen narration).
- Audio persistence (nothing is ever recorded or stored).

## 4. Detailed Design

### 4.1 Engine options

| Option | Quality | zh+en | First-audio latency | Footprint | Packaging |
|---|---|---|---|---|---|
| macOS system voices (AVSpeechSynthesizer-class) | decent, robotic at times | ok (per-voice varies) | very low | ~0 (system) | free |
| Windows system voices (SAPI/WinRT-class) | decent | ok (zh voice install may be needed) | very low | ~0 (system) | free |
| Neural, cross-platform (Piper-class, in Rust core) | good | good with per-language voices | low–medium | ~50 MB/voice class | bundle or download voices |

**Decision: Piper-class neural engine as primary, system voices as fallback
adapter.** Rationale: consistent quality/behavior across the two platforms
(one code path to test), zh quality controllable by explicit voice choice;
system-voice fallback keeps F-11 functional if neural init fails or voices
are missing (§4.7). The TTS layer sits behind a small adapter trait so the
fallback is a runtime swap, not a code fork.

Placement: Rust core (same rationale as RFC-0004 §4.1 — audio I/O and native
APIs live at the seam; the brain sends only text).

### 4.2 Text extraction from answers

- Input: the answer's markdown stream (RFC-0007).
- Extraction rules (v0): strip code blocks entirely (reading code aloud is
  noise; panel still shows them — speech says "[code omitted]"), strip link
  URLs (keep anchor text), keep headings/list items as plain sentences with
  brief pauses.
- Bilingual text passes through unchanged; sentence segmentation handles
  both `。！？` and `.!?`.

### 4.3 Sentence-chunked streaming

```
LLM token stream ──► sentence splitter ──► queue ──► synthesizer ──► audio out
                            │
                            └── panel shows full text independently
```

- On first complete sentence: begin synthesis + playback. Target
  time-to-first-audio ≤ 500 ms after the sentence is available (to validate
  on M3 hardware).
- Queue is bounded (e.g., 20 sentences); overflow drops to text-only for the
  remainder of that answer (protects RAM).

### 4.4 Queueing & interruption semantics

| Event | Behavior |
|---|---|
| Speech active + new answer arrives | Flush queue, cancel current utterance, start new |
| `Esc` (panel dismiss, AC-06) | Stop speech immediately, flush queue |
| PTT key-down | Duck/stop speech (full-duplex policy: stop, simpler and safer in v0) |
| Toggle off mid-speech | Stop + flush |

One active utterance at a time, strictly newest-wins.

### 4.5 Toggle (F-11) and defaults

- **Default: OFF.** Rationale: unsolicited computer voice is intrusive in
  shared spaces; the PRD positions TTS as a toggle (F-11 "toggle"), and the
  trust counter-metrics favor conservative defaults. One click/tray setting
  to enable; remembered.
- When ON, a per-answer mute button appears in the panel (cheap escape
  hatch without flipping the global toggle).

### 4.6 Voice selection & language handling

- Voice config: `{en: voice_id, zh: voice_id}` chosen in settings; defaults
  ship with one voice per language (bundled/downloaded per §4.1).
- Per-sentence script detection (Han-range check) selects the voice; mixed
  sentences are read by the dominant script's voice.
- Rate/pitch exposed as simple sliders; persisted in `TtsConfig`.

### 4.7 Failure modes

| Failure | Behavior |
|---|---|
| Neural engine init failure | Auto-fallback to system voices; tray indicator notes "basic voices" |
| zh voice missing | zh sentences read by en voice with warning once; settings nudge |
| Synthesis error mid-answer | Skip sentence, continue; log only |
| Audio output device gone | Stop speech, text unaffected |

## 5. Interfaces & Data Structures

```json
// brain → shell: tts.speak (only when toggle ON)
{ "answer_id": "ans_01H...", "text_stream_ref": "ipc:agent.tokens" }
// shell-internal: sentence chunks with {lang, voice_id, text}
```

- `TtsConfig {enabled: false, en_voice, zh_voice, rate, pitch}`.

## 6. Risks & Failure Modes

| Risk | Impact | Mitigation |
|---|---|---|
| Neural engine RAM pushes the envelope | Idle budget pressure (RFC-0002 §4.8) | Voices loaded lazily on first speak; engine unloaded after idle period |
| Voice download adds setup friction | Activation dip | System-voice fallback covers day one |
| zh quality varies by platform system voices | Inconsistent feel | Neural primary path; system fallback marked "basic" |
| Speech feels intrusive | Counter-metric damage | OFF by default, per-answer mute (§4.5) |

## 7. Open Questions

1. Piper-class engine bindings quality in Rust for M3 — maintainability vs
   shelling to the reference binary?
2. Time-to-first-audio measured with a realistic first sentence (target
   ≤ 500 ms — validate)?
3. Do we need prosody controls beyond rate/pitch in v0 (probably no)?
4. Voice bundling (installer size) vs first-run download — mirror RFC-0004's
   consent flow?

## 8. Dependencies

- **RFC-0002** — panel `Esc` wiring stops speech; tray/settings UI; RAM
  envelope; audio output device handling.
- **RFC-0007** — token stream is the input; sentence splitter contract.
- **RFC-0009** — TTS is network-free by construction; no data leaves; nothing
  persisted.

## 9. Acceptance Mapping

No PRD AC targets TTS directly; it supports **J2** (voice in, voice out) and
respects **AC-06** semantics (Esc dismiss = silence). Toggle behavior matches
F-11's "toggle" wording; OFF-by-default decision recorded in §4.5 for PRD
sign-off.

## 10. Milestone Alignment

- **M3:** neural primary + system fallback, sentence streaming, interruption
  semantics, per-language voices.
- **Post-v0:** prosody polish, per-app voice profiles.
