# Agent Memory: Research Digest & Practices for Ruoxi

- **Status:** Research input to [RFC-0011](../rfc/RFC-0011-memory-surface-and-input.md) · compiled 2026-10-01
- **Question:** What should Ruoxi's memory layer adopt, based on how the top agent-memory libraries are built and what the research literature actually supports?

Sources surveyed: Mem0, Letta (MemGPT), Zep/Graphiti, LangMem, Cognee, MemOS, LlamaIndex memory, plus the paper line MemGPT → Generative Agents → Reflexion/Voyager → HippoRAG → Zep → Mem0/A-Mem/MemOS, LongMemEval/LoCoMo/MemoryAgentBench/STALE benchmarks and 2025-26 surveys. (Full citations inline below.)

---

## 1. What the leaders do (convergent architecture)

Every serious system, despite different marketing, converges on the same skeleton:

| Concern | Consensus across Mem0 / Letta / Zep / LangMem / Cognee / MemOS |
|---|---|
| Tiers | Small **always-in-context core/profile** + large **searchable external store** (Letta core blocks; Zep user summary; LlamaIndex static blocks; MemGPT lineage) |
| Writes | **LLM-distilled facts, not raw transcripts** (Mem0 `infer`, LangMem extraction, Zep facts, LlamaIndex fact blocks) — extraction happens **off the interactive path** |
| Maintenance | **Scheduled consolidation**: cluster/dedupe/merge/promote (Mem0 Dream, Letta sleep-time "dreaming", LangMem ReflectionExecutor, Cognee `improve`) |
| Retrieval | **Multi-signal hybrid**: lexical + semantic + entity + temporal, fused (Mem0 4-signal, Zep semantic+BM25+BFS, Unforget 4-channel RRF) |
| Ranking | Composite **relevance × recency × importance** (Generative Agents lineage; LangMem "similarity + importance + strength"; ScalyClaw 0.6/0.2/0.2) |
| Change over time | **Invalidate, don't delete**: Zep's bi-temporal edges (`valid_at/invalid_at`), Mem0 Dream Supersede, soft-archive + restore everywhere |
| Trust | **Provenance on every item** (source, timestamps, user-saved vs inferred) + a **user-visible management surface** (Letta memory viewer, Zep RTBF delete, Cognee `forget` scopes, MemOS viewer) |
| Local-first | Proven viable: MemOS local plugin = SQLite + FTS5 + vectors fully on-device; Cognee ships SQLite+LanceDB+embedded graph; Graphiti runs on embedded Kuzu. Ruoxi's SQLite+FTS shape is a validated pattern, not a compromise. |

## 2. What the research actually supports (evidence tiers)

**Tier 1 — strong, controlled evidence**
- **Composite recency+importance+relevance ranking** — Generative Agents ablation, Cohen's d = 8.16; adopted by every production system. (arXiv:2304.03442)
- **Retrieval mechanics** — LongMemEval's controlled findings: fine-grained session/round decomposition **+11.4% recall**; fact-augmented index keys **+4–9% recall / +5% QA**; time-aware query expansion **+7–11% temporal recall**; a structured reading stage (JSON block + chain-of-note + abstention) **+10 absolute points even at perfect recall**. (arXiv:2410.10813, ICLR 2025)
- **Write-time consolidation** (decide add/update/supersede instead of append-only) — Mem0's ADD/UPDATE/DELETE/NOOP; LongMemEval treats knowledge-update as its own hard category. (arXiv:2504.19413)
- **Two-tier context** — MemGPT (93.4% vs 35.3% DMR, though the baseline was weak; the architecture is universally adopted). (arXiv:2310.08560)
- **Reflection into higher-level memories** — Generative Agents ablation; Reflexion HumanEval 91% vs 80%. (2304.03442, 2303.11366)

**Tier 2 — good evidence, narrower**
- **Temporal invalidation** — the STALE benchmark (arXiv:2605.06527): best frontier model only **55.2%** on implicit-conflict scenarios; the dominant failure is acting on stale info, not forgetting. Cheap to implement (`valid_to`/`superseded_by`), high value.
- **Procedural memory as verified skill libraries** — Voyager (3.3×/15.3× task gains), gated on self-verification. (arXiv:2305.16291)

**Tier 3 — plausible but overkill/unvalidated for us**
- HippoRAG graph + Personalized PageRank (+20% multi-hop QA) — needs LLM KG construction + embeddings; benefit confined to multi-hop document QA. (2405.14831)
- A-Mem Zettelkasten per-write note evolution — expensive per write, drift risk, evals inconsistent. (2502.12110)
- MemOS MemCube / parameter & activation memory — needs model internals. (2507.03724)
- Full bi-temporal event graph with community summaries (Zep) — keep the columns, skip the graph machinery.
- **Embeddings for a small personal corpus** — practical reports agree BM25/FTS5 is competitive at <500 notes (exact names, codes, keywords); simple hybrid beats complex pipelines; MMR and time-decay add nothing measurable at this scale. Add a local embedding model only when paraphrase recall demonstrably hurts.

**Benchmark skepticism** (why we don't chase leaderboards): LoCoMo's answer key has **6.4% errors**; its LLM judge accepts **~63% of intentionally wrong answers**; ~56% of per-category comparisons are statistical noise (Penfield Labs audit, 2026); Mem0↔Zep numbers are mutually disputed; MemGPT's headline gap was a weak-baseline artifact. **Build a personal eval instead** (below).

## 3. The minimal set Ruoxi should adopt

1. **One `memories` table**: `id, text, kind (episode|fact|reflection), created_at, last_accessed_at, access_count, importance, source/provenance, capture_ref, valid_from, valid_to, superseded_by, pinned` + FTS over text/keywords. (Everything else is columns, not infrastructure.)
2. **Write path**: episodes auto-log; **user saves are `pinned` and never auto-superseded**; cheap FTS near-dup check at write (no per-write LLM); contradictions → set `valid_to` + `superseded_by`, keep both rows.
3. **Read path**: FTS candidates → composite rerank `w_rec·recency (half-life, reset on access) + w_imp·importance (by kind: user-saved > reflection > episode) + w_fts·BM25`; time-aware filters from relative dates; emit a **structured JSON memory block with dates + IDs + abstention instruction**.
4. **Nightly reflection** (the LLM's real job, off the interactive path): distill the day's episodes into facts/reflections, build links, compute importance, propose supersessions.
5. **Management (F-16)**: list/search/delete — delete is **archive-first with restore**, mirroring the field consensus; user-saved notes are protected from batch curation.
6. **Personal eval**: 20–50 questions spanning LongMemEval's five abilities + STALE-style implicit conflicts; measure retrieval recall@k separately from answer accuracy.

## 4. Key citations

- Generative Agents — arxiv.org/abs/2304.03442 · MemGPT — /2304.03442 → /2310.08560 · Reflexion — /2303.11366 · Voyager — /2305.16291 · HippoRAG — /2405.14831 (+2: /2502.14802) · Zep — /2501.13956 · Mem0 — /2504.19413 · A-Mem — /2502.12110 · MemOS — /2507.03724 · LongMemEval — /2410.10813 · LoCoMo — /2402.17753 · MemoryAgentBench — /2507.05257 · STALE — /2605.06527 · Surveys — /2512.13564, /2603.07670
- Libraries: docs.mem0.ai · docs.letta.com · help.getzep.com (Graphiti) · langchain-ai.github.io/langmem · docs.cognee.ai · github.com/MemTensor/MemOS · developers.llamaindex.ai
- Practice write-ups: agent-memory.bruegs.com (flat-file + hybrid, small-corpus findings) · docs.unforget.sh (4-channel fusion, type boosts) · ScalyClaw memory docs (composite scoring, consolidation, dedup thresholds)
