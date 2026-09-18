# RFC-0004 — Voice Input (Local STT)

- **Status:** Draft for review
- **Author:** Dani
- **Parent:** [RFC-0001](RFC-0001-desktop-companion.md) (Desktop Companion)
- **Companion:** [Business PRD](../ruoxi_prd.md)
- **Covers (PRD):** F-02 (push-to-talk voice)
- **ACs:** AC-02, AC-10 · **Milestone:** M1 ("It answers")

---

## 1. Summary

Local speech-to-text for push-to-talk: hold hotkey → mic opens → release →
utterance transcribed by a whisper-family model via `whisper-rs` (Rust
bindings to whisper.cpp) inside the Rust core — per RFC-0001's amendment, the
Rust core is the native seam where STT lives. Audio is **ephemeral** (AC-10,
PRD decision #4): in-memory ring buffer, at most one transient temp file,
deleted within ~1 minute, never surfaced in the UI.

## 2. Motivation

Voice is half of "point and speak" (J2). It must be fully local: sending audio
to a cloud STT would break the local-first promise and AC-09's zero-network
guarantee in offline mode. whisper.cpp via Rust gives CPU-friendly inference
without a Python ML stack in the brain service.

## 3. Goals / Non-Goals

**Goals**
- PTT lifecycle bound to raw key-down/key-up (from RFC-0002).
- Good bilingual zh/en accuracy (user works in English + Chinese).
- STT sub-budget within the p50 ≤ 2s intent→answer target (RFC-0007).
- Fits the shared < 150 MB idle RAM envelope (RFC-0002 §4.8).
- Verifiable audio ephemerality (AC-10).

**Non-Goals**
- Always-on listening / wake-word (PRD: hotkey-triggered only).
- Speaker diarization, audio storage, replay (decision #4).
- Cloud STT fallback.

## 4. Detailed Design

### 4.1 Placement: Rust core vs Python brain

| | whisper-rs in Rust core (chosen) | whisper in Python brain |
|---|---|---|
| Latency | No IPC hop for audio; PTT events already in Rust | Extra hop: shell → brain → result back |
| RAM | Single process space; shares the envelope accounting | Two model-capable processes; double ML deps |
| Coupling | Keeps brain LLM-only, ML deps out of Python | Pulls PyTorch/ctranslate-class deps into brain |

**Decision: Rust core** (aligns with RFC-0001 amendment naming whisper-rs as
a Rust-core responsibility). The brain receives only the final transcript via
IPC (`stt.final`).

### 4.2 Model selection matrix (approximate — validate in M1 spike)

| Model | Size (quantized, approx.) | RAM (approx.) | en | zh | Verdict |
|---|---|---|---|---|---|
| tiny | ~75 MB q5 | ~70–100 MB | usable | weak | too weak for zh |
| base | ~140 MB q5 | ~100–150 MB | ok | marginal | fallback |
| small | ~460 MB q5 | ~250–400 MB | good | decent | quality option |
| distil variants (en-only) | small | small | good | none | rejected: no zh |

- **Default recommendation: `base` quantized for M1 bring-up; `small`
  quantized as the quality default if the RAM spike holds.** Bilingual
  requirement rules out en-only distils.
- Numbers are manufacturer-independent approximations to be measured on the
  M1 reference machines; final default set by spike result, not by this table.

### 4.3 Model distribution

- **Recommendation: first-run download** (with explicit consent + progress UI)
  over bundling: keeps the installer tens-of-MB instead of hundreds; the
  download is the ONLY model-related network event and is skipped entirely if
  the user imports a model file manually (offline-friendly).
- Model file lives in the app data dir; checksum-verified on load.

### 4.4 PTT lifecycle state machine

```
idle ──key-down──► capturing ──key-up──► transcribing ──final──► idle
                      │                                        ▲
                      └── max 60s cap / mic error ──────────────┘
```

- **capturing:** mic opened via a Rust audio-input mechanism (cpal-class);
  16 kHz mono PCM into a lock-free ring buffer sized for 60 s (~1.9 MB) —
  the hard utterance cap.
- Optional VAD pre-trim of leading/trailing silence (cheap energy-based in
  v0; smarter VAD later) — reduces hallucinated text on silence.
- **transcribing:** buffer → whisper in a single pass; interim partial
  hypotheses displayed in the panel only if free (v0: single final pass —
  see §4.5).
- Cancellation: Esc during transcribing discards the buffer entirely.

### 4.5 Streaming strategy

| | Partials during hold | Final pass on release (v0) |
|---|---|---|
| Latency feel | Transcript appears while speaking | Transcript appears after release |
| Complexity | Needs streaming decode + partial UI | Simple |
| Accuracy | Partials can mislead | Best accuracy |

**Decision: final pass on release for v0.** Rationale: the ≤ 2s p50 budget is
dominated by short utterances where a single small-model pass is fast
(target: ≤ 800 ms for ≤ 10 s audio — to validate); partials add UI + decode
complexity that M1 doesn't need.

### 4.6 Latency budget

STT owns the segment *key-up → transcript delivered* of RFC-0007's budget
table. Proposed allocation (targets): ≤ 800 ms p50 for ≤ 10 s utterances on
the reference hardware; p95 guard ≤ 2 s; over-budget → auto-downscale model
or trim policy. Final numbers set by the M1 spike and recorded in RFC-0007.

### 4.7 RAM strategy vs the 150 MB envelope

| | Lazy-load on first PTT | Preload at launch |
|---|---|---|
| Idle RAM | ~0 until first use | +model size always |
| First-use latency | +model load (~hundreds of ms to s) | none |

**Recommendation: lazy-load, then keep resident** (load on first PTT of the
session, keep for the session): idle-before-first-use is minimal, and
"frequently used companion" pays load cost once. The < 150 MB target in
RFC-0001 risk #4 is measured with whisper loaded — i.e., after first use.
If the spike shows `small` breaks the envelope, default drops to `base`.

### 4.8 Audio ephemerality (AC-10)

- Audio exists as: (1) the in-memory ring buffer, and (2) at most one temp
  file if the transcription path requires disk spill (deleted immediately
  after decode).
- Sweep job: any `stt-tmp-*` file older than 60 s is deleted unconditionally.
- UI/telemetry/notes never receive audio paths or waveforms.
- **Verification:** an automated test asserts (a) no audio files older than
  60 s exist under the data dir after a scripted session, (b) no audio path
  appears in any IPC message or log line (grep-based assertion in CI).

### 4.9 Error paths

| Failure | Behavior |
|---|---|
| Mic permission denied | Panel hint re-opens onboarding step (RFC-0002 §4.5) |
| Input device changes mid-hold | End utterance cleanly at buffer state; toast |
| Model missing/corrupt | Prompt import/re-download; PTT disabled with tray indicator |
| Empty/silence buffer | No-op with subtle panel feedback (no hallucinated text) |
| Unsupported output language | Transcript still delivered; language tag marked `low-confidence` |

## 5. Interfaces & Data Structures

```json
// IPC notification: stt.final  (Rust core → brain → panel)
{ "text": "explain this chart", "lang": "en",
  "duration_ms": 4200, "stt_ms": 610, "confidence_hint": "normal" }
```

- `SttConfig { model, lang_hint: "auto"|"en"|"zh", vad_trim: bool }`.
- Model registry entry: `{id, file, sha256, size_bytes, source_url}`.

## 6. Risks & Failure Modes

| Risk | Impact | Mitigation |
|---|---|---|
| RAM envelope broken by `small` | Always-resident promise at risk | Spike; fall back to `base`; lazy-then-resident policy |
| zh accuracy poor on small models | Core user (bilingual) underserved | Quality option + spike on real zh audio; model swappable |
| Key-up spike (RFC-0002 risk #1) fails | PTT dead | Toggle-mode fallback owned by RFC-0002 |
| Whisper hallucination on silence | Garbage queries | VAD trim + empty-buffer no-op |
| First-run download blocked (offline user) | No STT | Manual model import path |

## 7. Open Questions

1. Measured RAM/latency of `base` vs `small` quantized on the reference
   MacBook + Windows box (M1 spike — decides the default)?
2. Is zh accuracy of `small` q5 acceptable for J2 journeys, or do we need a
   zh-specialized variant?
3. Temp-file spill: can we stay purely in-memory (buffer → decoder) and drop
   the temp file entirely?
4. Ring buffer cap of 60 s — sufficient for real study-question utterances?

## 8. Dependencies

- **RFC-0002** — PTT raw key-down/key-up events; RAM envelope; model
  distribution consent UI; error-path toasts.
- **RFC-0007** — latency budget table (this RFC contributes the STT segment);
  transcript consumed by the agent loop.
- **RFC-0009** — audio ephemerality enforcement + verification harness;
  offline mode never touches STT (fully local).

## 9. Acceptance Mapping

| AC | Mechanism |
|---|---|
| AC-02 (STT half) | PTT → transcript → answer within budget; §4.4–4.6 |
| AC-10 | Ring buffer + 60 s sweep + no-path-in-UI contract + CI assertions (§4.8) |

## 10. Milestone Alignment

- **M1:** final-pass STT with `base` default (post-spike), ephemerality harness.
- **Post-M1:** partial streaming, zh quality option, smarter VAD.
