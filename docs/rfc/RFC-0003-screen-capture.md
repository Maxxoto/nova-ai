# RFC-0003 — Screen Capture & Visual Context Pipeline

- **Status:** Draft for review
- **Author:** Dani
- **Parent:** [RFC-0001](RFC-0001-desktop-companion.md) (Desktop Companion)
- **Companion:** [Business PRD](../ruoxi_prd.md)
- **Covers (PRD):** F-03 (region), F-04 (active window), F-05 (fullscreen), F-08 (store + timeline), F-13 (auto-capture, opt-in), F-14 (multi-monitor picker)
- **ACs:** AC-03, AC-04, AC-05 · **Milestones:** M0 (capture), M1 (vision→LLM), M2 (timeline search)

---

## 1. Summary

Everything that turns "what the user pointed at" into (a) a durable, indexed
local artifact (the capture store) and (b) a vision-context payload for the
LLM. Three capture scopes, a metadata-indexed on-disk store, a searchable
timeline over it, and the handoff contract to the agent brain.

## 2. Motivation

"Point at anything, speak, and get help" is the product's signature gesture.
The capture pipeline is also the biggest privacy surface (screen contents at
rest) and the biggest untrusted-input surface (screen text → prompt
injection, RFC-0009). It must be exact (AC-03: *exactly* the dragged region),
cheap, and honest about retention.

## 3. Goals / Non-Goals

**Goals**
- Three scopes, correct on multi-monitor + HiDPI, excluding our own chrome.
- Local-first capture store with metadata index and searchable timeline (F-08).
- Bounded, user-controlled retention with delete-all/delete-day/pause.
- Deterministic vision handoff (downscale, format, reference-by-id).

**Non-Goals**
- Video recording, scrolling-window stitching, OCR (the LLM reads pixels).
- Auto-capture ON by default (F-13 is explicit opt-in per app).
- Cloud storage of captures (never — RFC-0009).

## 4. Detailed Design

### 4.1 Capture scopes

| Scope | Trigger | Definition | AC |
|---|---|---|---|
| Region | Region hotkey → overlay drag | Exactly the dragged rectangle, in the display's coordinate space, at native resolution | AC-03 |
| Window | Window hotkey | Frontmost window **excluding** our overlay/panel/tray-owned windows | AC-04 |
| Fullscreen | Fullscreen hotkey | The display under the cursor, native resolution | AC-05 |

"Frontmost excluding self": enumerate windows visible on the active display,
filter by ownership (our process), take the top-most remaining — window-title
and app identity captured as metadata when the OS exposes them.

### 4.2 Capture mechanics (Rust core)

- Implemented against each platform's screen-capture mechanism (macOS screen-
  capture APIs; Windows desktop-duplication/GDI-class APIs). Specific API
  choice is an implementation spike; contract fixed here.
- **HiDPI/Retina:** all geometry flows carry a scale factor; capture at
  physical pixels, store logical dimensions in metadata. Mixed-DPI multi-
  monitor is an explicit test case.
- **Multi-monitor:** displays enumerated; fullscreen scope resolves "display
  under cursor". F-14 picker (COULD): hotkey pressed with a modifier opens a
  display chooser; deferred post-M1.
- Overlay interaction detail (window owned by RFC-0002): drag rectangle →
  commit → overlay hides → capture taken **after** overlay hide (so we never
  capture our own selection chrome).

### 4.3 Capture store (F-08)

**Decision: image files on disk + SQLite index** (derived, rebuildable from a
sidecar metadata scan; images are the source of truth).

```
<data_dir>/captures/
  2026/09/18/
    cap_20260918T142233_a1f3.webp      # image artifact
    ...
capture_index.sqlite                    # metadata only, rebuildable
```

Metadata schema (SQLite table `captures`):

| Column | Type | Notes |
|---|---|---|
| `capture_id` | TEXT PK | `cap_<ulid>` |
| `ts` | INTEGER | epoch ms |
| `scope` | TEXT | region / window / fullscreen |
| `display_id` | TEXT | logical display |
| `app`, `window_title` | TEXT? | when obtainable |
| `path` | TEXT | relative image path |
| `w_px`, `h_px` | INTEGER | physical pixels |
| `scale` | REAL | display scale factor |
| `sha256` | TEXT | dedupe key |
| `retention` | TEXT | default / pinned / pending-delete |
| `auto` | INTEGER | 1 if F-13 auto-capture |

### 4.4 Searchable timeline

- Timeline = query over the index (by day, app, scope, pinned), rendering
  thumbnails lazily from files. UI lives in the shell (RFC-0002); this RFC
  owns the query surface: `timeline.query({day?, app?, scope?, text?})` where
  `text` matches app/window-title metadata (NOT image content — no OCR in v0).
- Episodic memory entries (RFC-0006) reference `capture_id`s, so the timeline
  doubles as the "screenshot cited in answer" viewer (J3, AC-07).

### 4.5 Vision handoff (to RFC-0007)

| Aspect | Decision |
|---|---|
| Reference | By `capture_id`; image bytes attached once per request |
| Downscale | Max edge ~1568 px class (vision-model friendly), preserving aspect; original stays on disk untouched |
| Format | WebP on disk; JPEG (quality ~85, target) re-encode for request payload (smaller upload) |
| Metadata | Sent alongside: app, window_title, ts, scope — helps the model ground |
| Cost note | Vision tokens dominate request cost; one image per request in v0; explicit target in RFC-0007's budget table |

Estimates above are defaults, tunable in settings.

### 4.6 Opt-in auto-capture (F-13, COULD)

- OFF by default; per-app opt-in list. Trigger policy: on app focus change +
  quiet interval (no keystroke-detection — we do not keylog), min interval
  per app (e.g., 60s). Storage amplization risk is real: auto-captures carry
  `auto=1` and a shorter default retention.
- Rationale for OFF-by-default: PRD §12 "memory feels like surveillance"
  mitigation + unwanted-capture counter-metric.

### 4.7 Retention & size management

- Default retention: **manual delete** (keeps everything until user acts);
  optional auto-delete window (e.g., 30 days) opt-in.
- Quota: soft cap (default 5 GB, configurable) → oldest unpinned auto-first
  LRU cleanup, tray notification when trimming.
- User controls: delete-all, delete-day, pause captures (tray), pin/unpin.
- All deletes hard-delete image + index row + cascade to memory citations
  (RFC-0006 marks dangling refs as `[deleted capture]`).

### 4.8 Dedupe

`sha256` exact-dup skip (same content within a session window). Near-dup
(perceptual hash) marked in metadata for timeline collapse — post-v0.

### 4.9 Untrusted-input contract

Capture pixels/text are **untrusted** (RFC-0009): the handoff message to the
LLM tags capture content as data; captures must never trigger network tools,
silent memory writes, or extra tool steps. Contract enforced in RFC-0007/0008;
flagged here because the boundary originates at this component.

## 5. Interfaces & Data Structures

```json
// Rust command (to shell/brain): capture.take
{ "scope": "region", "rect": {"x": 120, "y": 80, "w": 640, "h": 480, "display": 1} }
// → { "capture_id": "cap_01H...", "path": "2026/09/18/cap_....webp",
//     "w_px": 1280, "h_px": 960, "scale": 2.0, "app": "Preview",
//     "window_title": "lecture-7.pdf", "ts": 1758211353000 }
```

Registry tool surface (`capture.take` / `capture.lookup`) defined in RFC-0008.

## 6. Risks & Failure Modes

| Risk | Impact | Mitigation |
|---|---|---|
| Per-platform API behavior differs (permissions revoked mid-run, black frames) | Broken captures | Health-check before each capture; re-prompt permission path |
| Mixed-DPI geometry errors | Wrong region captured (AC-03 violation) | Scale-factor-aware rect math + per-display test matrix |
| Store grows unbounded | Disk exhaustion | Quota + LRU + notifications (§4.7) |
| Sensitive captures at rest | Privacy harm | Local-only, retention controls, no cloud sync (RFC-0009) |
| Timeline query slowness at scale (100k+ rows) | UX lag | SQLite indexes on ts/app; pagination |

## 7. Open Questions

1. Which exact per-platform capture API set for M0 (needs implementation
   spike on both OSes)?
2. JPEG-vs-WebP for the request payload: actual vision-model pricing/quality
   trade per provider (validate in M1)?
3. Should pinned captures be excluded from quota trimming unconditionally?
4. Window-title metadata availability per platform when privacy features
   (e.g., macOS screen-capture privacy indicators) evolve?
5. Perceptual-hash near-dup: worth the index cost in v0?

## 8. Dependencies

- **RFC-0002** — overlay window + hotkeys host the region flow; IPC carries
  capture commands.
- **RFC-0007** — vision handoff contract, latency/cost budget.
- **RFC-0006** — episodic memory references `capture_id`s; citation viewer.
- **RFC-0009** — retention governance, injection-boundary policy, AC-09
  (captures never leave device except the active request image).

## 9. Acceptance Mapping

| AC | Mechanism |
|---|---|
| AC-03 | Overlay drag → exact rect → native-res capture (§4.1/4.2) |
| AC-04 | Frontmost-excluding-self window scope (§4.1) |
| AC-05 | Cursor-display fullscreen scope, native resolution (§4.1) |

## 10. Milestone Alignment

- **M0:** three scopes + store + index (timeline UI minimal).
- **M1:** vision handoff exercised end-to-end with RFC-0007.
- **M2:** timeline search + memory-cited capture viewing (J3).
- **Post-v0:** F-13 auto-capture, F-14 picker, perceptual dedupe.
