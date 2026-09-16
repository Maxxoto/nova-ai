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
