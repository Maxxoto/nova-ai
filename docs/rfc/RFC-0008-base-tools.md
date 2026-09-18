# RFC-0008 — Base Tool Catalog & Registry (v0)

- **Status:** Draft for review
- **Author:** Dani
- **Parent:** [RFC-0001](RFC-0001-desktop-companion.md) (Desktop Companion)
- **Companion:** [Business PRD](../ruoxi_prd.md)
- **Covers (PRD):** F-07 (tooling substrate), F-09 (tool-side cloud gating)
- **ACs:** AC-07, AC-09 (tool-side), AC-11 · **Milestones:** M1 (core set), M2 (memory tools)

---

## 1. Summary

The v0 base tool set the desktop agent may call, grounded in the tools that
exist in the repo today (`src/app/infrastructure/tools/`: ReadFile, WriteFile,
EditFile, ListDir, Exec, WebSearch, WebFetch + skills: memory, notes, todo,
web, github, weather), re-cut for the desktop trust model: a small catalog,
explicit trust levels, network tools hard-gated by the offline kill-switch,
and **no shell/general-filesystem tools** in v0.

## 2. Motivation

Every tool is both a capability and an attack surface. On the desktop, the
agent handles untrusted screen content (RFC-0009), so the catalog must be
minimal, typed, and trust-labeled — the CLI-era freedom (arbitrary file
writes, shell exec) is exactly what v0 must not carry over.

## 3. Goals / Non-Goals

**Goals**
- Complete v0 catalog with schemas, trust levels, PRD traceability.
- KEEP/ADAPT/DROP verdicts for every existing tool.
- Registry mechanics evolving the current `Tool` ABC + Pydantic → OpenAI
  schema generation.
- Structured error contract the LLM can recover from; result-size policy.

**Non-Goals**
- Community tool marketplace (extension path sketched only).
- Machine-control tools (PRD WON'T in v0).
- Dynamically loaded plugins.

## 4. Detailed Design

### 4.1 Inventory verdicts (existing code → v0)

| Existing (repo) | Verdict | Rationale |
|---|---|---|
| `ReadFile/WriteFile/EditFile/ListDir` (filesystem.py) | **DROP for v0** | General FS access on a screen-companion = exfiltration/破坏 risk; workspace-scoped notes tools replace the legitimate need |
| `Exec` (shell.py) | **DROP for v0** | Shell exec behind untrusted screen content is unacceptable; PRD says no machine control in v0 |
| `WebFetch/WebSearch` (web.py) | **ADAPT** | Keep as cloud-gated, domain-normalized fetch for study journeys |
| memory skill (MEMORY.md/HISTORY.md) | **ADAPT** | Becomes `memory.search`/`memory.save` per RFC-0006 |
| notes skill (workspace/notes/) | **ADAPT** | Becomes workspace-scoped `notes.*` tools |
| todo skill | **KEEP** | Local, harmless, feeds daily brief |
| github skill | **DEFER** (post-v0) | Power-user value, cloud + credential surface; not in PRD F-xx list |
| weather skill | **DROP for v0** | Cloud toy; not on any PRD journey; re-addable later |
| cron tool (cron.py) | **DROP for v0** | Background autonomy out of scope for v0 |

### 4.2 v0 base catalog

| Tool | Purpose | Key params | Returns | Trust | PRD |
|---|---|---|---|---|---|
| `capture.take` | Take a new capture | `scope: region\|window\|fullscreen` | `capture_id` + metadata | local-read (Rust-routed, RFC-0003) | F-03/04/05 |
| `capture.lookup` | Fetch stored capture metadata/thumbnail ref | `id` | metadata + path | local-read | F-08 |
| `memory.search` | Hybrid retrieval over memories | `query`, `types?`, `k?` | ranked hits w/ ids | local-read | F-07 ⭐ |
| `memory.save` | Save an answer/fact | `type: semantic\|procedural`, `content`, `source_ref?` | written id | local-write **sensitive** | F-10 |
| `notes.append` / `notes.read` / `notes.list` | Workspace notes CRUD-lite | `title/path`, `content` | result | local-write (workspace-scoped) | — |
| `todo.list` / `todo.add` / `todo.done` | Simple todos | `text`, `id` | result | local-write | F-12 support |
| `web.fetch` | Fetch a URL as clean text | `url` | truncated text | **network** | J-study support |
| `web.search` | Web search | `query` | top results | **network** | J-study support |

Explicit **absences** (security rationale): no general filesystem, no
shell/exec, no OS input synthesis, no email/messaging, no arbitrary HTTP
method tool (fetch is GET-only, text-normalized).

**Episodic is not tool-writable** (RFC-0006 §4.3): the session recorder
writes it; `memory.save` rejects `type: episodic`.

### 4.3 Trust / permission model

| Level | Meaning | Enforcement |
|---|---|---|
| `local-read` | Reads local stores only | no gating |
| `local-write` | Writes app-scoped data | allowed in-loop; `sensitive` variant (`memory.save`) surfaces in panel as "saved to memory" (user-visible) |
| `network` | Leaves the device | **hard-fail when offline** (AC-09); cloud indicator ON during call (RFC-0009); blocked for prompt-injected origins — see RFC-0009 §4f |

Offline kill-switch enforcement point: the registry's execute path checks
trust level vs the shared offline flag before dispatch — a single seam,
testable in CI (with RFC-0009's socket-audit harness).

### 4.4 Registry mechanics

- Keep the current pattern: `Tool` ABC with `name`, `description`,
  Pydantic `param_model` → OpenAI function schema via `to_schema()`;
  `ToolRegistry.get_definitions()/execute()`.
- Add: `trust: TrustLevel` property on `Tool`; registry enforces §4.3.
- Schema versioning: catalog ships a `catalog_version`; sessions record it;
  cross-version replay degrades gracefully (unknown tool → structured
  error, not crash).

### 4.5 Result-size policy

- Text results truncated to ~4 k tokens per tool call (configurable) with a
  `[truncated]` marker.
- Captures are referenced by `capture_id`, never inlined twice (the request
  image is attached once — RFC-0007 §4.3).

### 4.6 Budget interplay (AC-11)

Expected step counts: J1 explain → 0–1 (`capture.lookup` optional); J3
memory recall → 1 (`memory.search`); offline study w/ source check → 2
(`memory.search`, `web.fetch` gated). The 3-step ceiling (RFC-0007 §4.1)
accommodates all v0 journeys with margin; the panel's `tool_step {n_of_3}`
visibility keeps users oriented.

### 4.7 Error contract

Every tool returns a string that is either a JSON success envelope or a JSON
error the LLM can act on:

```json
{"ok": false, "error": "offline_network_blocked",
 "hint": "network tools are disabled while offline"}
```

No exceptions escape the registry; timeouts (default 10 s web, 2 s local)
produce the same shape.

### 4.8 Extension path (post-v0 sketch)

Community tools (MIT, OSS): declarative manifest `{name, schema, trust,
binary|prompt-only}`; `prompt-only` tools (no code) can ship in the catalog
safely; code tools require review + signing story (open question).

## 5. Interfaces & Data Structures

Defined by §4.2 tables; consolidated machine-readable catalog
(`tools/catalog.json`) generated from the registry at build time and
versioned.

## 6. Risks & Failure Modes

| Risk | Impact | Mitigation |
|---|---|---|
| Prompt injection drives `web.fetch` exfil | Data leaves device via URL params | Fetch is GET/text-only; egress domain allowlist (RFC-0009); URL content not containing memory context unless user-visible |
| Over-broad `notes.*` writes | Junk memory/notes | Workspace-scoped paths only; `sensitive` visibility for memory saves |
| Tool schema drift breaks sessions | Confusing failures | `catalog_version` + graceful unknown-tool errors |
| Truncation hides key content | Bad answers | Marker + retry-with-`capture.lookup` pattern documented in prompts |

## 7. Open Questions

1. Should `web.fetch` allowlist ship with a default set (docs/wiki/news) or
   start empty (user-added)? Leaning: curated minimal default.
2. `notes.*` and `todo.*` — merge into one "workspace" tool family to save
   schema slots?
3. Community prompt-only tools: where do manifests live (repo dir vs
   settings paste)?
4. Per-tool timeout defaults — measure in M1.

## 8. Dependencies

- **RFC-0003** — `capture.take/lookup` implementation lives Rust-side.
- **RFC-0006** — memory tool semantics, episodic write restriction,
  grounding formats.
- **RFC-0007** — the loop that executes this catalog; step budget.
- **RFC-0009** — trust-level enforcement, offline gate, injection defenses.

## 9. Acceptance Mapping

| AC | Mechanism |
|---|---|
| AC-07 | `memory.search`/`capture.lookup` give the ids citations need (§4.2) |
| AC-09 | Trust-level gate at registry execute (§4.3) — network tools hard-fail offline |
| AC-11 | Catalog sized for ≤ 3-step journeys; `tool_step` visibility contract (§4.6) |

## 10. Milestone Alignment

- **M1:** capture pair, notes/todo, web pair (gated); registry trust levels.
- **M2:** memory pair with citations.
- **Post-v0:** github/deferred tools, community prompt-only extensions.
