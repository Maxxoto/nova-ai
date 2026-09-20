# RFC-0006 — Memory System (Three Memories, Local & Human-Readable) ⭐

- **Status:** Draft for review
- **Author:** Dani
- **Parent:** [RFC-0001](RFC-0001-desktop-companion.md) (Desktop Companion)
- **Companion:** [Business PRD](../ruoxi_prd.md)
- **Covers (PRD):** F-07 ⭐ (memory-grounded answers), F-10 (save-to-memory), F-12 (daily brief)
- **ACs:** AC-07, AC-08 · **Milestones:** M2 ("It remembers" ⭐), F-12 in M3

---

## 1. Summary

The product's differentiator: three CoALA-style memory types — **episodic**
(what happened), **semantic** (what's true), **procedural** (how you work) —
stored as human-readable Markdown files with a rebuildable SQLite index,
evolving the repo's existing two-layer system (MEMORY.md + HISTORY.md). The
North Star metric (Memory-Grounded Assists/day, ≥ 30% of answers using
memory) lives or dies here, with a false-grounding guard keeping wrong-memory
citations near zero.

## 2. Motivation

"The moat: the agentic loop over *your own* local memories. The more you use
it, the more it knows you. 越用越懂你。" Memory is also the trust hot-spot:
it must feel like a notebook you own (files on disk), never surveillance
(PRD §12). The repo already ships a two-layer memory — this RFC is an
**evolution**, not a green-field design.

## 3. Goals / Non-Goals

**Goals**
- Three memory types per PRD decision #3, as plain Markdown + front-matter.
- Explicit Save (F-10) as the primary semantic write path; episodic is
  automatic; procedural writes require confirmation.
- Hybrid retrieval (keyword + optional local embeddings) with citations that
  link back to captures (J3: "answer with the original screenshot cited").
- Background consolidation: episodic → semantic candidates; daily digest.
- Fully local: retrieval works offline; no memory content leaves the device
  except as context inside the live reasoning request (RFC-0009).

**Non-Goals**
- Cloud sync (PRD non-goal), multi-user, encryption-at-rest beyond OS
  facilities in v0.
- Vector DB servers, external services — embedded stores only.
- Silent learning from screen content (poisoning defense, §4.9).

## 4. Detailed Design

### 4.1 From two layers to three types (reconciliation)

The current brain (see `src/app/infrastructure/skills/builtin/memory/SKILL.md`)
uses MEMORY.md (always-loaded facts) + HISTORY.md (searchable archive). Mapping:

| CoALA type | Today | Becomes |
|---|---|---|
| Episodic | HISTORY.md append-only log | Sharded event entries w/ `capture_id` refs, tool traces, Q/A pairs |
| Semantic | MEMORY.md (single file, sections) | Per-note Markdown files, tagged, citable |
| Procedural | (implicit in MEMORY.md prose) | First-class: preferences/corrections/patterns with a confirm loop |

`MEMORY.md` remains as a **compiled context digest** (generated, not
hand-edited) — keeping the existing brain's always-load path intact while
real storage moves to the sharded layout.

### 4.2 Storage layout

```
<data_dir>/memory/
  episodic/2026/09/18/e_20260918T1422.md     # one entry per interaction
  semantic/mitochondria-notes.md             # user-facing notes/facts
  procedural/how-i-study.md                  # preferences, corrections
  digest/MEMORY.md                           # compiled always-load digest
  index.sqlite                               # derived; rebuildable
```

Front-matter schema (all types):

```yaml
id: mem_01H...            # ULID
type: episodic|semantic|procedural
created: 2026-09-18T14:22:33+07:00
source_refs: [cap_01H...]  # captures / session ids (episodic esp.)
tags: [study, biology]
confidence: 0.9            # set at write; consolidation may adjust
origin: user-save|auto|consolidation|confirm
```

**Why Markdown + SQLite index (not a DB):** RFC-0001 goal "human-readable
files" — the user can read, grep, edit, and export their memory with any
tool; the index is a cache (FTS + optional vectors) rebuildable by scanning
front-matter. Files are the source of truth.

### 4.3 Write paths

| Path | Trigger | Type | Confirmation |
|---|---|---|---|
| Auto-log | Every ask/answer completes | episodic | none (it's a diary, includes capture refs) |
| **Save** (F-10) | User hits Save on an answer | semantic | explicit user act |
| "Remember that I…" | User says so | procedural | explicit user act |
| Inferred candidate | Consolidation proposes | semantic/procedural | **required** — surfaces as a tray suggestion the user accepts/ignores |
| Citation follow-up | "what did I save about X" | — | read-only |

LLM-assisted extraction (title/tags/summary on Save) is **cloud-gated**:
when offline, Save stores raw text with auto-tags only. `episodic` is never
tool-writable (RFC-0008) — it is written by the session recorder, not the
agent, to prevent manipulation.

### 4.4 Retrieval

- **Stage 1 — keyword:** SQLite FTS over note bodies + tag/app/title metadata
  (fast, exact, offline).
- **Stage 2 — semantic (optional, on by default when available):** local
  embedding of memory items into sqlite-vec-class storage; embedding model
  runs **locally in the Python brain** (small multilingual model; no API).
- **Merge/rerank:** score = `0.6·lexical + 0.4·vector` (tunable), then
  recency boost for episodic, confidence filter for semantic; top-k (k=5)
  with type diversification (don't return 5 episodic hits for a fact
  question).
- Interface: `memory_search(query, types?, k?, since?)` → ranked hits with
  `id, type, snippet, score, source_refs` (RFC-0008 tool).
- Target: p50 ≤ 100 ms warm (to validate; must not eat the latency budget).

### 4.5 Grounding & citations (AC-07)

- Retrieved items are injected into context as a numbered **Memory block**
  with explicit instructions: answers must cite used memories as `[mem_id]`,
  and captures as `[cap_id]`.
- The panel renders `[cap_id]` citations as clickable thumbnails opening the
  timeline viewer (J3's "original screenshot cited"; RFC-0003 §4.4).
- **False-grounding guard (counter-metric):** before final render, a
  post-check verifies every `[mem_id]`/`[cap_id]` in the answer (a) exists,
  (b) was in the injected block, and (c) textually overlaps the claimed
  claim (cheap n-gram check). Violations → citation stripped + flagged in
  the session log (measurement input for the counter-metric, no user
  friction in v0).

### 4.6 Consolidation & daily brief (F-12)

- Nightly (or on first launch of the day) job over yesterday's episodic
  entries: clusters → drafts semantic "day notes" + procedural candidates →
  queues them as confirm-suggestions (§4.3). Cloud-gated (uses the LLM);
  offline days simply skip.
- **Daily brief (F-12):** tray-click surfaces a one-liner generated from the
  digest of yesterday + open todos/notes. Cached; generated at most once per
  day; cloud-gated.

### 4.7 Lifecycle & capacity

- No hard quota on memory (it's small text); episodic entries older than N
  months (default 12) are candidates for archival compaction (summarized by
  consolidation, originals folded into an archive folder the user can purge).
- Export = copy the `memory/` dir (it's files). Import = drop-in + rebuild
  index. "Forget everything" wipes the dir + index.

### 4.8 Memory under offline mode

- Retrieval (keyword always; vector if model is local) works offline —
  memory-grounded answers remain available when the LLM is unreachable only
  if the configured endpoint is local; with a cloud endpoint, offline mode
  shows the memory hits themselves (source list) without a generated answer
  (RFC-0007 §4g decides presentation).
- Consolidation, brief, and LLM extraction disabled offline (zero network).

### 4.9 Poisoning defense (untrusted screen content)

Captures/transcripts are untrusted (RFC-0009). Rules:

1. No semantic/procedural write may originate from capture content without
   explicit user Save/confirm (origin field enforces provenance).
2. Episodic auto-logs may reference captures but are never injected as
   instructions — only as retrieval data.
3. Consolidation candidates from episodic text derived from screens are
   marked `origin: consolidation` and still require confirmation.

This blocks the "malicious webpage writes into your long-term memory"
attack path.

## 5. Interfaces & Data Structures

Examples:

```markdown
---
id: mem_01HXY...; type: episodic; created: 2026-09-18T14:22:33+07:00
source_refs: [cap_01HA...]; tags: [study, pdf]; origin: auto
---
User boxed a paragraph in "lecture-7.pdf" and asked "explain simply".
Answer (a83f): Krebs cycle summary… [cites cap_01HA…]
```

```markdown
---
id: mem_01HXZ...; type: semantic; created: 2026-09-18T14:25:01+07:00
source_refs: [a83f]; tags: [biology]; confidence: 0.9; origin: user-save
---
# Krebs cycle — simple version
Turns pyruvate into energy carriers… (2 saved sentences from answer a83f)
```

```markdown
---
id: mem_01HY0...; type: procedural; created: 2026-09-19T09:00:12+07:00
tags: [study-flow]; confidence: 0.8; origin: confirm
---
# How Dani studies
Prefers "explain simply" first, then depth on request. Studies bilingually;
answers may mix English and Chinese.
```

API: `memory_search(query, types?, k?)`, `memory_save(type, content,
source_ref?)`, `memory_confirm(id)`, `timeline.query(...)` (RFC-0003).

## 6. Risks & Failure Modes

| Risk | Impact | Mitigation |
|---|---|---|
| False grounding (wrong memory cited) | North-star trust collapse; counter-metric | Post-check guard (§4.5); measure rate per release |
| Memory feels like surveillance | PRD §12 risk | Explicit Save default; episodic is a diary the user can purge; visible indicators |
| Index drift vs files (user edits by hand) | Stale retrieval | File-watcher + rebuild-on-startup; index is derived |
| Embedding model RAM in brain | Envelope pressure (RFC-0002) | Small model, lazy-load with STT-style policy; keyword-only fallback |
| Retrieval latency blows budget | p50 ≤ 2s at risk | Warm cache, k≤5, FTS-first |
| Unbounded episodic growth | Disk + context noise | 12-month compaction (§4.7) |

## 7. Open Questions

1. Which local multilingual embedding model fits the RAM/latency envelope
   (spike; sqlite-vec-class store assumed)?
2. Merge weights (0.6/0.4) and k — tune on real study-session data in M2?
3. Digest generation: full rebuild nightly vs incremental on write?
4. Should confirm-suggestions expire (e.g., 7 days) to avoid nagging?
5. Do procedural entries need per-domain scoping (study vs work) in v0?

## 8. Dependencies

- **RFC-0003** — capture store provides `capture_id` refs + citation viewer.
- **RFC-0007** — context assembly consumes the Memory block; citation
  post-check hooks the answer path; daily-brief generation.
- **RFC-0008** — `memory.search`/`memory.save` tool schemas; episodic
  write-path restriction.
- **RFC-0009** — poisoning rules, offline gating, export/wipe UX.

## 9. Acceptance Mapping

| AC | Mechanism |
|---|---|
| AC-07 | Memory block injection + mandatory `[mem_id]`/`[cap_id]` citations + post-check (§4.5) |
| AC-08 | Save writes semantic note; `memory_search` retrieves it; timeline shows it via `source_refs` (§4.3/4.4) |

## 10. Milestone Alignment

- **M2 ⭐:** three-type store, hybrid retrieval, grounding + citations, false-
  grounding guard, Save (F-10).
- **M3:** daily brief (F-12), consolidation confirm-loop polish.
