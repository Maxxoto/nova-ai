---
version: alpha
name: Ruoxi-design-system
description: |
  The design system for Ruòxī (若曦) — "as clear as dawn's first light" — a calm,
  tray-resident AI companion for macOS + Windows built with Tauri 2, React, Tailwind
  CSS, and shadcn/ui. The visual language is built on one idea: dawn breaking over a
  quiet desk. A warm cream canvas by day, a warm near-black night mode after dusk, a
  single dawn-coral accent for every action, and a small set of honest state colors
  (live ember, thinking sky, speaking coral, offline gray) that never appear without
  an icon or a word. The companion's voice is set in a rounded face (Nunito); the
  machine's voice is set in Inter; the archive's voice is set in JetBrains Mono.
  Everything floats: the result panel hovers over the user's work with a soft shadow
  and OS vibrancy, steals no focus, and always leaves on `Esc`. Kawaii comes from
  shape and voice — soft radii, gentle breathing motion, a rare kaomoji — never from
  pastel noise. Trust is a design material: the cloud indicator is event-driven and
  can never lag reality, and every privacy state pairs a color with an icon and a
  plain-language label.

colors:
  # ---- Light theme · "Dawn" (primary) ----
  canvas: "#fbf7f1"                # hsl(36 56% 96%) dawn cream — app + panel base
  ink: "#2b241e"                   # hsl(28 18% 14%) warm near-black text
  body: "#5f5348"                  # hsl(28 13% 33%) secondary text
  mute: "#74655a"                  # hsl(25 12.6% 40.4%) metadata, captions
  faint: "#9c8f84"                 # hsl(28 10% 56%) disabled, hairline labels
  surface: "#fefdfc"               # hsl(30 50% 99%) card / popover fill
  surface-soft: "#f4ede4"          # hsl(34 42% 93%) muted fill, input rest
  primary: "#c55026"               # hsl(16 68% 46%) dawn coral — actions, speaking
  primary-hover: "#ad4520"         # hsl(16 68% 40%)
  primary-active: "#963a1b"        # hsl(15 70% 35%) pressed + citation-chip text
  primary-foreground: "#ffffff"
  primary-soft: "#f9e5db"          # hsl(20 71% 92%) coral tint — selected rows, banners, chips
  thinking-text: "#35526e"         # hsl(210 35% 32%) sky ink — "thinking" label text
  secondary: "#dce9f2"             # hsl(205 46% 91%) dawn sky fill
  secondary-foreground: "#28405a"  # hsl(211 38% 25%)
  accent: "#f8e3b7"                # hsl(41 82% 85%) dawn gold — highlights, listening fill
  accent-foreground: "#4e3a20"     # hsl(34 42% 22%)
  live: "#d16405"                  # hsl(28 95% 42%) ember — mic-live dots, waveform bars (non-text, ≥3:1 on canvas)
  warning: "#9a4a04"               # hsl(28 95% 31%) AA ember — warning text/fills (5.8:1 canvas, 5.1:1 primary-soft)
  success: "#267326"               # hsl(120 50% 30%) dawn green — local-only, saved
  destructive: "#bb2b1f"           # hsl(5 72% 43%) — errors, destructive actions (5.6:1 canvas, 5.0:1 primary-soft)
  destructive-foreground: "#ffffff"
  border: "#e2dbd2"                # hsl(34 22% 85%)
  border-strong: "#cfc5b8"         # hsl(34 19% 77%)
  overlay-dim: "rgba(43, 36, 30, 0.35)"  # capture dim over the user's screen
  # ---- Dark theme · "Night before dawn" ----
  dark-canvas: "#1d1815"           # hsl(22 16% 10%)
  dark-ink: "#f2e9de"              # hsl(33 43% 91%) warm off-white
  dark-body: "#c9bcb0"             # hsl(29 20% 74%)
  dark-mute: "#a39386"             # hsl(27 69% 58%)
  dark-surface: "#262019"          # hsl(32 21% 12%)
  dark-surface-soft: "#302922"     # hsl(30 17% 16%)
  dark-primary: "#d96f42"          # hsl(18 67% 55%) brighter coral
  dark-primary-hover: "#e47e50"    # hsl(19 72% 60%)
  dark-primary-active: "#eda47f"   # hsl(20 75% 71%) pale coral — dark chip/citation text
  dark-primary-foreground: "#1d130c" # hsl(25 41% 8%) dark text on bright coral (AA)
  dark-primary-soft: "#5a3516"     # hsl(27 61% 22%) coral tint — dark chips
  dark-thinking: "#7fb3d5"         # hsl(204 51% 67%) sky for dark
  dark-speaking: "#e07a4e"         # hsl(18 70% 59%) coral for dark
  dark-secondary: "#273849"        # hsl(210 30% 22%) night-sky fill
  dark-secondary-foreground: "#dbe9f4" # hsl(206 53% 91%)
  dark-accent: "#5a3516"           # hsl(27 61% 22%) amber-tint fill for dark
  dark-accent-foreground: "#fce2b6" # hsl(38 92% 85%)
  dark-live: "#f7a033"             # hsl(33 92% 58%)
  dark-success: "#4d9e5d"          # hsl(132 34% 46%)
  dark-destructive: "#ee6a63"      # hsl(3 80% 66%) — dark error text (5.8:1 canvas)
  dark-destructive-foreground: "#1d130c" # hsl(25 41% 8%) dark ink on bright red fills (6.0:1)
  dark-warning: "#f97d10"          # hsl(28 95% 52%) — dark degraded text (6.7:1 canvas)
  dark-border: "#3a332b"           # hsl(32 15% 20%)

typography:
  font-ui: "Inter — UI chrome: answers, settings, timeline, buttons. 400/500/600."
  font-companion: "Nunito — Ruòxī's voice: state labels, greetings, daily brief, onboarding why-lines. 600/700."
  font-mono: "JetBrains Mono — capture ids, citations, timestamps, hotkey kbd. 400/500."
  display:
    fontSize: 28px
    fontWeight: 700
    fontFamily: "{typography.font-ui}"
    lineHeight: 1.2
    letterSpacing: "-0.02em"
    use: "Onboarding headline only — one per flow"
  title:
    fontSize: 17px
    fontWeight: 600
    fontFamily: "{typography.font-ui}"
    lineHeight: 1.3
    letterSpacing: "-0.01em"
    use: "Settings/timeline section titles, modal titles"
  companion:
    fontSize: 15px
    fontWeight: 600
    fontFamily: "{typography.font-companion}"
    lineHeight: 1.35
    use: "Panel state labels, daily brief, greeting lines, why-lines"
  body:
    fontSize: 14px
    fontWeight: 400
    fontFamily: "{typography.font-ui}"
    lineHeight: 1.45
    use: "Default — answer text, settings rows, timeline metadata"
  body-strong:
    fontSize: 14px
    fontWeight: 600
    fontFamily: "{typography.font-ui}"
    lineHeight: 1.45
    use: "Row labels, button labels, emphasis inside answers"
  body-sm:
    fontSize: 13px
    fontWeight: 400
    fontFamily: "{typography.font-ui}"
    lineHeight: 1.45
    use: "Dense lists, transcript preview, tool-step labels"
  caption:
    fontSize: 11px
    fontWeight: 500
    fontFamily: "{typography.font-ui}"
    lineHeight: 1.4
    use: "Cloud indicator label, dimension chip, badges"
  mono:
    fontSize: 11px
    fontWeight: 500
    fontFamily: "{typography.font-mono}"
    lineHeight: 1.4
    use: "capture_id chips, citations, timestamps, kbd hotkeys"

rounded:
  sm: 6px      # inputs, chips, small buttons
  md: 10px     # default --radius: buttons, cards, menu items
  lg: 14px     # panel shell, modals, daily-brief card
  xl: 18px     # onboarding cards, settings windows
  pill: 9999px # ptt-pill, waveform pill, cloud-indicator chip, orb
  sharp: 2px   # capture selection rectangle ONLY — precision against softness

spacing:
  xs: 4px
  sm: 8px
  md: 12px
  lg: 16px     # panel inner padding (with 20px horizontal)
  xl: 20px
  xxl: 32px
  clamp: 12px  # screen-edge clamp for every floating surface
  panel-w: 400px  # panel width; min 360, max 440

motion:
  duration-instant: 80ms    # presses, toggles
  duration-fast: 120ms      # Esc dismiss, panel hide (AC-06 — must feel instant)
  duration-standard: 200ms  # panel show, state changes, save morph
  duration-slow: 320ms      # overlay dim in, brief card entrance
  duration-deliberate: 480ms # onboarding transitions
  ease-standard: "cubic-bezier(0.4, 0, 0.2, 1)"
  ease-out: "cubic-bezier(0.16, 1, 0.3, 1)"
  ease-snappy: "cubic-bezier(0.32, 0.72, 0, 1)"
  breathe: "3s ease-in-out infinite — idle orb scale 1 → 1.03"
  pulse-ring: "1.2s ease-out infinite — listening ring scale 1 → 1.35, opacity 0.5 → 0"
  orbit: "1.6s linear infinite — thinking dots around orb"
  wave: "0.9s ease-in-out — waveform bars, driven by live mic level, not a timer"
  shimmer: "1.4s linear infinite — thinking skeleton blocks"
  token-caret: "1s step-end infinite — streaming caret blink"
  reduced-motion: "prefers-reduced-motion: reduce → every transform anim becomes an opacity fade, durations halve, waveform bars freeze with a static 'listening' label + icon"

components:
  panel-shell:
    backgroundColor: "{colors.canvas}"
    textColor: "{colors.ink}"
    rounded: "{rounded.lg}"
    width: "{spacing.panel-w}"
    shadow: "0 8px 32px rgba(43, 36, 30, 0.16), 0 2px 8px rgba(43, 36, 30, 0.08)"
    vibrancy: "backdrop-filter: blur(24px) saturate(140%); surface opacity 0.78; fallback = opacity 0.97, no blur (RFC-0002 risk #3)"
    focus: "never activates — non-activating window on both OSes"
  panel-header:
    height: 44px
    layout: "orb 20px + state label ({typography.companion} 13px) + spacer + cloud-indicator + Esc kbd hint"
    borderBottom: "1px solid {colors.border}"
  button-primary:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.primary-foreground}"
    typography: "{typography.body-strong}"
    rounded: "{rounded.md}"
    padding: "8px 16px"
    height: 36px
  button-primary-hover: "{colors.primary-hover}"
  button-primary-active: "{colors.primary-active}"
  button-secondary:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.ink}"
    border: "1px solid {colors.border-strong}"
    rounded: "{rounded.md}"
    padding: "8px 16px"
  button-ghost:
    backgroundColor: transparent
    textColor: "{colors.mute}"
    hover: "background {colors.surface-soft}, text {colors.ink}"
  button-save-memory:
    extends: "{components.button-secondary}"
    icon: "bookmark — after save: check icon morph, border {colors.success}, 200ms {motion.duration-standard}"
  ptt-pill:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.ink}"
    rounded: "{rounded.pill}"
    shadow: "e3 — same as panel: 0 8px 32px rgba(43, 36, 30, 0.16), 0 2px 8px rgba(43, 36, 30, 0.08)"
    content: "live dot ({colors.live}, {motion.pulse-ring}) + 5 waveform bars ({colors.live}, {motion.wave} driven by real mic level) + label {typography.companion} 13px 'listening'"
    position: "near cursor, {spacing.clamp} from screen edges"
  capture-overlay:
    dim: "{colors.overlay-dim}"
    selection:
      border: "1.5px solid {colors.primary}"
      rounded: "{rounded.sharp}"
      handles: "6px squares at 4 corners, {colors.surface} fill, {colors.primary} border"
    dimension-chip:
      backgroundColor: "{colors.primary}"
      textColor: "{colors.primary-foreground}"
      typography: "{typography.mono}"
      rounded: "{rounded.sm}"
      content: "W × H px, floats below selection"
  cloud-indicator:
    offline:
      icon: "cloud-off"
      dot: "{colors.mute}"
      label: "offline · nothing leaves this Mac"
    local-only:
      icon: "cloud (hollow)"
      dot: "{colors.success}"
      label: "local only"
    calling-cloud:
      icon: "cloud-upload"
      dot: "{colors.live} pulsing"
      label: "sending to cloud"
    rule: "event-driven from NetPolicy — never a timer, never color alone (icon + label always)"
  tool-step-indicator:
    shape: "3 dots, 6px, gap 4px"
    done: "{colors.primary}"
    active: "{colors.primary} + {motion.pulse-ring}"
    pending: "{colors.border-strong}"
    label: "{typography.body-sm} 'step 1 of 3'"
  answer-stream:
    typography: "{typography.body}"
    streaming: "token-by-token reveal + {motion.token-caret}; tool steps shown by {components.tool-step-indicator}"
    citation-chip:
      backgroundColor: "{colors.primary-soft}"   # dark: {colors.dark-primary-soft}
      textColor: "{colors.primary-active}"       # dark: {colors.dark-primary-active}
      typography: "{typography.mono}"
      rounded: "{rounded.sm}"
      padding: "2px 6px"
      hover: "screenshot thumbnail popover 160px, rounded {rounded.md}, shadow e2"
  daily-brief:
    backgroundColor: "{colors.surface}"
    rounded: "{rounded.xl}"
    shadow: "e3 — same as panel: 0 8px 32px rgba(43, 36, 30, 0.16), 0 2px 8px rgba(43, 36, 30, 0.08)"
    greeting: "{typography.companion} — may end with a kaomoji (see Do's)"
    brief: "{typography.body} one line + up to 3 memory highlights"
  onboarding-card:
    backgroundColor: "{colors.surface}"
    rounded: "{rounded.xl}"
    padding: 32px
    why-line: "{typography.companion} — plain-language reason BEFORE each OS permission prompt (AC-12)"
    permission-name: "{typography.body-strong}"
  banner-error:
    backgroundColor: "{colors.primary-soft}"   # dark bg: {colors.dark-surface-soft} (bright coral tint fails AA under dark text rules)
    textColor: "{colors.destructive}"          # dark: {colors.dark-destructive}
    border: "1px solid {colors.destructive}"   # dark: {colors.dark-destructive}
    rounded: "{rounded.md}"
    typography: "{typography.body-sm}"
    action: "retry ghost button inline"
  banner-degraded:
    extends: "{components.banner-error}"
    textColor: "{colors.warning}"          # dark: {colors.dark-warning}
    border: "1px solid {colors.warning}"   # dark: {colors.dark-warning}
    content: "Ruòxī is resting — the assistant core is restarting"
  tray-menu:
    type: native OS menu
    items:
      - "daily brief (F-12 — opens brief card)"
      - "pause captures — checkbox, pauses ALL capture; when ON the tray icon swaps to its paused variant (F-13 off-switch, RFC-0009 §4.5)"
      - "offline mode — checkbox, the kill-switch control; instant effect, no restart (F-09, RFC-0009 §4.1)"
      - "open settings · open timeline · check for updates (disabled while offline)"
      - "quit"
  capture-paused-state:
    rule: "visually obvious the whole time captures are paused (RFC-0009 §4.5) — tray icon variant + persistent 'captures paused' chip in panel chrome; capture hotkeys show a one-line dimmed overlay note instead of silently doing nothing"
---

# Ruòxī Design System (若曦)

> Companion to [RFC-0002](rfc/RFC-0002-platform-shell.md) (shell/panel), [RFC-0003](rfc/RFC-0003-screen-capture.md) (capture),
> [RFC-0009](rfc/RFC-0009-privacy-offline.md) (indicator). Traces to PRD features F-01–F-14 and ACs 01–12.

## Overview

Ruòxī is not a chat window. She is a quiet companion who lives in the tray and appears
only when called — a small floating panel near where the user is already looking, a
transient listening pill near the cursor, a precise selection box over the thing being
pointed at. The design language renders that promise ("always one keystroke away; never
in the way") in four moves:

1. **Dawn, not neon.** 若曦 means "as clear as dawn's first light." The palette is a
   warm cream canvas (`{colors.canvas}`), warm ink text (`{colors.ink}`), and one
   dawn-coral accent (`{colors.primary}`) that carries every action. No gradients, no
   glow, no purple-cyber. Dark mode is "night before dawn": the same warmth inverted.
2. **Kawaii through shape and voice, not pastel.** Softness lives in radii (10–18px),
   in the breathing idle orb, in Nunito's rounded terminals when *she* speaks, and in
   one rare kaomoji at the right moment. The machine chrome stays in Inter and stays
   restrained — companion warmth against professional bones. (Competitors split into
   purple-glow fandom or pastel play; Ruòxī is deliberately the calm third thing.)
3. **Honest states.** Every system state — listening, thinking, speaking, offline,
   sending to cloud — is a color **plus an icon plus a word**, event-driven from real
   IPC events (`agent.*`, `stt.*`, `net.state`), never a timer, never color alone.
   The indicator can never lie by lagging (RFC-0009 §4.2).
4. **Float, don't push.** The panel hovers with a single signature shadow and OS
   vibrancy, never takes focus, clamps 12px from screen edges, and `Esc` dismisses it
   in 120ms with no side effects (AC-06). When vibrancy fails on a platform, the
   fallback is a near-opaque surface — never a broken effect (RFC-0002 risk #3).

## Colors

### Light · "Dawn" (default)

| Token | Value | Role |
|---|---|---|
| `{colors.canvas}` | `#fbf7f1` | App + panel base. Warm, paper-like. |
| `{colors.ink}` | `#2b241e` | Primary text. Warm near-black, never pure black. |
| `{colors.body}` | `#5f5348` | Paragraph text, secondary rows. |
| `{colors.mute}` | `#74655a` | Metadata, timestamps, inactive icons. |
| `{colors.faint}` | `#9c8f84` | Disabled text, hairline annotations. |
| `{colors.surface}` | `#fefdfc` | Cards, popovers, menus. |
| `{colors.surface-soft}` | `#f4ede4` | Inputs at rest, hovered rows, muted fills. |
| `{colors.primary}` | `#c55026` | Dawn coral. All primary actions; "speaking" state. AA (4.6:1) with white. |
| `{colors.primary-soft}` | `#f9e5db` | Coral tint. Selected rows, citation chips. |
| `{colors.secondary}` | `#dce9f2` | Dawn sky fill. Thinking tint, info surfaces. |
| `{colors.accent}` | `#f8e3b7` | Dawn gold. Highlights, listening fill washes. |
| `{colors.live}` | `#d16405` | Ember. Mic-live dots + waveform bars only — never text. |
| `{colors.warning}` | `#9a4a04` | AA ember for warning text/fills (5.8:1 on canvas, 5.1:1 on `primary-soft`). |
| `{colors.success}` | `#267326` | Local-only indicator, saved confirmations. |
| `{colors.destructive}` | `#bb2b1f` | Errors, destructive confirmations. |
| `{colors.border}` | `#e2dbd2` | Hairlines, separators, input strokes. |
| `{colors.border-strong}` | `#cfc5b8` | Secondary button strokes, tab rules. |
| `{colors.overlay-dim}` | `rgba(43,36,30,.35)` | Capture overlay dim over the user's screen. |

### Dark · "Night before dawn"

| Token | Value | Role |
|---|---|---|
| `{colors.dark-canvas}` | `#1d1815` | Base. Warm near-black, never blue-black. |
| `{colors.dark-ink}` | `#f2e9de` | Primary text. |
| `{colors.dark-body}` / `{colors.dark-mute}` | `#c9bcb0` / `#a39386` | Secondary / metadata. |
| `{colors.dark-surface}` / `-soft` | `#262019` / `#302922` | Cards / inputs. |
| `{colors.dark-primary}` | `#d96f42` | Brighter coral; pairs with `{colors.dark-primary-foreground}` dark text (AA). |
| `{colors.dark-live}` | `#f7a033` | Ember for dark. |
| `{colors.dark-thinking}` | `#7fb3d5` | Sky for dark. |
| `{colors.dark-speaking}` | `#e07a4e` | Coral for dark. |
| `{colors.dark-success}` | `#4d9e5d` | Green for dark. |
| `{colors.dark-border}` | `#3a332b` | Hairlines. |

### State color assignments (the honest-states palette)

| State | Light | Dark | Icon | Label |
|---|---|---|---|---|
| Idle | `{colors.mute}` | `{colors.dark-mute}` | orb (breathing) | — |
| Listening | `{colors.live}` | `{colors.dark-live}` | mic + waveform | "listening" |
| Thinking | `{colors.thinking-text}` | `{colors.dark-thinking}` | orbit dots | "thinking" |
| Speaking | `{colors.primary}` | `{colors.dark-speaking}` | sound waves | "speaking" |
| Offline | `{colors.mute}` | `{colors.dark-mute}` | cloud-off | "offline · nothing leaves this Mac" |
| Local only | `{colors.success}` | `{colors.dark-success}` | cloud (hollow) | "local only" |
| Calling cloud | `{colors.live}` (pulse) | `{colors.dark-live}` | cloud-upload | "sending to cloud" |
| Error | `{colors.destructive}` | `{colors.dark-destructive}` | alert | message + retry |
| Degraded | `{colors.warning}` | `{colors.dark-warning}` | moon-zzz | "Ruòxī is resting…" |
| Captures paused | `{colors.mute}` | `{colors.dark-mute}` | pause | "captures paused" — tray + panel chrome, obvious until resumed |

**Rules.** State colors are reserved — coral is action/speaking, ember is *live/
egress*, sky is *thinking*, green is *local/saved*. One accent per view. Every state
pairs color with icon and label; if you can't name the icon, the color is wrong.

## Typography

| Token | Face | Size/Weight | Use |
|---|---|---|---|
| `{typography.display}` | Inter | 28/700 | Onboarding headline only — one per flow |
| `{typography.title}` | Inter | 17/600 | Section + modal titles |
| `{typography.companion}` | Nunito | 15/600 (13px variant in panel header) | Her voice: state labels, brief, why-lines |
| `{typography.body}` | Inter | 14/400 | Default — answers, settings, timeline |
| `{typography.body-strong}` | Inter | 14/600 | Labels, buttons, emphasis |
| `{typography.body-sm}` | Inter | 13/400 | Dense lists, transcript preview |
| `{typography.caption}` | Inter | 11/500 | Indicator labels, dimension chip, badges |
| `{typography.mono}` | JetBrains Mono | 11/500 | capture_ids, citations, timestamps, kbd |

**The three-voice rule.** Inter is the *machine* voice (everything functional).
Nunito is *her* voice (only where Ruòxī herself speaks: greetings, state labels,
onboarding why-lines, daily brief). JetBrains Mono is the *archive* voice (anything
that references stored captures or time). If a string could be spoken aloud *by her*,
it's Nunito; otherwise it isn't. Never use Nunito for buttons, settings, or answers.

**Load:** Inter 400/500/600, Nunito 600/700, JetBrains Mono 400/500 — system-ui
fallbacks. Bundle as WOFF2 (~180KB total); no font trading latency for a ≤2s answer
budget (PRD §9).

## Layout

**Spacing scale:** `{spacing.xs}` 4 · `{spacing.sm}` 8 · `{spacing.md}` 12 ·
`{spacing.lg}` 16 · `{spacing.xl}` 20 · `{spacing.xxl}` 32 · `{spacing.clamp}` 12 ·
`{spacing.panel-w}` 400px. Base unit 4px; panel body breathes at 16/20 (v/h).

**Panel geometry (F-06):** width 400px (min 360, max 440); max-height 60vh with
internal scroll below; positioned near the capture region or cursor, then clamped
≥ `{spacing.clamp}` from every screen edge (RFC-0002 §4.4). It may overlap the
user's work; it must never cover the selection the user just made.

**Floating-surface hierarchy:** ptt-pill < daily-brief < panel < capture-overlay.
Only one floating surface owns the screen at a time; a new capture dismisses the panel.

**Settings & timeline windows:** centered, 720px max width, `{spacing.xxl}` padding,
light chrome — these are ordinary windows, not overlays; they may take focus.

## Elevation & Depth

| Level | Treatment | Use |
|---|---|---|
| e1 | `0 1px 2px rgba(43,36,30,.06)` | Cards, rows at rest |
| e2 | `0 4px 12px rgba(43,36,30,.10)` | Popovers, citation thumbnails, menus |
| e3 | `0 8px 32px rgba(43,36,30,.16), 0 2px 8px rgba(43,36,30,.08)` | **The signature.** Panel, ptt-pill, daily-brief |
| vibrancy | `backdrop-filter: blur(24px) saturate(140%)`, surface at 78% opacity | Panel + brief on macOS/WebView2; fallback = 97% opacity, no blur |

Depth is reserved for things that float over the user's work. In-window content is
flat: cards separate with `{colors.border}` hairlines and surface tints, not shadows.
The capture overlay uses no elevation at all — the dim (`{colors.overlay-dim}`) *is*
its depth. Warm-tinted shadow rgba only; never neutral black, never coral-tinted.

## Shapes

| Token | Value | Use |
|---|---|---|
| `{rounded.sm}` | 6px | Inputs, chips, citation chips, dimension chip |
| `{rounded.md}` | 10px | Buttons, menu items, small cards (shadcn `--radius`) |
| `{rounded.lg}` | 14px | **Panel shell**, modals |
| `{rounded.xl}` | 18px | Onboarding + brief cards, settings window |
| `{rounded.pill}` | 9999px | ptt-pill, cloud-indicator chip, orbs |
| `{rounded.sharp}` | 2px | **Capture selection only** |

The deliberate tension: everything Ruòxī *is* is soft; everything she *draws on your
screen* is sharp. The selection rectangle's 2px corners + corner handles read as
precision instruments against the soft chrome — the user is trusting the crop.

## AI State System

The orb is the avatar: a 20–28px coral disc with a soft inner glow. Orb states map
1:1 to the IPC reality (RFC-0002 §4.6 notifications) — every *behavioral* transition
below is driven by a received event, never a timeout. The single exception is the
cosmetic `complete → idle` settle (a UI-only timer after 2s of stillness, with no
state semantics); anything that gates a feature waits for a real event:

| State | Trigger (event) | Orb | Chrome |
|---|---|---|---|
| **idle** | panel hidden / 2s after complete (cosmetic settle — the only timer, see above) | `motion.breathe` scale 1→1.03 | tray = dawn dot |
| **listening** | PTT key-down | glow expands (`pulse-ring`) | ptt-pill appears near cursor: live dot + 5 bars (`motion.wave`, mic-level-driven) + "listening" |
| **transcribing** | PTT key-up → `stt.final` | bars freeze → 3 dots | pill swaps waveform for transcript preview (body-sm) |
| **thinking** | `session.ask` sent | `motion.orbit` 3 sky dots | panel opens in streaming state; tool steps tick `{components.tool-step-indicator}`; skeleton blocks shimmer in answer area |
| **speaking** | TTS start (RFC-0005) | `motion.wave` reactive bounce | header label "speaking"; `Esc` stops audio (AC-06) |
| **streaming** | `agent.token` notifications | coral, still | tokens reveal with caret in `{components.answer-stream}` |
| **complete** | final token | back to breathe | Save-to-memory button becomes visible (F-10) |
| **error** | agent/LLM/network error | destructive ring | `{components.banner-error}` inline, retry affordance |
| **degraded** | sidecar restart exceeded max (RFC-0002 §4.7) | moon-zzz | `{components.banner-degraded}` + tray variant |

**Privacy is a first-class state machine.** The `{components.cloud-indicator}` chip
lives in the panel header *and* the tray menu simultaneously, driven by `net.state`
events (`offline / local_only / calling_cloud`). It transitions on request lifecycle,
not timers — an honest sub-second view of egress (RFC-0009 §4.2, F-09, AC-09).

**Motion law.** `Esc` = 120ms fade (`motion.duration-fast`) — dismissal must feel
instant (AC-06). Entrances use `ease-out`, exits use `ease-standard`. Nothing loops
except: breathe (idle), pulse-ring (listening), orbit (thinking), wave (live audio),
shimmer (loading), caret (streaming). Every loop must reflect a real, current system
condition — the listening pulse runs because the mic is open, not because a timer
says so; the only pure timer in the product is the cosmetic idle settle. Under `prefers-reduced-motion`: transforms become
opacity fades, durations halve, waveform freezes to a static labeled icon — the
state must remain legible without motion.

## Components

*(Full token specs in front matter; behavioral notes here.)*

- **`panel-shell`** (F-06, AC-06) — frameless, non-activating, vibrancy, `{rounded.lg}`,
  e3. Three zones: header / answer-stream / action footer. Never steals focus from the
  user's app — validated in the AC-06 suite (RFC-0002 risk table).
- **`answer-stream`** (F-07, AC-07) — Inter 14, markdown-rendered; inline
  citation chips (`{components.answer-stream}.citation-chip`) reference the exact
  capture: hover shows the screenshot thumbnail; click opens the timeline entry.
- **`button-save-memory`** (F-10, AC-08) — visible only on complete; morphs to a green
  check on save; nothing writes to memory without this explicit action (RFC-0009 §4.6).
- **`ptt-pill`** (F-02, AC-02) — transient, positioned at cursor, clamped; waveform
  bars follow real mic RMS. Appears ≤100ms after key-down; gone 300ms after `stt.final`.
- **`capture-overlay`** (F-03, F-04, F-05, AC-03, AC-04, AC-05) — dims all monitors, sharp selection
  + handles + live dimension chip; mode hint row at top (region / window / fullscreen);
  display picker on multi-monitor setups (F-14, RFC-0003).
- **TTS read-back toggle** (F-11) — panel footer icon-toggle; ON shows the speaking
  state's reactive waveform; OFF mutes after the current sentence; `Esc` always stops.
- **Offline kill-switch control** (F-09, J7) — a checkbox in the tray menu *and* a
  toggle in settings (introduced during onboarding): instant, no restart, and the
  moment it flips, the `{components.cloud-indicator}` moves to `offline` everywhere.
- **Pause-captures control + state** (RFC-0009 §4.5) — one tray click; the paused
  state is visually obvious (`{components.capture-paused-state}`) until resumed.
- **Auto-capture settings** (F-13, RFC-0009 §4.5) — per-app opt-in list in settings;
  every app OFF by default; each row states plainly what is stored and when it
  captures (the "surveillance feel" mitigation is a design obligation, not copy).
- **Tray menu** (F-01, RFC-0002 §4.2) — native, contents per `{components.tray-menu}`;
  mirrors the offline + paused states so the tray alone tells the truth.
- **`cloud-indicator`** (F-09, AC-09) — the trust chip. Same component in panel header
  and tray menu; states per front matter; label always in words.
- **`tool-step-indicator`** (AC-11) — 3 fixed segments; abort via `Esc` is always live.
- **`daily-brief`** (F-12, J6) — one Nunito greeting line + one-line brief + ≤3 memory
  highlights; kaomoji permitted here and in the onboarding finale, nowhere else.
- **`onboarding-card`** (J8, AC-12) — the why-line ritual: permission name (Inter 600),
  why (Nunito 15), then the OS prompt. Ends with the guided first capture — the
  "it works!" moment gets the only kaomoji besides the brief.
- **`timeline-view`** (F-08) — capture grid with mono timestamps, search, day groups;
  delete-all uses typed confirmation (RFC-0009 §4.7).
- **Banners** — `banner-error` / `banner-degraded`: inline, tinted, always with an
  action; never toast modals.

## Do's and Don'ts

### Do
- Lead with calm: one idea per surface, generous air, no competing accents.
- Use the three-voice rule (Inter machine / Nunito her / Mono archive) on every string.
- Pair every state color with icon + word; drive every state from a real event.
- Reserve `{colors.live}` ember for genuinely-live things (mic, egress).
- Keep `{rounded.sharp}` for the capture rectangle only — it's the precision accent.
- Make `Esc` work in 120ms on every floating surface, with zero side effects.
- Honor `prefers-reduced-motion`, `focus-visible` (2px `{colors.primary}` ring, offset 2), WCAG AA.
- Use a kaomoji at most twice in the product: daily-brief greeting, onboarding finale.

### Don't
- Don't use gradients, glows, neon, or purple-cyber treatments. Dawn is flat and warm.
- Don't express states with color alone — trust surfaces especially (RFC-0009).
- Don't animate a state the system isn't actually in; no decorative loop without an event.
- Don't let the panel exceed 440px or cover the user's fresh selection.
- Don't use Nunito for machine chrome or Inter for her voice.
- Don't place white text on `{colors.live}` (fails AA — it's a dot/bar color, use `{colors.warning}` for ember text).
- Don't render recorded or replayable audio UI — no waveforms of past audio, no
  playback controls, no audio files surfacing anywhere: audio is ephemeral by law
  (AC-10, RFC-0009 §4.4). Live mic-level indication IS allowed and required (ptt-pill
  bars, speaking reactive wave) — it visualizes the live stream only, never a recording.
- Don't add a component that can't be expressed in the existing token vocabulary
  without first adding the tokens to front matter.

## Responsive & Platform Behavior

- **Displays:** position relative to the capture/cursor's monitor; clamp per-monitor;
  the overlay spans all monitors (RFC-0003), the panel lives on the active one.
- **Windows DPI 125%/150%:** geometry in CSS px; vibrancy = WebView2 backdrop-filter;
  if DWM acrylic artifacts appear (RFC-0002 risk #2/#3) → opacity-0.97 fallback, no blur.
- **macOS:** vibrancy via backdrop-filter over WKWebView; panel ignores Dock (accessory
  policy, AC-01).
- **Themes:** follow OS light/dark on both platforms; manual override in settings.
- **Focus & keyboard:** panel is non-activating; all in-panel interaction reachable by
  keyboard once user clicks into it; `Tab` order header → stream → footer; visible
  focus ring everywhere.
- **Small screens / multi-monitor edge cases:** panel re-clamps on display change;
  if the anchor point leaves the screen, panel docks to the nearest edge at `{spacing.clamp}`.

## Tailwind + shadcn Implementation

```css
/* app.css — shadcn theme bridge. GENERATED from front-matter hexes by sync (each var ↔ token, cross-checked by scripts/verify_design_md.py). */
:root {
  --background: 36.0 55.6% 96.5%; /* {colors.canvas} */
  --foreground: 27.7 17.8% 14.3%; /* {colors.ink} */
  --card: 30.0 50.0% 99.2%;       /* {colors.surface} */
  --card-foreground: 27.7 17.8% 14.3%; /* {colors.ink} */
  --popover: 30.0 50.0% 99.2%;    /* {colors.surface} */
  --popover-foreground: 27.7 17.8% 14.3%; /* {colors.ink} */
  --primary: 15.8 67.7% 46.1%;    /* {colors.primary} */
  --primary-foreground: 0 0% 100.0%; /* {colors.primary-foreground} */
  --primary-soft: 20.0 71.4% 91.8%; /* {colors.primary-soft} */
  --primary-active: 15.1 69.5% 34.7%; /* {colors.primary-active} */
  --thinking-text: 209.5 35.0% 32.0%; /* {colors.thinking-text} */
  --speaking-text: 15.8 67.7% 46.1%; /* {colors.primary} */
  --secondary: 204.5 45.8% 90.6%; /* {colors.secondary} */
  --secondary-foreground: 211.2 38.5% 25.5%; /* {colors.secondary-foreground} */
  --muted: 33.7 42.1% 92.5%;      /* {colors.surface-soft} */
  --muted-foreground: 25.4 12.6% 40.4%; /* {colors.mute} */
  --accent: 40.6 82.3% 84.5%;     /* {colors.accent} */
  --accent-foreground: 33.9 41.8% 21.6%; /* {colors.accent-foreground} */
  --destructive: 4.6 71.6% 42.7%; /* {colors.destructive} */
  --destructive-foreground: 0 0% 100.0%; /* {colors.destructive-foreground} */
  --success: 120.0 50.3% 30.0%;   /* {colors.success} */
  --warning: 28.0 94.9% 31.0%;    /* {colors.warning} */
  --live: 27.9 95.3% 42.0%;       /* {colors.live} */
  --border: 33.7 21.6% 85.5%;     /* {colors.border} */
  --input: 33.7 21.6% 85.5%;      /* {colors.border} */
  --ring: 15.8 67.7% 46.1%;       /* {colors.primary} */
}
.dark {
  --background: 22.5 16.0% 9.8%;  /* {colors.dark-canvas} */
  --foreground: 33.0 43.5% 91.0%; /* {colors.dark-ink} */
  --card: 32.3 20.6% 12.4%;       /* {colors.dark-surface} */
  --card-foreground: 33.0 43.5% 91.0%; /* {colors.dark-ink} */
  --popover: 32.3 20.6% 12.4%;    /* {colors.dark-surface} */
  --popover-foreground: 33.0 43.5% 91.0%; /* {colors.dark-ink} */
  --primary: 17.9 66.5% 55.5%;    /* {colors.dark-primary} */
  --primary-foreground: 24.7 41.5% 8.0%; /* {colors.dark-primary-foreground} */
  --primary-soft: 27.4 60.7% 22.0%; /* {colors.dark-primary-soft} */
  --primary-active: 20.2 75.3% 71.4%; /* {colors.dark-primary-active} */
  --thinking-text: 203.7 50.6% 66.7%; /* {colors.dark-thinking} */
  --speaking-text: 18.1 70.2% 59.2%; /* {colors.dark-speaking} */
  --secondary: 210.0 30.4% 22.0%; /* {colors.dark-secondary} */
  --secondary-foreground: 206.4 53.2% 90.8%; /* {colors.dark-secondary-foreground} */
  --muted: 30.0 17.1% 16.1%;      /* {colors.dark-surface-soft} */
  --muted-foreground: 26.9 13.6% 58.2%; /* {colors.dark-mute} */
  --accent: 27.4 60.7% 22.0%;     /* {colors.dark-accent} */
  --accent-foreground: 37.7 92.1% 85.1%; /* {colors.dark-accent-foreground} */
  --destructive: 3.0 80.3% 66.1%; /* {colors.dark-destructive} */
  --destructive-foreground: 24.7 41.5% 8.0%; /* {colors.dark-destructive-foreground} */
  --success: 131.9 34.5% 46.1%;   /* {colors.dark-success} */
  --warning: 28.1 95.1% 52.0%;    /* {colors.dark-warning} */
  --live: 33.4 92.5% 58.4%;       /* {colors.dark-live} */
  --border: 32.0 14.9% 19.8%;     /* {colors.dark-border} */
  --input: 32.0 14.9% 19.8%;      /* {colors.dark-border} */
  --ring: 17.9 66.5% 55.5%;       /* {colors.dark-primary} */
}
```

```ts
// tailwind.config.ts (extract)
export default {
  theme: {
    extend: {
      colors: {
        border: "hsl(var(--border))", input: "hsl(var(--input))",
        ring: "hsl(var(--ring))", background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        primary: { DEFAULT: "hsl(var(--primary))", foreground: "hsl(var(--primary-foreground))" },
        secondary: { DEFAULT: "hsl(var(--secondary))", foreground: "hsl(var(--secondary-foreground))" },
        destructive: { DEFAULT: "hsl(var(--destructive))", foreground: "hsl(var(--destructive-foreground))" },
        muted: { DEFAULT: "hsl(var(--muted))", foreground: "hsl(var(--muted-foreground))" },
        accent: { DEFAULT: "hsl(var(--accent))", foreground: "hsl(var(--accent-foreground))" },
        success: "hsl(var(--success))",
        warning: "hsl(var(--warning))",
        live: "hsl(var(--live))",
        "primary-soft": "hsl(var(--primary-soft))",
        "primary-active": "hsl(var(--primary-active))",
        "thinking-text": "hsl(var(--thinking-text))",
        "speaking-text": "hsl(var(--speaking-text))",
        card: { DEFAULT: "hsl(var(--card))", foreground: "hsl(var(--card-foreground))" },
      },
      fontFamily: {
        ui: ["Inter", "system-ui", "sans-serif"],
        companion: ["Nunito", "system-ui", "sans-serif"],
        mono: ["JetBrains Mono", "ui-monospace", "monospace"],
      },
      borderRadius: {
        sm: "6px", DEFAULT: "var(--radius)", lg: "14px", xl: "18px",
        pill: "9999px", sharp: "2px",
      },
      boxShadow: {
        e1: "0 1px 2px rgba(43,36,30,0.06)",
        e2: "0 4px 12px rgba(43,36,30,0.10)",
        e3: "0 8px 32px rgba(43,36,30,0.16), 0 2px 8px rgba(43,36,30,0.08)",
      },
      backdropBlur: { panel: "24px" },
      keyframes: {
        breathe: { "0%,100%": { transform: "scale(1)" }, "50%": { transform: "scale(1.03)" } },
        "pulse-ring": { "0%": { transform: "scale(1)", opacity: "0.5" }, "100%": { transform: "scale(1.35)", opacity: "0" } },
        orbit: { to: { transform: "rotate(360deg)" } },
        shimmer: { "0%": { backgroundPosition: "-200% 0" }, "100%": { backgroundPosition: "200% 0" } },
        "token-caret": { "0%,100%": { opacity: "1" }, "50%": { opacity: "0" } },
      },
      animation: {
        breathe: "breathe 3s ease-in-out infinite",
        "pulse-ring": "pulse-ring 1.2s ease-out infinite",
        orbit: "orbit 1.6s linear infinite",
        shimmer: "shimmer 1.4s linear infinite",
        "token-caret": "token-caret 1s step-end infinite",
      },
    },
  },
};
```

**shadcn mapping:** `Button` → primary/secondary/ghost specs above; `Card` →
`{colors.surface}` + e1; `Popover`/`Tooltip` → e2; `Dialog` (settings/timeline) →
`{rounded.xl}`; `Badge` → cloud-indicator chip base; scrollbars styled to
`{colors.border-strong}`. Every custom component (panel, pill, overlay) wraps
shadcn primitives where a primitive exists.

## Agent Prompt Guide

Quick reference: canvas `#fbf7f1` · ink `#2b241e` · coral `#c55026` · sky `#dce9f2` ·
gold `#f8e3b7` · ember `#d16405` · green `#267326` · border `#e2dbd2` · radius 10/14/18 ·
panel 400px · shadow `0 8px 32px rgba(43,36,30,.16)`.

Prompts:
1. "Build the floating result panel per DESIGN.md `panel-shell`: 400px, radius 14,
   vibrancy with fallback, e3 shadow, header = orb + state label + cloud chip + Esc hint."
2. "Add the ptt-pill listening component: pill, ember live dot with pulse-ring,
   5 mic-level waveform bars, Nunito 'listening' label, clamped to screen edges."
3. "Implement the cloud-indicator chip with the three states offline/local-only/
   calling-cloud from DESIGN.md — icon + label + dot, event-driven, never color alone."

## Iteration Guide

1. Change tokens in front matter first, prose second — components always reference
   `{tokens}`, never raw values.
2. New variants are new entries (`-hover`, `-active`, `-error`), not prose.
3. After any edit, run the verification script (YAML parse, `{ref}` resolution,
   AA contrast) and the state-coverage check — both must pass before commit.
4. When adding a component, first ask whether the existing dawn vocabulary (one
   accent, soft radii, e1–e3, three voices) can express it. New tokens need a
   PRD/RFC trace, like everything in this file.
5. Keep the file under ~400 lines of front matter + core sections; split deep-dive
   specs into `docs/design/` if it grows.
