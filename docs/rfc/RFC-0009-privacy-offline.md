# RFC-0009 — Privacy, Offline Mode & Data Governance

- **Status:** Draft for review
- **Author:** Dani
- **Parent:** [RFC-0001](RFC-0001-desktop-companion.md) (Desktop Companion)
- **Companion:** [Business PRD](../ruoxi_prd.md)
- **Covers (PRD):** F-09 (cloud indicator + offline kill-switch), J7 ("nothing leaves my machine")
- **ACs:** AC-09, AC-10 · **Milestone:** M3 (enforcement seams land M0–M1)

---

## 1. Summary

The trust substrate: one offline kill-switch enforced at a single egress
choke point, a visible cloud indicator with honest states, a precise
inventory of what leaves the device (and what never does), audio
ephemerality enforcement, capture retention governance, and prompt-injection
defenses for untrusted screen content. Every claim here maps to a testable
mechanism (§9).

## 2. Motivation

The PRD's trust thesis: users "distrust tools that ship data to a server"
and the trust metric is **sessions where the user mis-believes data left the
machine < 1%**. Trust must be architectural (impossible to leak by accident),
visible (indicator states), and verifiable (CI-asserted zero-network in
offline mode). The alternative — policy-only privacy — is how companions
become surveillance stories (PRD §12).

## 3. Goals / Non-Goals

**Goals**
- Offline kill-switch: **zero network calls**, verifiable (AC-09).
- Cloud indicator states: offline / local-only / calling-cloud (F-09).
- Exact data-flow inventory; no undocumented egress.
- Audio ephemerality enforced + verified (AC-10).
- Retention controls: delete-all/day, pause, quota (with RFC-0003).
- Prompt-injection defense layers for untrusted captures/transcripts.

**Non-Goals**
- Cloud sync (PRD non-goal), accounts, telemetry of any kind (none ships),
- E2E encryption of local stores (OS user accounts suffice in v0).

## 4. Detailed Design

### 4.1 Offline kill-switch: single choke point

**Design: one shared `NetPolicy` object owned by the brain, mirrored to the
shell.** Every network-capable surface routes through it:

| Surface | Route | Offline behavior |
|---|---|---|
| LLM calls (LiteLLM adapter) | adapter checks `NetPolicy` before every request | hard-block; offline answer path (RFC-0007 §4.7) |
| Network tools (`web.fetch/search`) | registry trust gate (RFC-0008 §4.3) | hard-fail with structured error |
| Model/voice downloads (first-run) | download manager checks policy | deferred with notice |
| Update checks | manual "check for updates" only | in-v0: the button itself is disabled offline |
| Telemetry / crash reporting | **does not exist in the product** | n/a |

The `NetPolicy` flag is user-set (tray + onboarding), persisted, and its
state is the single source for the indicator (§4.2). Setting it ON is
instant (no restart): the flag is read at each gate.

### 4.2 Cloud indicator (F-09)

| State | Shown when |
|---|---|
| Offline (kill-switch ON) | always, in tray + panel chrome |
| Local-only | online, no request in flight |
| Calling-cloud | an LLM request or network tool is in flight (per-call, honest sub-second fidelity) |

Transitions are event-driven from `NetPolicy` + request lifecycle, not
timers — the indicator must never lie by lagging.

### 4.3 Data-flow inventory (what leaves the device)

| Leaves | When | Contains | Destination |
|---|---|---|---|
| Live reasoning request | user asks, online | transcript, downscaled capture image (RFC-0003 §4.5), retrieved memory snippets included as context, persona/procedural digest | **user-configured OpenAI-compatible endpoint** (may itself be a local server — fully-local setups exist) |
| Network tool calls | agent calls web.* in-loop | the URL/query only — never memory file contents or capture images | fetch/search targets (subject to allowlist, §4.6) |

| Never leaves | Ever |
|---|---|
| Raw audio | deleted ≤ 1 min (§4.4) |
| Capture store images (beyond the single active request image) | local only |
| Memory files, notes, todos, sessions, index | local only |
| Usage stats | do not exist |

### 4.4 Audio ephemerality (AC-10)

Mechanism owned by RFC-0004 §4.8; this RFC owns the **verification harness**:
scripted PTT session → assert (a) no audio artifacts older than 60 s anywhere
under the app data dir, (b) no audio path/waveform in IPC logs, UI state, or
memory entries (grep assertions in CI). PRD decision #4 ("audio ephemeral, no
replay") is thus testable, not aspirational.

### 4.5 Capture retention governance

- Defaults and controls per RFC-0003 §4.7 (manual-delete default, optional
  auto-delete window, quota + LRU trim, pin, pause).
- This RFC adds the **privacy UX contract**: onboarding states plainly what
  is stored, where, and how to purge; "delete all captures" also sweeps
  memory `source_refs` to `[deleted capture]` (RFC-0006); pause-capture is
  one tray click and visually obvious (no quiet capture while paused).
- F-13 auto-capture remains per-app opt-in, OFF by default — the
  "surveillance feel" mitigation (PRD §12).

### 4.6 Prompt-injection defense (untrusted screen content)

Captures and transcripts are **untrusted input**. Contract — screen text must
never cause:

1. **Exfiltration:** `web.fetch` URLs constructed from screen content that
   embed memory/personal data. Defenses: fetch is GET/text-only;
   egress allowlist (user-managed, curated default); URL parameters
   length/entropy review before dispatch; indicator shows calling-cloud
   state during any fetch (visible exfil window).
2. **Silent memory writes:** no semantic/procedural write without explicit
   user Save/confirm; `origin` provenance enforced (RFC-0006 §4.9); episodic
   auto-logs record but never instruct.
3. **Runaway loops:** hard 3-step budget + abort (RFC-0007 §4.1) bounds any
   injected "keep going" instructions.
4. Context delimitation: capture/transcript content is framed as data in the
   message layout (RFC-0007 §4.3).

**Residual risk (honest):** a determined injection may still steer the
answer's *tone/content* within one response. We bound blast radius (no
exfil path, no persistence, ≤ 3 steps) rather than pretend prevention.
Users handling hostile content should keep the allowlist tight.

### 4.7 Local data protection

- All state under one versioned `<data_dir>` (captures, memory, sessions,
  settings) with OS-appropriate user-only permissions on create.
- Export = copy dir; Wipe = delete dir + index rebuild. Both exposed in
  settings; wipe confirmed with typed confirmation.
- Secrets (endpoint API key) stored via the OS keychain mechanism, never in
  plaintext settings files.

### 4.8 OSS trust signals

- MIT (PRD decision #5); public roadmap; reproducible-build feasibility
  (pinned toolchains + hashes) — **open question**, not a v0 promise.
- Releases signed; signatures published alongside artifacts.

## 5. Interfaces & Data Structures

```json
// NetPolicy (shared state, evented)
{ "offline": true, "changed_at": "2026-09-18T15:04:11+07:00" }
// Indicator events: net.state {offline|local_only|calling_cloud}
```

- `PrivacySettings {offline, retention_days|null, quota_gb, allowlist[],
  auto_capture_apps[]}`.

## 6. Risks & Failure Modes

| Risk | Impact | Mitigation |
|---|---|---|
| New egress path added without a gate | Silent leak, AC-09 broken | Single-choke architecture + CI socket-audit (§7 in PRD terms: the harness below) |
| Indicator lags a fast call | "< 1% mis-belief" metric damage | Event-driven states (§4.2) |
| Injection crafts allowed-domain exfil | Data leaves via allowlisted host | Param entropy review + visible calling state; document residual risk (§4.6) |
| Wipe misses derived data (index/digest) | "Deleted" data recoverable | Wipe = whole data dir; index always derived |

### Verification harness (AC-09)

CI job: run scripted ask/session with kill-switch ON under a
**deny-all local socket proxy** (and OS-level firewall rule in the release
pipeline): any connection attempt fails the build. Repeat with OFF to assert
only allowlisted destinations connect. This converts AC-09 from a promise
into a test.

## 7. Open Questions

1. Allowlist default set — which curated domains ship enabled?
2. Reproducible builds: feasible with Tauri + Python sidecar packaging
   (worth the M3 effort or defer)?
3. Should `calling-cloud` indicator also record a local (private) log of
   egress events for user audit (privacy dashboard lite)?
4. Wipe vs export conflicts (export then wipe flow UX)?

## 8. Dependencies

- **RFC-0002** — indicator/tray UI, update-check gating, keychain use.
- **RFC-0003** — retention/quota mechanics, untrusted-content origin.
- **RFC-0004** — audio ephemerality mechanism (this RFC verifies it).
- **RFC-0007** — LLM gate at the adapter; offline answer presentation.
- **RFC-0008** — trust levels, registry gate, allowlist enforcement.

## 9. Acceptance Mapping

| AC | Mechanism |
|---|---|
| AC-09 | Single choke `NetPolicy` + registry trust gate + deny-all-proxy CI harness (§4.1, §6) |
| AC-10 | RFC-0004 mechanism + this RFC's verification harness (§4.4) |

## 10. Milestone Alignment

- **M0–M1:** `NetPolicy` seam + LLM/tool gates + indicator (enforcement
  lands with the features it guards).
- **M3:** full harness in CI, retention UX polish, injection-defense review,
  wipe/export, signing.
