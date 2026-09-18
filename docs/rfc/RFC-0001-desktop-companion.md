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
