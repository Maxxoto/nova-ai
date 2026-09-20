# RFC-0007 — Agent Brain: Loop, Context Assembly & LLM Routing

- **Status:** Draft for review
- **Author:** Dani
- **Parent:** [RFC-0001](RFC-0001-desktop-companion.md) (Desktop Companion)
- **Companion:** [Business PRD](../ruoxi_prd.md)
- **Covers (PRD):** F-07 (engine side), F-12 (daily brief generation)
- **ACs:** AC-02 (latency), AC-07 (grounding), AC-11 (≤ 3 tool steps, Esc abort) · **Milestones:** M1 (answers), M2 (memory grounding), M3 (brief)

---

## 1. Summary

How the existing pure-Python agent loop (`src/app/application/services/
agent_loop.py`: while-loop tool calling, `ToolRegistry`, LiteLLM adapter,
JSON session persistence) evolves into the desktop companion's brain: a hard
≤ 3-tool-step budget with Esc abort, ordered context assembly over the three
memories, multimodal capture handling, single-endpoint LLM routing, token
streaming over IPC — and ownership of the **latency budget** (successor of
RFC-0001's missing §12).

## 2. Motivation

The brain is already proven in CLI form. The desktop context changes its
contract: answers must feel instant (p50 ≤ 2s), be abortable mid-loop (Esc,
AC-11), cite memory (AC-07), and accept images — all over a supervised
sidecar boundary (RFC-0002 §4.7) with offline hard-gating (RFC-0009).

## 3. Goals / Non-Goals

**Goals**
- Evolve, not rewrite: same loop/registry/LiteLLM seams, new contracts.
- Hard budget: ≤ 3 tool steps per response; Esc aborts cleanly (AC-11).
- Context assembly with explicit layer ordering + token budget.
- Vision: capture images as first-class message content.
- Streaming tokens → panel + TTS.
- Own and publish the latency budget table.

**Non-Goals**
- Multi-agent orchestration, planning graphs (PRD WON'T: multi-tool loops).
- Model self-selection beyond the vision/no-vision rule.
- On-device inference (PRD non-goal).

## 4. Detailed Design

### 4.1 The loop, evolved

Today: `while iteration < max_iterations` → LLM call → tool calls → append
results → repeat. Changes:

- **Budget:** `max_iterations = 3` **hard** (AC-11). Budget is enforced
  before each LLM call; when a 4th tool call is requested, the loop instead
  forces a final answer turn with a "wrap up" instruction. Common journeys
  expect 0–2 steps (J1: 0–1; J3: 1 memory.search).
- **Abort:** `session.abort` (Esc) sets a cancellation token checked (a)
  before each LLM call, (b) between tool executions, and forwarded into the
  in-flight LiteLLM request's cancellation support. Completed steps' effects
  stand (memory writes from tools already executed are NOT rolled back —
  they're user-visible actions); no new steps start; panel returns to idle.
- **Streaming:** LiteLLM stream mode → per-token `agent.token` notifications
  over IPC; first-token timestamp feeds the latency budget (§4.6).

### 4.2 Context assembly (per request)

Ordered layers, each with a soft token cap (eviction order = drop from the
bottom of this list first):

| # | Layer | Source | Soft cap (tokens) |
|---|---|---|---|
| 1 | System: persona + rules | SOUL + fixed instructions (bootstrap skill) | ~400 |
| 2 | Procedural digest | RFC-0006 compiled digest (MEMORY.md successor) | ~400 |
| 3 | Retrieved memory block | RFC-0006 `memory_search` hits, numbered for citation | ~800 |
| 4 | Episodic window | last N turns of this session | ~1200 |
| 5 | Current turn | transcript + capture refs + mode flags | rest |

Caps are defaults, tunable; overflow triggers layer-specific trimming
(summarize episodic, drop lowest-scored memory hits). System + procedural
are never evicted.

### 4.3 Multimodal: captures in messages

- Each active `capture_id` becomes an image content block in the final user
  message (OpenAI-compatible multimodal format via LiteLLM), using the
  downscaled request payload from RFC-0003 §4.5 (one image per request in
  v0; further captures reachable via the `capture.lookup` tool).
- Metadata (app, window_title, ts, scope) rides along as text, clearly
  delimited as **data, not instructions** (injection defense, RFC-0009).

### 4.4 LLM routing

- Single configured OpenAI-compatible endpoint via the LiteLLM adapter
  (100+ providers); user-set key/URL in settings; no built-in provider
  rotation in v0.
- **Model rule:** captures present → vision-capable model; text-only →
  default text model (can be the same model if configured so). Both model
  ids are settings, not code.
- Daily brief + consolidation use the text model (RFC-0006 §4.6).

### 4.5 Latency budget (owns former §12)

p50 intent→answer ≤ 2s (PRD §9); p95 guarded (target ≤ 4s, to validate).

| Segment | Owner | p50 target |
|---|---|---|
| key-up → transcript delivered | RFC-0004 | ≤ 800 ms |
| context assembly + retrieval | RFC-0006 | ≤ 150 ms |
| first LLM token (network) | this RFC | ≤ 600 ms |
| tool steps allowance (≤ 3) | this RFC + 0008 | ≤ 600 ms |
| final token → panel complete | RFC-0002 | ≤ 50 ms |

All numbers are **targets to validate in the M1 spike** on reference
hardware/network; the table is updated with measurements, not guesses.

### 4.6 Streaming transport

- Tokens flow brain → shell as `agent.token` notifications (stdio JSON-RPC,
  RFC-0002 §4.6). TTS sentence-chunking subscribes to the same stream
  (RFC-0005 §4.3).
- Progress affordance: `agent.tool_step {n_of_3, tool}` lets the panel show
  "searching memory (2/3)…" — supports AC-11's visibility.

### 4.7 Offline behavior (AC-09)

Offline flag ON → **no LLM calls** (hard gate at the LiteLLM adapter
boundary — RFC-0009 choke point). Fallback behavior:

- Voice/text questions still run retrieval; the panel shows **memory hits
  directly** (source list with capture thumbnails) instead of a generated
  answer, labeled "offline — from your memory".
- Rationale: a refused assistant feels dead; showing the user their own
  remembered material is honest, useful, and reinforces the local-memory
  moat. Cloud-only questions get an explicit "needs cloud — retry when
  online" state. This choice is flagged for PRD sign-off.

### 4.8 Daily brief (F-12)

- Trigger: first tray click after a new day (rate-limited to 1/day).
- Input: yesterday's consolidated digest + open todos/notes; output: one
  line + three bullets max. Cloud-gated; cached for the day; failure =
  silent skip (tray stays useful).

### 4.9 Sessions

- Keep JSON-file sessions (existing pattern); add `capture_ids`,
  `memory_refs`, latency samples per turn (budget telemetry), and abort
  markers. Sessions feed episodic memory writes (RFC-0006).

## 5. Interfaces & Data Structures

```json
// session.ask (shell → brain)
{ "transcript": "explain this chart", "capture_ids": ["cap_01H…"],
  "mode": "ask" }
// messages: system+procedural → memory block → episodic window →
// user turn with image blocks (see §4.2/4.3)
```

- `BrainConfig {endpoint, api_key_ref, vision_model, text_model,
  max_tool_steps: 3, layer_caps…}`.

## 6. Risks & Failure Modes

| Risk | Impact | Mitigation |
|---|---|---|
| Network jitter dominates p50 | Budget blown on bad WiFi | p95 guard + panel shows progressive state; budget table re-measured |
| Model ignores citation instructions | AC-07 misses | Post-check strips invalid citations (RFC-0006 §4.5) |
| Abort races a completing tool | Ambiguous UI state | Cancellation token + single-writer session state; completed-step report |
| Context bloat with big memory digest | Latency + cost | Layer caps + digest compilation (RFC-0006) |
| Vision model cost per capture | User cost anxiety | Downscale policy (RFC-0003), one-image rule, cost note in settings |

## 7. Open Questions

1. Offline fallback presentation (§4.7) — PRD sign-off on "memory hits
   instead of refusal"?
2. Should `max_tool_steps` be user-tunable above 3 (AC-11 says 3 — proposal:
   no in v0)?
3. Streaming + tool calls interplay across LiteLLM providers — uniform
   enough in practice (spike in M1)?
4. Layer caps correctness — validate token accounting with real zh/en mixed
   content (tokenizer variance)?

## 8. Dependencies

- **RFC-0002** — IPC transport, `session.abort` wiring, panel streaming.
- **RFC-0003** — capture payload + metadata contract.
- **RFC-0006** — memory block format, citation post-check, digest, brief.
- **RFC-0008** — tool catalog the loop executes; step-budget interplay.
- **RFC-0009** — offline gate at the adapter; egress allowlist.

## 9. Acceptance Mapping

| AC | Mechanism |
|---|---|
| AC-02 | Latency budget table (§4.5) + streaming path; measured at M1 |
| AC-07 | Memory block + citation rules + post-check (with RFC-0006) |
| AC-11 | Hard 3-step budget, forced wrap-up, abort token, tool_step visibility (§4.1) |

## 10. Milestone Alignment

- **M1:** loop budgets + abort + streaming + vision messages; budget table
  instrumented.
- **M2:** memory block + citations end-to-end.
- **M3:** daily brief; offline memory-hits presentation polish.
