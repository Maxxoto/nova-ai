# RFC-0011 — Post-M2 Completion: Memory Surface & Input Bindings

- **Status:** Draft for review
- **Author:** Sisyphus
- **Parent:** [RFC-0001](RFC-0001-desktop-companion.md) (Desktop Companion)
- **Relates to:** [RFC-0006](RFC-0006-memory-system.md) (memory), [RFC-0007](RFC-0007-agent-brain.md) (loop), [RFC-0002](RFC-0002-platform-shell.md) (shell/hotkeys)
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
and a new `memory_delete` command (by note id). List view with search, one
row per semantic note: snippet, source (capture thumb or `ask`), age, delete
(typed-confirm only when more than one row is selected for deletion — single
delete is one click + undo toast). Rationale: reuse the FTS index; no new
store surface or window; delete is the sensitive action so it gets the
guard, browsing does not.

### D5 — Brain involvement

None for F-15 (pure shell). F-10/F-16 touch the brain only through the
existing JSON-RPC commands — no loop changes, no new tool steps.

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

1. Should episodic notes (auto-logged asks) also be deletable from F-16, or
   only semantic? (Leaning: both, separate group in the list.)
2. Cap on semantic notes before suggesting cleanup? (Leaning: no cap, surface
   a count only.)
