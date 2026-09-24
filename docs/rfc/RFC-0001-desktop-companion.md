# RFC-0001 — Ruòxī Desktop Companion (Technical Design)

- **Status:** Draft for review
- **Author:** Dani
- **Companion doc:** [Business PRD](../ruoxi_prd.md)
- **Scope:** MVP v0 (macOS + Windows)

> **⚠️ Amendment — 2026-09-17:** The app shell changed from **Electron → Tauri 2**.
> Rationale: RAM/battery budget for an always-resident tray companion (system WebView vs bundled
> Chromium); the Rust core doubles as the native adapter seam (capture, PTT raw key events,
> `whisper-rs`, and the v1 OS-control layer — removing the need for a separate Swift adapter);
> and the agent/memory brain lives in the Python/LangGraph service, so the shell stays thin.
>
> **Open risks — spike before sections 5+ are finalized:**
> 1. PTT needs key-down *and* key-up on global hotkeys (raw events on macOS / Windows) — neither framework's shortcut API gives key-up for free.
> 2. Transparent capture-overlay window quirks on Windows (DWM/acrylic).
> 3. Result-panel vibrancy/blur on WKWebView (macOS) and WebView2 (Windows).
> 4. Idle-RSS budget for the tray-resident app — target **< 150 MB**, panel closed, whisper loaded (watch tauri#5889, WebKit RAM edge cases).
> 5. Global-shortcut edge cases in fullscreen/game contexts (tauri#7318).

---

## 1. Summary

Ruòxī is a Tauri 2 desktop app with a thin **Core** and a swappable
**Platform Adapter**. It captures screen regions/windows/fullscreen, transcribes
voice locally, sends the live context to an OpenAI-compatible LLM endpoint, and
speaks answers via a local TTS engine. All persistent data (screenshots,
transcripts, answers, memories) stays on disk.

---

## 2. Motivation

Deep OS access (hotkeys, capture overlay, mic, tray) is required, but we want
one codebase for two platforms and a later native-control upgrade. An adapter
seam lets the Tauri shell ship now and Rust OS-control modules slot in later without a rewrite.

---

## 3. Goals / Non-Goals

**Goals**
- One codebase → macOS + Windows.
- Clean Core/Adapter boundary (no platform imports in Core).
- Fully local persistence; only the live reasoning request leaves the device.
- Three memory types (CoALA) as human-readable files.
- Latency within budget (see §12).

**Non-Goals**
- On-device LLM inference.
- Machine control in v0.
- Cloud sync / multi-user.
- Audio replay/storage.

---

## 4. Architecture Overview

```
┌──────────────────────────────────────────────┐
│ CORE (portable, no platform APIs)            │
│ • Panel/UI • Agent loop • Memory store       │
│ • STT/TTS orchestration • LLM client         │
└──────────────────────────────────────────────┘
                    │ Platform Adapter interface
                    ▼
   ┌───────────────────────┐   ┌──────────────────────┐
   │ Adapter: Tauri 2 (v0) │   │ Adapter: Native (v1) │
   │ macOS + Windows       │   │ Rust OS-control      │
   └───────────────────────┘   └──────────────────────┘
```

> *Note: the source document was truncated at §4 when it was first saved —
> sections 5–13 (including the §12 latency budget referenced in §3) still need
> to be added.*

---

> **⚠️ Amendment — 2026-09-21 (as-built boundary):** the §4 diagram predates the
> implementation. Three of its placements are inverted in the shipped code, and
> the Core/Adapter interface turned out to be **bidirectional**. Recorded here so
> the umbrella document matches reality — the intent of §4 (a clean, swappable
> platform seam) is unchanged.
>
> **Where components actually live**
>
> | Component | §4 diagram | As built | Why the inversion stands |
> |---|---|---|---|
> | Memory store | Core | **Adapter** — Rust `memory.rs`: sharded Markdown + FTS index + digest (RFC-0006 §4.2) | The store owns the capture data dir and is written on the ask path; keeping it in Rust avoids a cross-process write hop and keeps the index rebuildable from files |
> | STT | Core | **Adapter** — Rust `voice.rs`: whisper-rs + parakeet FFI (RFC-0004) | Native inference in-process; S2 measured 147.6 MB idle with the model lifecycle under our control |
> | TTS | Core | **Adapter** — Rust `tts.rs`: kokoro-micro (RFC-0005) | Sentence-streamed playback in ~0.6 s time-to-first-audio; an IPC hop per chunk would spend the budget on transport |
>
> Moving any of the three back into Python re-opens the RAM (§12-adjacent) and
> latency budgets, so the plan is to amend the boundary, not relocate the code.
>
> **CORE — portable reasoning, no platform APIs (frozen Python).** Agent loop +
> tool registry (`application/services/tool_loop.py`), LLM client/routing/degradation
> (`adapters/llm_providers`, RFC-0007 §4.4), context assembly and the citation
> guard, and at M3 consolidation + daily brief (RFC-0006 §4.6). Packaged as a
> single frozen executable (`ruoxi-brain`, PyInstaller) shipped **inside** the
> `.app` — the DMG carries adapter + core.
>
> **ADAPTER — Tauri 2 / Rust (macOS + Windows).** Everything needing OS access or
> native performance: lifecycle (tray, single instance, activation policy,
> settings, Keychain), input (global hotkeys, PTT raw key-down/key-up, Esc tap,
> permission rituals), capture (screen, region overlay, store + index), window
> shells (panel, overlay, onboarding, settings, timeline), native inference
> (whisper-rs/parakeet, kokoro-micro), audio I/O (cpal, rodio), model catalog +
> downloader, sidecar supervision, and the privacy kill-switch / cloud indicator.
>
> **The interface is bidirectional** (the §4 diagram draws one arrow):
>
> | Direction | Envelopes |
> |---|---|
> | Adapter → Core | `ping`, `health`, `session.ask`, `session.abort`, `config.test` |
> | Core → Adapter (reverse-RPC) | `capture.lookup`, `timeline.query`, `memory.search`, `memory.lookup`, `memory.digest` |
> | Core → Adapter (notifications) | `agent.token`, `agent.tool_step` |
> | Adapter → UI | `panel:token`, `panel:complete`, `panel:error`, `panel:tool_step`, `panel:listening`, `panel:transcript`, `panel:ask_cancelled` |
>
> The core is a **child process of the adapter** that calls back over the same
> stdio channel; the shell's `RequestRouter` chain is the concrete realisation of
> what §4 labels the "Platform Adapter interface". The adapter injects the core's
> runtime contract at spawn: LLM endpoint/key/model, the offline flag, and a
> sandboxed `RUOXI_WORKSPACE` for file/shell tools.
>
> **Freeze status:** not yet built. Today's DMG ships the adapter only, so the
> brain needs a Python with the repo deps (`sidecar_command` in settings). The
> freeze + `externalBin` packaging + signing is the remaining W6/M3 hardening.

---

## 5. Component RFC Index

> **Decomposition — 2026-09-18:** instead of finishing this document as a single
> monolith, the technical design is decomposed into per-component RFCs. Each is
> Status: Draft for review and traces to the PRD features (F-xx) and acceptance
> criteria (AC-xx) it covers. This document remains the umbrella: architecture,
> boundaries, and the cross-cutting risks listed above.

| RFC | Component | Covers (PRD) | Milestone |
|---|---|---|---|
| [RFC-0002](RFC-0002-platform-shell.md) | Platform Shell & Lifecycle (Tauri 2) — tray, hotkeys/PTT, overlay, panel, onboarding, IPC | F-01, F-06; AC-01, AC-06, AC-12 | M0 |
| [RFC-0003](RFC-0003-screen-capture.md) | Screen Capture & Visual Context Pipeline | F-03–F-05, F-08, F-13, F-14; AC-03–AC-05 | M0–M2 |
| [RFC-0004](RFC-0004-voice-input-stt.md) | Voice Input (Local STT, whisper-rs) | F-02; AC-02, AC-10 | M1 |
| [RFC-0005](RFC-0005-voice-output-tts.md) | Voice Output (Local TTS) | F-11 | M3 |
| [RFC-0006](RFC-0006-memory-system.md) ⭐ | Memory System (CoALA three-type, local, human-readable) | F-07, F-10, F-12; AC-07, AC-08 | M2–M3 |
| [RFC-0007](RFC-0007-agent-brain.md) | Agent Brain — loop, context assembly, LLM routing, latency budget | F-07, F-12; AC-02, AC-07, AC-11 | M1–M3 |
| [RFC-0008](RFC-0008-base-tools.md) | Base Tool Catalog & Registry (v0) | F-07, F-09; AC-07, AC-09, AC-11 | M1–M2 |
| [RFC-0009](RFC-0009-privacy-offline.md) | Privacy, Offline Mode & Data Governance | F-09; AC-09, AC-10 | M3 |

Cross-cutting ownership:

- The **latency budget** (the formerly missing §12) is owned by
  [RFC-0007](RFC-0007-agent-brain.md), decomposed per pipeline stage.
- The **RAM budget** (< 150 MB idle) is owned by
  [RFC-0002](RFC-0002-platform-shell.md), with allocations agreed in
  RFC-0004 (STT) and RFC-0005 (TTS).
- The five open risks above map primarily to RFC-0002 (risks 1, 2, 3, 5) and
  jointly to RFC-0002 + RFC-0004 (risk 4, idle RAM with whisper loaded).
