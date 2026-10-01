# Ruoxi — system flow (shell · brain · ui)

How the pieces talk. The **Tauri shell (Rust)** owns the windows, capture, tray and the
sidecar; the **WebView UI (React + TS)** renders every surface; the **Python brain
sidecar** answers over stdio JSON-RPC. Every name below matches the code:
`shell/src-tauri/src/`, `shell/ui/src/`, `src/app/interfaces/sidecar/`.

Related: [RFC-0002](rfc/RFC-0002-platform-shell.md) (shell) · [RFC-0003](rfc/RFC-0003-screen-capture.md)
(capture) · [DESIGN.md](DESIGN.md) (contract) · [shell/README](../shell/README.md).

**As-built (2026-10-01):** `session.ask` runs the repo's real agent loop (tools, memory,
LiteLLM routing) in the sidecar — frozen with PyInstaller and shipped inside the app
(`Contents/Resources/brain/`). Memory has its own section below (§7).

## 1 · Component map

```mermaid
flowchart TB
  subgraph mac["macOS host"]
    HOT["global hotkeys — Carbon<br/>Alt+Shift+R / W / F"]
    TAP["PTT key tap<br/>F8 hold to talk (rebindable)"]
    TCC["TCC permissions<br/>Screen Recording · Accessibility · Microphone"]
    XCAP["xcap — display / window bitmaps"]
  end

  subgraph shell["Tauri shell — nova-shell (Rust)"]
    TRAY["tray.rs<br/>menu · state icons (template)"]
    HKS["hotkeys.rs · ptt.rs<br/>CaptureIntent · key tap"]
    CAP["capture.rs + capture worker<br/>fullscreen · window · region"]
    OVL["overlay.rs<br/>region overlay window"]
    STORE[("capture_store.rs<br/>SQLite index + PNG files")]
    SUP["supervisor.rs<br/>spawn · ping · backoff"]
    BL["brain.rs — BrainLink<br/>mpsc queue + online flag"]
    WIN["windows<br/>panel · settings · onboarding · timeline · overlay"]
  end

  subgraph ui["WebView UI — React + TS (routes via ?view=)"]
    PANEL["panel — live result"]
    OTHERS["onboarding · settings<br/>timeline · overlay · review board"]
  end

  subgraph py["Python brain sidecar"]
    SRV["app.interfaces.sidecar<br/>stdio JSON-RPC (M0 stub)"]
    AGENT["agent loop · tools · memory<br/>M1+ behind the same seam"]
  end

  HOT --> HKS
  TAP --> HKS
  HKS --> CAP
  HKS --> OVL
  CAP --> XCAP
  OVL --> XCAP
  TCC -. gates .-> XCAP
  CAP --> STORE
  OVL --> STORE
  CAP --> WIN
  STORE --> WIN
  TRAY --> WIN
  SUP --> SRV
  SRV --> AGENT
  WIN <-->|"Tauri IPC — invoke / emit"| PANEL
  WIN <-->|"Tauri IPC"| OTHERS
  BL --> SUP
```

## 2 · Capture → answer (the full loop)

```mermaid
sequenceDiagram
  autonumber
  actor U as You
  participant HK as Rust · hotkeys
  participant CW as Rust · capture
  participant XC as xcap
  participant ST as store
  participant PN as Rust · panel
  participant UI as UI · panel
  participant BL as Rust · brain pump
  participant PY as Python · sidecar

  U->>HK: Alt+Shift+F — or Alt+Shift+R (region overlay)
  HK->>CW: CaptureIntent
  Note over CW,ST: region flow hides the overlay first — our chrome is never in the shot
  CW->>XC: capture at native pixels
  XC-->>CW: RGBA bitmap (scale-aware)
  CW->>ST: insert (sha256 dedupe → file + row)
  ST-->>CW: record {capture_id, ts, …}
  CW->>PN: show + focus
  CW-->>UI: emit panel:capture {id, at_ms}
  UI->>BL: invoke session_ask {transcript, captureIds}
  BL->>PY: {"method":"session.ask","params":{…}}
  loop streamed answer
    PY-->>BL: notification agent.token {delta}
    BL-->>UI: emit panel:token {delta}
  end
  PY-->>BL: response {answer, aborted}
  BL-->>UI: emit panel:complete {answer}
  U->>UI: Esc
  UI->>PN: invoke hide_panel (+ session_abort while streaming)
```

## 3 · Sidecar supervision + brain link

```mermaid
flowchart LR
  subgraph rustside["Rust core"]
    CMD["commands<br/>session_ask · session_abort"] --> LINK["BrainLink<br/>mpsc queue · online flag"]
    LINK --> PUMP["supervisor pump<br/>select! ping tick | queue"]
    PUMP --> PROC["SidecarProcess<br/>stdin/stdout · request ids"]
  end
  subgraph pyside["Python sidecar"]
    SERVER["server.py<br/>readline loop → dispatch"]
  end
  PROC <-->|"newline-delimited JSON-RPC"| SERVER
  PUMP -. "panel:token · panel:complete · panel:error" .-> UIE["WebView events"]
```

- **Spawn:** `settings.json → sidecar_command` / `sidecar_args` (e.g. the repo
  `.venv/bin/python -m app.interfaces.sidecar`).
- **Health:** `ping` every `ping_interval_secs` (default 5 s, 3 s timeout).
- **Restart:** exponential backoff (1, 2, 4, 8, 16 s, max 5) → then the tray tooltip
  says the brain is offline.
- **While an ask streams:** a queued `session.abort` is written immediately; other
  requests wait behind the stream (30 s ask timeout → `panel:error`).

## 4 · IPC surfaces (three hops)

```mermaid
flowchart LR
  JS["WebView (JS)"] -- "invoke(command, args)" --> RUST["Rust core"]
  RUST -- "emit(event, payload)" --> JS
  RUST -- "JSON-RPC request" --> PY["Python sidecar"]
  PY -- "response / notification" --> RUST
```

**JS → Rust — `invoke` commands (Tauri):**
`get_settings` · `set_settings` · `show_settings` · `show_onboarding` ·
`show_panel` · `hide_panel` · `session_ask` · `session_abort` ·
`permissions_status` · `permissions_request` · `open_privacy_pane` ·
`capture_store_stats` · `capture_delete_all` · `show_timeline` · `timeline_list` ·
`capture_thumbnail` · `list_displays` · `start_region_capture` · `overlay_cancel` ·
`capture_region_commit` · `validate_hotkey` · `validate_dismiss_hotkey` ·
`memory_save_semantic` · `memory_list` · `memory_archive` · `memory_archived_list` ·
`memory_restore`

**Rust → JS — `emit` events:**
`panel:capture {id, at_ms}` · `panel:token {delta}` · `panel:complete {answer}` ·
`panel:error {message}` · `settings:changed {…settings}`

**Rust ⇄ Python — stdio JSON-RPC (newline-delimited):**
requests `ping` · `session.ask {transcript, capture_ids}` · `session.abort {}`;
notifications `agent.token {answer_id, delta}`; responses `{answer_id, answer, steps, aborted}`.

## 5 · Permissions & first run

```mermaid
flowchart TB
  UI["onboarding · settings (JS)"] -->|invoke permissions_status| P["Rust · permissions.rs"]
  P --> SR["CGPreflightScreenCaptureAccess"]
  P --> AXP["AXIsProcessTrusted"]
  P --> MIC["AVCaptureDevice.authorizationStatus"]
  UI -->|invoke permissions_request| RQ["CGRequestScreenCaptureAccess<br/>AX prompt option<br/>requestAccessForMediaType"]
  RQ -. "fallback / revoke" .-> PANE["open System Settings panes"]
```

- The setup ritual (J8, AC-12) shows a **why-line before each OS prompt**; the gate
  needs **Screen Recording + Accessibility** (the microphone is read but never blocks —
  macOS reports it as `unknown`/`not determined` honestly).
- Without **Screen Recording** macOS returns wallpaper-only captures (no other apps'
  windows); without **Accessibility** hold-to-talk cannot tap the keyboard.

## 6 · Storage

```mermaid
flowchart LR
  APP["app data dir<br/>~/Library/Application Support/com.ruoxi.shell"] --> S["settings.json"]
  APP --> DB[("capture_index.sqlite<br/>captures: id · ts · scope · app · sha256 · …")]
  APP --> FILES["captures/YYYY/MM/DD/*.png"]
  APP --> MEM["memory/<br/>episodic/YYYY/MM/DD · semantic · procedural ·<br/>digest/MEMORY.md · archive/ · index.sqlite (FTS5)"]
  DB -. "dedupe by sha256 · day/app/scope indexes" .- FILES
  MEM -. "files are the source of truth — index rebuilds from drift" .- MEM
```

## 7 · Memory flow (RFC-0006 · RFC-0011)

```mermaid
sequenceDiagram
  autonumber
  actor U as You
  participant UI as UI · panel / settings
  participant MS as Rust · memory.rs (MemoryStore)
  participant SU as Rust · supervisor
  participant PY as Python · sidecar

  Note over U,MS: Save (F-10) — explicit, LLM-free, offline-safe
  U->>UI: Save to memory
  UI->>MS: invoke memory_save_semantic {title, body, sourceRefs:[cap_id]}
  MS->>MS: dedupe — an identical body returns the existing note
  MS-->>UI: {id, created:true|false} → “Saved” / “Already saved”
  U->>UI: Undo (4 s)
  UI->>MS: invoke memory_archive {id} — file moves to memory/archive/

  Note over PY,SU: Episodic auto-log — the diary
  PY-->>SU: answer completes (panel:complete)
  SU->>MS: log_episodic — a date-sharded note with the capture ref

  Note over PY,MS: Retrieval — grounding with citations (AC-07)
  PY->>MS: memory.search {query, k}
  MS-->>PY: BM25 hits → Memory block + [mem_id]/[cap_id] citation guard

  Note over U,MS: Management (F-16)
  U->>UI: Settings → Memory — list · search · archive · restore
```

- **Files are the source of truth**: every note is Markdown with YAML
  front-matter (`id · type · created · source_refs · tags · confidence ·
  origin`); `index.sqlite` (FTS5) is derived and rebuilds when it drifts.
- **Archive-first**: nothing is hard-deleted — `memory_archive` moves a note
  to `memory/archive/` and removes it from retrieval; `memory_restore` puts it
  back. Identical saves dedupe instead of copying.
- **Planned** (RFC-0011, evidence in
  [`research/agent-memory-practices.md`](research/agent-memory-practices.md)):
  composite ranking (recency × importance × BM25, §D6) and a nightly
  reflection pass that distils episodes into facts (§D7). Vector search is
  deliberately not built (BM25 is competitive at personal-corpus scale).
