# Ruòxī (若曦) — Business PRD
### Desktop Companion Assistant · MVP v1.0

- **Codename:** Ruòxī (若曦) — "as clear as dawn's first light"
- **Owner:** Dani
- **Status:** Approved to build
- **License:** MIT (open source)
- **Platforms:** macOS + Windows
- **Document type:** Business / Product PRD (technical design → see RFC-0001)

---

## 1. Executive Summary

Ruòxī is a quiet AI assistant that lives in your tray/menu bar. It **sees your
screen** and **hears your voice** — press a hotkey, point at anything (or the
whole screen), speak, and get help **without leaving your current app**. It
remembers what matters locally, so it grows into a personal second brain.

> 云思考，本地记忆。 Cloud reasons; local remembers.

---

## 2. Problem Statement

**The friction:** getting AI help mid-work means breaking flow — switch apps,
copy-paste context, lose your place, re-explain yourself.

**The gap:** screen-aware tools skew to meetings; dictation tools write text but
don't *understand*. Nothing offers a calm, trustworthy, always-there companion
that both sees and hears, at any scope you choose, with data kept local.

**The cost to the user:** dozens of micro-interruptions a day, each costing
focus. 打断一次，重来一次。

---

## 3. Target Users

| Persona | Who | Core need |
|---|---|---|
| **Primary — "Dani the Learner"** | Studies + works deep in browser, PDFs, docs | On-screen help without losing focus; privacy |
| **Secondary — "The Generalist"** | Any knowledge worker | An ambient, always-there helper |

**Shared traits:** values privacy · uses hotkeys · macOS-first (Windows second) ·
distrusts tools that ship data to a server.

---

## 4. Value Proposition & Positioning

> **"Point at anything, speak, and get help — without leaving your app."**

| We are | We are not |
|---|---|
| A calm tray companion | A chat window you must visit |
| Screen-aware **and** voice-native | A meeting copilot / a dictation keyboard |
| Local-first (your data stays) | A cloud-synced account |
| A second brain that compounds | A stateless Q&A box |

**The moat:** the agentic loop over *your own* local memories. The more you use
it, the more it knows you. 越用越懂你。

---

## 5. Core Concepts

| Concept | Definition |
|---|---|
| **Point & Speak** | The signature gesture: capture *something* and/or talk. |
| **Capture spectrum** | Region (precise) → Active window (context) → Full screen (everything). |
| **Two verbs** | **Ask** · **Save** |
| **Three memories** | Episodic (what happened) · Semantic (what's true) · Procedural (how you work). |

---

## 6. MVP Features (Product View)

Priority = MoSCoW. *This section lists product capabilities only; implementation is in the RFC.*

### MUST — the product
| ID | Feature | User-facing value |
|---|---|---|
| F-01 | Tray / menu-bar resident | Always one keystroke away; never in the way |
| F-02 | Push-to-talk voice | Ask hands-free, anywhere |
| F-03 | Region capture → Ask | "Explain *this*" — the magic moment |
| F-04 | Active-window capture → Ask | Effortless "what's in this doc/app?" |
| F-05 | Full-screen capture → Ask | "Help me with everything here" |
| F-06 | Floating result panel (`Esc` dismiss) | Non-intrusive by design |
| F-07 | **Memory-grounded answers** ⭐ | Answers built from *your* notes & history |
| F-08 | Local capture store + searchable timeline | Your screen history becomes a library |
| F-09 | Cloud indicator + offline kill-switch | Trust you can see and control |

### SHOULD — the companion feel
| ID | Feature | Value |
|---|---|---|
| F-10 | Save-to-memory from any answer | One-offs become permanent knowledge |
| F-11 | TTS read-back (toggle) | Hear explanations while you read |
| F-12 | Daily brief on tray click | Ambient value, zero prompting |

### COULD — power moves
| ID | Feature | Value |
|---|---|---|
| F-13 | Auto-capture opt-in per app | Passive second brain |
| F-14 | Multi-monitor display picker | Precise capture on big setups |

### WON'T (v0)
Machine control (click/type/edit) · flashcards · multi-tool loops · monetization.

---

## 7. Use Cases & User Journeys

> Each journey tags the features it exercises, so acceptance can be traced.

### J1 — "Explain this" (the core loop) 🟢
**As a** student reading a dense PDF,
**I want** to ask about one paragraph without leaving the page,
**so that** I keep my place and momentum.
**Flow:** box paragraph (`F-03`) → Ask "explain simply" (`F-07`) → read answer in panel (`F-06`) → hit Save (`F-10`) → note lands in Semantic memory.
**Features:** F-03 · F-06 · F-07 · F-10

### J2 — "Ask by voice while reading" 🟢
**As a** reader mid-page,
**I want** to speak a question about what's on screen,
**so that** I don't touch the keyboard.
**Flow:** hold PTT (`F-02`) → speak ("summarize this") → answer in panel (`F-06`) → optional TTS (`F-11`).
**Features:** F-02 · F-06 · F-11

### J3 — "What did I save about X?" (memory recall) ⭐
**As a** learner revisiting a topic,
**I want** to ask my own captured knowledge,
**so that** the assistant answers with my material, not the internet's.
**Flow:** PTT (`F-02`) → ask "what did I save about mitochondria?" → agent searches Semantic memory (`F-07`) → answer **with the original screenshot cited** (`F-08`).
**Features:** F-02 · F-07 · F-08

### J4 — "Help me with everything here" (work) 🟡
**As a** worker juggling many open things,
**I want** the assistant to see my whole screen,
**so that** I get situational help.
**Flow:** full-screen capture (`F-05`) → Ask "summarize what's happening and what's next" → answer with cited captures (`F-07`, `F-08`).
**Features:** F-05 · F-07 · F-08

### J5 — "Explain this trend" (chart) 🟡
**As an** analyst,
**I want** to box a chart and ask about it,
**so that** I read it correctly fast.
**Flow:** active-window or region capture (`F-04`/`F-03`) → Ask "explain this trend + likely cause" → answer.
**Features:** F-03 · F-04 · F-06

### J6 — "Daily brief" 🟡
**As a** returning user,
**I want** a one-line summary when I click the tray,
**so that** I re-orient instantly.
**Flow:** tray click (`F-12`) → daily brief.
**Features:** F-01 · F-12

### J7 — "Nothing leaves my machine" (privacy) 🔵
**As a** privacy-conscious user,
**I want** to work fully offline on demand,
**so that** I trust the tool with real work.
**Flow:** toggle offline (`F-09`) → captures and answers stay local → no outbound calls.
**Features:** F-09

### J8 — "First run" (onboarding) 🔵
**As a** new user,
**I want** to grant permissions with understanding,
**so that** I trust the app from minute one.
**Flow:** launch → guided ritual explains Screen Recording, Microphone, Accessibility, each with a one-line *why* → first demo capture runs → "it works!" moment.
**Features:** F-01 · F-03 · F-09

---

## 8. Acceptance Criteria

> Testable, Given / When / Then. One AC per must-have feature.

| ID | Criterion |
|---|---|
| **AC-01** | **Given** the app is running, **when** I look at the tray, **then** its icon is present and no Dock/taskbar window clutters my desktop. |
| **AC-02** | **Given** I hold the PTT hotkey, **when** I speak and release, **then** a transcript is produced and an answer appears in the panel within the latency budget (§ latency in RFC). |
| **AC-03** | **Given** I press the region hotkey, **when** I drag a box and release, **then** exactly the dragged region is captured and used as context. |
| **AC-04** | **Given** I trigger active-window capture, **then** the frontmost window (not the desktop) is captured. |
| **AC-05** | **Given** I trigger full-screen capture, **then** the display under my cursor is captured at native resolution. |
| **AC-06** | **Given** an answer is shown, **when** I press `Esc`, **then** the panel dismisses without side effects. |
| **AC-07** | **Given** I ask a question related to a saved note, **then** the answer cites at least the matching local source (screenshot/note id). |
| **AC-08** | **Given** I save an answer, **then** it is retrievable later by a memory query and appears in the timeline. |
| **AC-09** | **Given** offline mode is ON, **when** I use any feature, **then** zero network calls are made and the UI shows the offline state. |
| **AC-10** | **Given** any speech session ends, **then** the temporary audio is removed within ~1 minute and never appears in the UI. |
| **AC-11** | **Given** the agent is responding, **then** no more than 3 tool steps occur, and `Esc` can abort mid-loop. |
| **AC-12** | **Given** a first run, **then** each permission prompt is preceded by a plain-language *why* line. |

---

## 9. Success Metrics 📊

### 🌟 North Star Metric
> **Memory-Grounded Assists per Active User per Day (MGA/D)**
>
> Count of daily interactions where the assistant delivers a useful answer —
> and **at least one is grounded in the user's own memory**.

**Why this one:** it captures all three truths at once — the user actually *shows
up* (engagement), gets *value* (answer delivered), and the *differentiator fires*
(memory grounding). A chatbot can have high usage without memory; Ruòxī's whole
identity is compounding memory. 记忆，是北极星。

### Supporting metrics
| Layer | Metric | Target (MVP) |
|---|---|---|
| **Activation** | % of new users who complete a successful capture in session 1 | ≥ 60% |
| **Speed** | Intent → answer, p50 | ≤ 2s |
| **Retention** | D7 return of users who activated | ≥ 40% |
| **Differentiation** | % of answers using memory | ≥ 30% |
| **Breadth** | Use of all three capture scopes | ≥ 40% use 2+ scopes |
| **Trust** | Sessions where user mis-believes data left the machine | < 1% |

### Counter-metrics (guard against gaming) 🛡️
| Metric | Why |
|---|---|
| **False-grounding rate** | Answers citing *wrong* memory — must stay near zero |
| **Unwanted captures** | Captures the user didn't intend |
| **Permission abandonment** | Users who quit at the permissions wall |
| **Latency p95** | Protects worst-case feel, not just median |

---

## 10. Scope

**In (MVP):** the MUST + SHOULD features above, macOS + Windows, single user.
**Out (v0):** machine control, IDE integrations, teams, on-device LLM, monetization.

---

## 11. Release Plan (product level)

| Release | Theme | Content |
|---|---|---|
| **v0.1 (M0)** | It lives | Tray, PTT, 3 capture modes, panel |
| **v0.2 (M1)** | It answers | STT + LLM responses |
| **v0.3 (M2)** | It remembers ⭐ | Memory-grounded answers |
| **v0.4 (M3)** | It feels | Save, TTS, brief, offline, onboarding |
| **v1.0** | It acts | Native-control adapter (Phase 2) |

---

## 12. Product Risks

| Risk | Mitigation |
|---|---|
| Users don't discover the hotkeys | Onboarding demo + tray hints |
| Memory feels like surveillance | Explicit Save default + visible cloud indicator |
| Full-screen capture = noisy answers | Scope hints + let user crop the answer ("focus here") |
| Permissions wall kills activation | Guided ritual with *why* per permission |
| Open-source forks dilute brand | Strong identity + docs + roadmap (MIT is fine with this) |

---

## 13. Decisions Log

| # | Decision |
|---|---|
| 1 | Product = general desktop assistant (study flagship) |
| 2 | North Star = Memory-Grounded Assists / active user / day |
| 3 | Three memories: episodic / semantic / procedural |
| 4 | Audio ephemeral (no replay) |
| 5 | Open source, MIT; monetization deferred |
| 6 | Free usefulness first; v1 adds "act" layer |
