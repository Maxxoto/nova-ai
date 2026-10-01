# RFC-0011 — Post-M2 Completion: Memory Surface & Input Bindings

- **Status:** Implemented (2026-10-01) — F-10/F-15/F-16 in `9da51ea`; **D6** (composite ranking, `c82e214`) and **D7** (nightly reflection, `13994b0`) in the same block. §7's eval ships as `cargo run --example memory_eval` (`f7596ce`), which caught and verified fixes for two real retrieval defects on its first run
- **Author:** Sisyphus
- **Parent:** [RFC-0001](RFC-0001-desktop-companion.md) (Desktop Companion)
- **Relates to:** [RFC-0006](RFC-0006-memory-system.md) (memory), [RFC-0007](RFC-0007-agent-brain.md) (loop), [RFC-0002](RFC-0002-platform-shell.md) (shell/hotkeys)
- **Research basis:** [docs/research/agent-memory-practices.md](../research/agent-memory-practices.md) — survey of Mem0/Letta/Zep/LangMem/Cognee/MemOS + the memory literature; amended decisions below cite it
- **Covers (PRD):** F-10 (save-to-memory wiring), F-15 (rebindable dismiss), F-16 (memory management) · **Amends:** AC-06
- **Milestone:** post-M2, next implementation block

---

## 1. Summary

Three pieces of unfinished product sit between the current build and the PRD's
MVP promise: the panel's **Save to memory** button is a placeholder, the
**dismiss hotkey** is hardcoded, and saved memories have **no management
surface**. This RFC pins the decisions to finish all three. It also records
the product call to defer VoxCPM ([RFC-0010](RFC-0010-voxcpm-tts-engine.md))
indefinitely.

## 2. Motivation

- J1 (the core loop in the PRD) ends with "hit Save → note lands in Semantic
  memory" — today that hit is dead. The store, FTS index and
  `memory_save_semantic` command all exist and are tested; only the wiring is
  missing.
- Esc's meaning changed (stop read-aloud; never dismiss) with ⌥⇧D as the
  dismiss key. A hardcoded key that conflicts with a user's muscle memory or
  another app has no escape hatch — every other hotkey is already rebindable.
- Trust for F-07/F-10 requires an answer to "what have you remembered about
  me, and how do I make you forget it?" Without a view/delete surface the
  memory feature is a one-way door — unacceptable for a privacy-first tool.

## 3. Scope

### In

1. **F-10 — Save to memory, wired.** The panel button stores the answer.
2. **F-15 — Dismiss hotkey, rebindable** (`dismiss_hotkey` setting, validated).
3. **F-16 — Memory management** (Settings → Memory: list, search, delete).
4. **AC-06 amendment** — Esc stops speech, the dismiss hotkey hides the panel.

### Out (with reasons)

- **VoxCPM TTS (RFC-0010): deferred** — product call, Oct 2026. Kokoro stays
  the on-device voice; the RFC returns only if the voice becomes the blocker.
- **Signing/notarization** — external prerequisite (Apple Developer ID); tracked
  as an ops task, not an RFC.
- **F-13 auto-capture, F-14 display picker, F-12 daily brief** — unchanged
  priority, separate work.

## 4. Decisions

### D1 — What a save stores (F-10)

`memory_save_semantic` (existing command) receives the **answer text plus its
provenance**: the ask's capture id(s) and the model/endpoint that produced it.
Rationale: F-07's citation guard already resolves `[cap_id]` refs at answer
time; persisting provenance keeps a saved note independently verifiable and
lets the management surface show "saved from capture ⌈thumb⌉ on <date>".

- Target: **semantic** store (the user's explicit "keep this" gesture), never
  episodic (that path is automatic and already exists).
- Dedupe: exact-text match against existing semantic notes returns the
  existing note and the UI says "Already saved" instead of writing a copy.
- Saved notes carry `pinned`: never auto-superseded by consolidation, and
  always eligible for the prompt's memory block (research: user-curated
  memory is authoritative everywhere — Letta core blocks, Mem0 layers).
- The raw answer is stored as-is; **distilled facts are extracted later by
  the nightly reflection pass** (D7), not at save time — keeps the save
  instant and off any LLM path.

### D2 — Save UX (F-10)

Press → optimistic "Saved ✓" chip state on the button (no panel navigation),
with a 4 s "Undo" affordance that deletes the just-written note. Rationale:
saving must not interrupt the read; undo covers the misclick without a
confirmation dialog (which would).

### D3 — Dismiss hotkey (F-15)

- Setting `dismiss_hotkey`, default `Alt+Shift+D`, validated by the existing
  `validate_hotkey` path and rejected on conflict with the capture hotkeys or
  PTT. Applies on next app start, same as PTT today.
- The global-shortcut registration gains the string alongside the capture
  accelerators; `is_dismiss_accelerator` compares normalized forms.
- Esc keeps its current two roles (cancel in-flight ask, stop read-aloud) and
  never dismisses — AC-06 is amended to match shipped behavior.

### D4 — Memory management surface (F-16)

Settings gains a **Memory** section backed by the existing `memory.search`
and new `memory_archive`/`memory_restore` commands. List view with search,
one row per semantic note: snippet, source (capture thumb or `ask`), age,
archive (single click + undo toast; typed-confirm for multi-select).
Archived notes leave retrieval but stay restorable — **no hard delete**.
Rationale: the field consensus is invalidate-don't-delete (Zep's bi-temporal
edges, Mem0 Supersede, soft-archive in Letta/Cognee/MemOS), and the STALE
benchmark shows stale-belief handling is the dominant failure mode; history
and undo are worth more than disk. User-pinned notes are exempt from any
batch curation.

### D5 — Brain involvement

None for F-15 (pure shell). F-10/F-16 touch the brain only through the
existing JSON-RPC commands — no loop changes, no new tool steps.

### D6 — Retrieval ranking (memory read path, from the research)

Memory candidates re-rank by composite score —
`w_rec·recency + w_imp·importance + w_fts·BM25` — with recency as an
exponential decay whose clock **resets on access** (`last_accessed_at`
bumped asynchronously), importance defaulted by kind (pinned > reflection >
episode), and relative dates in the query ("last week") turned into
time-aware filters. Injected memory is a structured block (id, date, text)
with an abstention instruction. Evidence: Generative Agents' ablation and
LongMemEval's controlled findings (+4–11% per mechanism; +10 points from
the reading stage). Embeddings stay **out** until a personal eval shows
paraphrase-recall misses — BM25 is competitive at personal-corpus scale.

### D7 — Nightly reflection pass

A scheduled consolidation (off the interactive path): distil the day's
episodes and saved answers into durable `fact`/`reflection` notes, build
links, assign importance, and propose supersessions for contradictions
(applied to non-pinned notes only). Evidence: Generative Agents' reflection
ablation; identical scheduling in Mem0 Dream / Letta sleep-time / LangMem /
Cognee. This is where the LLM earns its cost — never on the write path.

## 5. Test plan

- Rust: hotkey validation table (conflicts, normalisation); settings
  round-trip includes `dismiss_hotkey`.
- Python: save dedupe; provenance fields present; delete removes from FTS and
  files; a saved note is retrievable via `memory.search` (AC-08).
- UI: button states (idle → saving → saved/already-saved → undo countdown);
  Memory section lists and deletes.
- AC-06 (amended): Esc during read-aloud stops audio with the panel visible;
  the dismiss hotkey hides the panel; neither gesture does the other's job.

## 6. Open questions

1. Should episodic notes (auto-logged asks) also be archivable from F-16, or
   only semantic? (Leaning: both, separate group in the list.)
2. Cap on semantic notes before suggesting cleanup? (Leaning: no cap, surface
   a count only.)
3. Reflection cadence: nightly vs Generative-Agents-style importance
   threshold. (Leaning: nightly to start — simpler, debuggable.)

## 7. Evaluation

Per the research digest: a personal eval set (20–50 questions) covering
LongMemEval's five abilities plus STALE-style implicit conflicts, measuring
retrieval recall@k separately from answer accuracy — not vendor
leaderboards, whose methodology is documented as noisy.
