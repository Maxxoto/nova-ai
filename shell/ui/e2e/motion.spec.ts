import { mkdirSync, readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { expect, test, type Locator, type Page } from "@playwright/test";

import type { CanonicalState, NetState, PanelState } from "../src/components/panel/types";

/**
 * Motion harness for the Ruòxī UI (task T2; extended by T4–T12).
 *
 * Helpers are exported so later tasks assert the ported motion 1:1 against the
 * production build. Every assertion reads `getComputedStyle` from a real
 * Chromium page — never a mocked value.
 */

/** Directory `shot()` writes to; kept in git via `.gitkeep`. */
export const SCREENSHOT_DIR = resolve(dirname(fileURLToPath(import.meta.url)), "screenshots");

/** The computed motion values later tasks assert against. */
export type AnimationSnapshot = {
  animationName: string;
  animationDuration: string;
  animationTimingFunction: string;
  transitionDuration: string;
  transitionTimingFunction: string;
};

export type PanelMode = "panel" | "mini";

export type OpenPanelOptions = {
  mode?: PanelMode;
  state?: PanelState;
  net?: NetState;
};

const SURFACE_SELECTOR: Record<PanelMode, string> = {
  panel: ".panel-surface",
  mini: ".mini",
};

/** The root element of the requested panel surface (`.mini` lands with T4). */
export function panelSurface(page: Page, mode: PanelMode = "panel") {
  return page.locator(SURFACE_SELECTOR[mode]).first();
}

/**
 * Read the computed animation/transition values for `selector` from the live
 * DOM. `animationName` + `animationDuration` are what the per-state motion
 * assertions compare.
 */
export async function animationOf(page: Page, selector: string): Promise<AnimationSnapshot> {
  return page.locator(selector).first().evaluate((node) => {
    const style = getComputedStyle(node);
    return {
      animationName: style.animationName,
      animationDuration: style.animationDuration,
      animationTimingFunction: style.animationTimingFunction,
      transitionDuration: style.transitionDuration,
      transitionTimingFunction: style.transitionTimingFunction,
    };
  });
}

/** Emulate `prefers-reduced-motion: reduce`; call before navigating. */
export async function withReducedMotion(page: Page): Promise<void> {
  await page.emulateMedia({ reducedMotion: "reduce" });
}

/** Screenshot the current page into `e2e/screenshots/<name>.png`. */
export async function shot(page: Page, name: string): Promise<string> {
  mkdirSync(SCREENSHOT_DIR, { recursive: true });
  const file = resolve(SCREENSHOT_DIR, `${name}.png`);
  await page.screenshot({ path: file });
  return file;
}

/** Navigate the static dev harness to the panel surface and wait for it. */
export async function openPanel(page: Page, options: OpenPanelOptions = {}): Promise<void> {
  const mode = options.mode ?? "panel";
  const params = new URLSearchParams({ view: "panel", mode });
  if (options.state) params.set("state", options.state);
  if (options.net) params.set("net", options.net);
  await page.goto(`/?${params.toString()}`);
  await panelSurface(page, mode).waitFor({ state: "visible" });
}

test("smoke — the panel surface boots and renders at ?view=panel&state=thinking", async ({ page }) => {
  await openPanel(page, { state: "thinking" });

  const surface = panelSurface(page);
  await expect(surface).toBeVisible();
  await expect(surface).toHaveAttribute("data-state", "thinking");
  await expect(page.getByRole("group", { name: "Ruòxī result panel" })).toBeVisible();

  await shot(page, "smoke-panel-thinking");
});

/**
 * T8 — the tray surface. The review board (`/`) renders one `StateCard` per
 * `TRAY_STATES` entry; each card carries `data-tray-state` and renders a
 * `.tray-mark` whose `data-anim` comes from the registry's `anim`/`loopName`.
 * The listening mark must run `pulse-ring 1.2s` on its `.tr-live-ring` element;
 * idle and captures-paused stay static (no decorative loops).
 */
test.describe("tray — listening loop is live, idle and paused stay static", () => {
  test("listening mark pulses via pulse-ring, idle/paused render no animation", async ({ page }) => {
    await page.goto("/");

    const listeningMark = page.locator('[data-tray-state="listening"] .tray-mark').first();
    await expect(listeningMark).toHaveAttribute("data-anim", "pulse");

    const loop = await animationOf(page, '[data-tray-state="listening"] .tray-mark .tr-live-ring');
    expect(loop.animationName).toBe("pulse-ring");
    expect(loop.animationDuration).toBe("1.2s");

    const idleMark = page.locator('[data-tray-state="idle"] .tray-mark').first();
    expect(await idleMark.getAttribute("data-anim")).toBeNull();
    const idleAnim = await animationOf(page, '[data-tray-state="idle"] .tray-mark svg');
    expect(idleAnim.animationName).toBe("none");

    const pausedMark = page.locator('[data-tray-state="captures-paused"] .tray-mark').first();
    expect(await pausedMark.getAttribute("data-anim")).toBeNull();
    const pausedAnim = await animationOf(page, '[data-tray-state="captures-paused"] .tray-mark svg');
    expect(pausedAnim.animationName).toBe("none");
  });

  test("the gallery labels the listening loop as live", async ({ page }) => {
    await page.goto("/");

    await expect(page.locator('[data-tray-state="listening"]').first()).toContainText("live · pulse-ring");
    await expect(page.locator('[data-tray-state="idle"]').first()).not.toContainText("live ·");
    await expect(page.locator('[data-tray-state="captures-paused"]').first()).not.toContainText("live ·");
  });

  test("reduced motion freezes the listening loop", async ({ page }) => {
    await withReducedMotion(page);
    await page.goto("/");

    const listeningMark = page.locator('[data-tray-state="listening"] .tray-mark').first();
    await expect(listeningMark).toHaveAttribute("data-anim", "pulse");

    const loop = await animationOf(page, '[data-tray-state="listening"] .tray-mark .tr-live-ring');
    expect(loop.animationName).toBe("none");

    await expect(page.locator('[data-tray-state="listening"]').first()).toContainText("frozen · pulse-ring");
  });
});

/**
 * T9a — the panel / onboarding / overlay interactive-token sweep.
 *
 * Hovers + colour compute `--t-fast` 120ms with `--ease-standard`; toggles and
 * presses compute `--t-instant` 80ms; the CiteChip popover enters on `--ease-out`.
 * The last test is a source-level census so no off-token timing can creep back.
 */
const TOKEN_SWEEP_FILES = [
  "panel/PanelHeader.tsx",
  "panel/ReadAloudButton.tsx",
  "panel/SaveMemoryButton.tsx",
  "panel/AnswerStream.tsx",
  "panel/PanelBanner.tsx",
  "panel/PttPill.tsx",
  "onboarding/styles.ts",
  "onboarding/PermissionStep.tsx",
  "onboarding/PreferencesStep.tsx",
  "onboarding/ModelsStep.tsx",
  "onboarding/ReadyStep.tsx",
  "onboarding/FirstCaptureStep.tsx",
  "onboarding/StepRail.tsx",
  "onboarding/OnboardingRitual.tsx",
  "overlay/CaptureOverlay.tsx",
] as const;

test.describe("tokens-panel — panel / onboarding / overlay interactive motion", () => {
  test("a sampled hover control computes transition-duration 0.12s (--t-fast)", async ({ page }) => {
    await openPanel(page, { state: "complete" });

    const collapse = page.locator('[aria-label="Collapse to mini"]').first();
    await expect(collapse).toBeVisible();

    const snapshot = await animationOf(page, '[aria-label="Collapse to mini"]');
    expect(snapshot.transitionDuration).toBe("0.12s");
    expect(snapshot.transitionTimingFunction).toBe("cubic-bezier(0.4, 0, 0.2, 1)");
  });

  test("a sampled toggle computes transition-duration 0.08s (--t-instant)", async ({ page }) => {
    await page.goto("/?view=onboarding&step=preferences");

    const track = page.locator('[role="switch"]').first();
    await expect(track).toBeVisible();

    const trackStyle = await animationOf(page, '[role="switch"]');
    expect(trackStyle.transitionDuration).toBe("0.08s");

    const knobDuration = await track.locator("span").first().evaluate((node) => {
      return getComputedStyle(node).transitionDuration;
    });
    expect(knobDuration).toBe("0.08s");
  });

  test("no off-token ease-in-out / duration-150 / duration-200 remains in the swept files", () => {
    const componentsDir = resolve(dirname(fileURLToPath(import.meta.url)), "../src/components");
    for (const name of TOKEN_SWEEP_FILES) {
      const source = readFileSync(resolve(componentsDir, name), "utf8");
      expect(source, `${name} must not use ease-in-out`).not.toContain("ease-in-out");
      expect(source, `${name} must not use duration-150`).not.toContain("duration-150");
      expect(source, `${name} must not use a raw 150ms`).not.toContain("150ms");
      expect(source, `${name} must not use duration-200`).not.toContain("duration-200");
    }
  });
});

/* ------------------------------------------------------------------ */
/* T4 — the mark's four canonical states (idle · listening · thinking  */
/* · answered). Each finer `PanelState` collapses onto one, and the OD  */
/* mark rules fire off `data-state` (components.css:63-154, 539-548).   */
/* ------------------------------------------------------------------ */
test.describe("mini-mark — the four-state model", () => {
  test("mini-mark idle — the mark hops 2.8s on the canonical idle state", async ({ page }) => {
    await openPanel(page, { mode: "mini", state: "idle" });

    const surface = panelSurface(page, "mini");
    await expect(surface).toBeVisible();
    await expect(surface).toHaveAttribute("data-state", "idle");
    await expect(surface).toHaveAttribute("data-phase", "idle");

    const mark = await animationOf(page, ".mini[data-state='idle'] .mini-mark");
    expect(mark.animationName).toContain("mini-hop");
    expect(mark.animationDuration).toBe("2.8s");

    await shot(page, "mini-idle");
  });

  test("mini-mark listening — the disc ripples and the bars wave 0.9s", async ({ page }) => {
    await openPanel(page, { mode: "mini", state: "listening" });

    const surface = panelSurface(page, "mini");
    await expect(surface).toHaveAttribute("data-state", "listening");
    await expect(surface).toHaveAttribute("data-phase", "listening");

    // The OD ripple lives on `.mini-mark::before/::after`, so read the pseudo.
    const ripple = await page
      .locator(".mini[data-state='listening'] .mini-mark")
      .evaluate((node) => {
        const style = getComputedStyle(node, "::before");
        return { animationName: style.animationName, animationDuration: style.animationDuration };
      });
    expect(ripple.animationName).toContain("pulse-ring");
    expect(ripple.animationDuration).toBe("1.2s");

    const bars = await animationOf(page, ".mini-bars i");
    expect(bars.animationName).toContain("wave");
    expect(bars.animationDuration).toBe("0.9s");

    await shot(page, "mini-listening");
  });

  test("mini-mark thinking — the comet orbits 0.9s and the dots pulse", async ({ page }) => {
    await openPanel(page, { mode: "mini", state: "thinking" });

    const surface = panelSurface(page, "mini");
    await expect(surface).toHaveAttribute("data-state", "thinking");
    await expect(surface).toHaveAttribute("data-phase", "thinking");

    const spin = await animationOf(page, ".mini-spin");
    expect(spin.animationName).toContain("orbit");
    expect(spin.animationDuration.split(",")[0].trim()).toBe("0.9s");

    const dots = await animationOf(page, ".mini-dots i");
    expect(dots.animationName).toContain("dot-pulse");

    await shot(page, "mini-thinking");
  });

  test("mini-mark answered — the badge pops 0.28s and the bulb glows", async ({ page }) => {
    await openPanel(page, { mode: "mini", state: "complete" });

    const surface = panelSurface(page, "mini");
    await expect(surface).toHaveAttribute("data-state", "answered");
    await expect(surface).toHaveAttribute("data-phase", "complete");

    const badge = await animationOf(page, ".mini-badge");
    expect(badge.animationName).toContain("badge-pop");
    expect(badge.animationDuration).toBe("0.28s");

    const bulb = await animationOf(page, ".mini-badge svg");
    expect(bulb.animationName).toContain("bulb-flicker");
    expect(bulb.animationName).toContain("bulb-glow");

    await shot(page, "mini-answered");
  });
});

/** T9b — settings / models / timeline / shell token sweep: 120ms hovers, 80ms toggles. */
const SETTINGS_TOKEN_FILES = [
  "components/settings/primitives.tsx",
  "components/settings/SettingsWindow.tsx",
  "components/settings/AnchorPicker.tsx",
  "components/settings/ModelSetup.tsx",
  "components/models/ModelRows.tsx",
  "components/timeline/TimelineView.tsx",
  "App.tsx",
] as const;

const SETTINGS_SRC_DIR = resolve(dirname(fileURLToPath(import.meta.url)), "../src");

test.describe("tokens-settings — settings / models / timeline / shell interactive motion", () => {
  test("sampled settings + models controls compute the token durations", async ({ page }) => {
    await page.goto("/?view=settings");
    await expect(page.getByRole("button", { name: "Delete all…" })).toBeVisible();

    // hover / colour controls → --t-fast 120ms + --ease-standard
    const danger = await animationOf(page, 'button:has-text("Delete all…")');
    expect(danger.transitionDuration).toBe("0.12s");
    expect(danger.transitionTimingFunction).toBe("cubic-bezier(0.4, 0, 0.2, 1)");

    const secondary = await animationOf(page, 'button:has-text("Run setup again")');
    expect(secondary.transitionDuration).toBe("0.12s");

    // toggle → --t-instant 80ms on the track + knob; the outer hover stays 120ms
    const outer = await animationOf(page, '[role="switch"][aria-label="Offline mode"]');
    expect(outer.transitionDuration).toBe("0.12s");
    const track = await animationOf(page, '[role="switch"][aria-label="Offline mode"] span.relative');
    expect(track.transitionDuration).toBe("0.08s");
    const knob = await animationOf(page, '[role="switch"][aria-label="Offline mode"] span.absolute');
    expect(knob.transitionDuration).toBe("0.08s");

    // segmented selection (aria-pressed) → 80ms
    const segmented = await animationOf(page, 'button[aria-pressed]:has-text("Short")');
    expect(segmented.transitionDuration).toBe("0.08s");

    // theme swatch (aria-pressed) → 80ms
    const swatch = await animationOf(page, 'button[aria-pressed]:has-text("Dawn sky")');
    expect(swatch.transitionDuration).toBe("0.08s");

    // anchor picker (an aria-pressed press) → 80ms; reveal it via fixed placement
    await page.getByRole("button", { name: "Fixed position", exact: true }).click();
    const anchor = await animationOf(page, 'button[aria-label="Top left"]');
    expect(anchor.transitionDuration).toBe("0.08s");

    // models: the LLM config panel is rendered by ModelRows
    await page.getByRole("button", { name: "Configure…" }).click();
    const save = await animationOf(page, 'button:has-text("Save")');
    expect(save.transitionDuration).toBe("0.12s");

    // a focus-only input gained the token transition (it had none)
    await page.getByRole("button", { name: "Delete all…" }).click();
    const confirmDuration = await page
      .getByLabel("Confirmation")
      .evaluate((node) => getComputedStyle(node).transitionDuration);
    expect(confirmDuration).toBe("0.12s");
  });

  test("sampled timeline controls compute the token durations", async ({ page }) => {
    await page.goto("/?view=timeline&demo=1");
    await expect(page.getByRole("region", { name: "Capture timeline" })).toBeVisible();

    const card = await animationOf(page, "article");
    expect(card.transitionDuration).toBe("0.12s");

    const del = await animationOf(page, 'button:has-text("Delete all captures…")');
    expect(del.transitionDuration).toBe("0.12s");

    await page.getByRole("button", { name: "Delete all captures…" }).click();
    const confirmDuration = await page
      .getByLabel("Confirmation")
      .evaluate((node) => getComputedStyle(node).transitionDuration);
    expect(confirmDuration).toBe("0.12s");
  });

  test("sampled dev-board shell controls compute the token durations", async ({ page }) => {
    await page.goto("/");

    const chip = await animationOf(page, 'button:has-text("thinking")');
    expect(chip.transitionDuration).toBe("0.12s");

    const toggleOuter = await animationOf(page, 'button[role="switch"]');
    expect(toggleOuter.transitionDuration).toBe("0.12s");
    const toggleTrack = await animationOf(page, 'button[role="switch"] span.relative');
    expect(toggleTrack.transitionDuration).toBe("0.08s");
    const toggleKnob = await animationOf(page, 'button[role="switch"] span.absolute');
    expect(toggleKnob.transitionDuration).toBe("0.08s");

    await page.getByRole("button", { name: "dismiss (esc works too)" }).click();
    const show = await animationOf(page, 'button:has-text("show again")');
    expect(show.transitionDuration).toBe("0.12s");
  });

  test("no off-token ease or duration remains in the swept files", () => {
    for (const name of SETTINGS_TOKEN_FILES) {
      const source = readFileSync(resolve(SETTINGS_SRC_DIR, name), "utf8");
      expect(source, `${name} must not use ease-in-out`).not.toContain("ease-in-out");
      expect(source, `${name} must not use duration-150`).not.toContain("duration-150");
      expect(source, `${name} must not use duration-200`).not.toContain("duration-200");
    }
    // the determinate download fill keeps its linear mechanism on a token duration
    const modelRows = readFileSync(resolve(SETTINGS_SRC_DIR, "components/models/ModelRows.tsx"), "utf8");
    expect(modelRows).toContain("transition-[width] duration-[120ms] ease-linear");
  });
});

/* ------------------------------------------------------------------ */
/* T5 — the panel surface: rise-in on mount, the answered stagger, a   */
/* restrained error entrance (no loop) and the 120ms Esc fade. Numbers  */
/* are the OD's fixed per-state block (components.css:235, 539-563).    */
/* ------------------------------------------------------------------ */
test.describe("panel-states — entrance, answered stagger, error motion, reduced motion", () => {
  test("answered — canonical data-state and body children rise 260ms, 80ms apart", async ({ page }) => {
    await openPanel(page, { state: "complete" });

    const surface = panelSurface(page);
    await expect(surface).toHaveAttribute("data-state", "answered");
    await expect(surface).toHaveAttribute("data-phase", "complete");

    const first = await animationOf(page, ".panel-body > *:nth-child(1)");
    expect(first.animationName).toContain("block-rise");
    expect(first.animationDuration).toBe("0.26s");

    const second = await page
      .locator(".panel-body > *:nth-child(2)")
      .first()
      .evaluate((node) => {
        const style = getComputedStyle(node);
        return {
          animationName: style.animationName,
          animationDuration: style.animationDuration,
          animationDelay: style.animationDelay,
        };
      });
    expect(second.animationName).toContain("block-rise");
    expect(second.animationDuration).toBe("0.26s");
    expect(second.animationDelay).toBe("0.08s");

    await shot(page, "panel-state-answered");
  });

  test("thinking — a coherent thinking surface: three shimmering skeleton lines", async ({ page }) => {
    await openPanel(page, { state: "thinking" });

    const surface = panelSurface(page);
    await expect(surface).toHaveAttribute("data-state", "thinking");

    const shimmer = await page.locator(".panel-skeleton").first().evaluate((node) => {
      const style = getComputedStyle(node, "::after");
      return { animationName: style.animationName, animationDuration: style.animationDuration };
    });
    expect(shimmer.animationName).toContain("panel-shimmer");
    expect(shimmer.animationDuration).toBe("1.4s");
    await expect(page.locator(".panel-skeleton")).toHaveCount(3);

    await shot(page, "panel-state-thinking");
  });

  test("error — the banner rises 200ms and nothing in the panel loops", async ({ page }) => {
    await openPanel(page, { state: "error", net: "offline" });

    const banner = await animationOf(page, ".panel-error-banner");
    expect(banner.animationName).toContain("block-rise");
    expect(banner.animationDuration).toBe("0.2s");

    // The OD defines no error gesture: a one-shot entrance, the orb static,
    // and no element anywhere in the surface running an infinite loop.
    const loops = await page
      .locator(".panel-surface")
      .first()
      .evaluate((host) => {
        const found: string[] = [];
        host.querySelectorAll("*").forEach((el) => {
          const style = getComputedStyle(el);
          const infinite = style.animationName !== "none" &&
            style.animationIterationCount.split(",").some((v) => v.trim() === "infinite");
          if (infinite) found.push(`${el.tagName}.${el.className}:${style.animationName}`);
        });
        return found;
      });
    expect(loops, `looping elements: ${loops.join(", ")}`).toEqual([]);

    await shot(page, "panel-state-error");
  });

  test("reduced motion — the entrance, stagger and error motion all hold still", async ({ page }) => {
    await withReducedMotion(page);

    await openPanel(page, { state: "complete" });
    for (const selector of [".panel-surface", ".panel-body > *:nth-child(1)", ".panel-body > *:nth-child(2)"]) {
      const snapshot = await animationOf(page, selector);
      expect(snapshot.animationName, `${selector} under reduced motion`).toBe("none");
    }

    await openPanel(page, { state: "error", net: "offline" });
    const banner = await animationOf(page, ".panel-error-banner");
    expect(banner.animationName).toBe("none");

    await shot(page, "panel-state-reduced-motion");
  });
});

/* ------------------------------------------------------------------ */
/* T6 — one gesture per orb state, and `step-run` for the panel's       */
/* step dots. Loops run only while a real condition holds: breathe on   */
/* the resting states (idle/ask/complete), pulse-ring while listening,  */
/* orbit while thinking/transcribing, wave while speaking — and a static */
/* colour with no loop for error/degraded. The active step dot runs     */
/* `step-run 1.2s` (OD components.css:553-558) and differs from a done  */
/* dot by a NON-motion property so reduced motion stays legible.        */
/* ------------------------------------------------------------------ */
test.describe("orb-steps — one gesture per state", () => {
  test("idle — the orb body breathes 3s", async ({ page }) => {
    await openPanel(page, { state: "idle" });

    const orb = await animationOf(page, ".panel-surface .orb");
    expect(orb.animationName).toContain("breathe");
    expect(orb.animationDuration).toBe("3s");

    await shot(page, "orb-state-idle");
  });

  test("listening — the orb body is still and only the ring ripples 1.2s", async ({ page }) => {
    await openPanel(page, { state: "listening" });

    const body = await animationOf(page, ".panel-surface .orb");
    expect(body.animationName).toBe("none");

    const ring = await animationOf(page, ".panel-surface .orb-ring");
    expect(ring.animationName).toContain("pulse-ring");
    expect(ring.animationDuration).toBe("1.2s");

    await shot(page, "orb-state-listening");
  });

  test("thinking — the comet orbits 1.6s and the orb body does not breathe", async ({ page }) => {
    await openPanel(page, { state: "thinking" });

    const orbit = await animationOf(page, ".panel-surface .orb-orbit");
    expect(orbit.animationName).toContain("orbit");
    expect(orbit.animationDuration).toBe("1.6s");

    const body = await animationOf(page, ".panel-surface .orb");
    expect(body.animationName).not.toContain("breathe");
    expect(body.animationName).toBe("none");

    await shot(page, "orb-state-thinking");
  });

  test("speaking — the orb body waves 0.9s", async ({ page }) => {
    await openPanel(page, { state: "speaking" });

    const orb = await animationOf(page, ".panel-surface .orb");
    expect(orb.animationName).toContain("wave");
    expect(orb.animationDuration).toBe("0.9s");

    await shot(page, "orb-state-speaking");
  });

  test("error — the orb holds a static colour with no loop", async ({ page }) => {
    await openPanel(page, { state: "error", net: "offline" });

    const orb = await animationOf(page, ".panel-surface .orb");
    expect(orb.animationName).toBe("none");

    await shot(page, "orb-state-error");
  });

  test("thinking — the active step runs step-run 1.2s and differs from done without motion", async ({ page }) => {
    await openPanel(page, { state: "thinking" });

    const active = page.locator('.step-dot[data-step="active"]').first();
    const done = page.locator('.step-dot[data-step="done"]').first();
    await expect(active).toBeVisible();
    await expect(done).toBeVisible();

    const activeAnim = await active.evaluate((node) => {
      const style = getComputedStyle(node);
      return { animationName: style.animationName, animationDuration: style.animationDuration };
    });
    expect(activeAnim.animationName).toContain("step-run");
    expect(activeAnim.animationDuration).toBe("1.2s");

    // the OD staggers the three dots 150ms apart
    const secondDelay = await page.locator(".step-dot").nth(1).evaluate((node) => getComputedStyle(node).animationDelay);
    expect(secondDelay).toBe("0.15s");

    // reduced-motion contract: active vs done must differ by a non-animation property
    const activeShadow = await active.evaluate((node) => getComputedStyle(node).boxShadow);
    const doneShadow = await done.evaluate((node) => getComputedStyle(node).boxShadow);
    expect(await active.getAttribute("aria-current")).toBe("step");
    expect(activeShadow).not.toBe(doneShadow);

    await shot(page, "orb-steps-thinking");
  });
});

/* ------------------------------------------------------------------ */
/* T7 — the capture overlay. The dim enters in --t-slow (320ms) on      */
/* --ease-standard (DESIGN.md: overlay dim = 320ms); Esc fades the      */
/* in-webview surface out in --t-fast (120ms) before `overlay_cancel`   */
/* is invoked; and the drag selection stays instant — direct            */
/* manipulation never transitions (no transition on the selection rect, */
/* its handles or the dimension chip). Under reduced motion the dim     */
/* appears immediately, with no transform.                             */
/* ------------------------------------------------------------------ */

type InvokeCall = { cmd: string; at: number };

declare global {
  interface Window {
    __overlayInvokes?: InvokeCall[];
    __overlayLeavingAt?: number;
  }
}

test.describe("overlay-dim — the 320ms dim entrance and the 120ms Esc fade", () => {
  test("the dim enters with overlay-dim-in 0.32s on --ease-standard", async ({ page }) => {
    await page.goto("/?view=overlay");

    const dim = page.locator(".capture-dim").first();
    await expect(dim).toBeVisible();

    const snapshot = await animationOf(page, ".capture-dim");
    expect(snapshot.animationName).toContain("overlay-dim-in");
    expect(snapshot.animationDuration).toBe("0.32s");
    expect(snapshot.animationTimingFunction).toBe("cubic-bezier(0.4, 0, 0.2, 1)");
    // The dim also carries the 320ms opacity fade so the leaving class has a
    // 120ms transition to override (both read 0.32s on entrance).
    expect(snapshot.transitionDuration).toBe("0.32s");

    await shot(page, "overlay-dim-entrance");
  });

  test("a static selection keeps the dim entrance and the selection is instant", async ({ page }) => {
    await page.goto("/?view=overlay&sel=120,140,260,180");

    const dim = await animationOf(page, ".capture-dim");
    await expect(page.locator(".capture-dim").first()).toBeVisible();
    expect(dim.animationName).toContain("overlay-dim-in");
    expect(dim.animationDuration).toBe("0.32s");

    const sel = page.locator(".capture-sel").first();
    await expect(sel).toBeVisible();
    const selStyle = await sel.evaluate((node) => {
      const style = getComputedStyle(node);
      return { transitionDuration: style.transitionDuration, animationName: style.animationName };
    });
    expect(selStyle.transitionDuration).toBe("0s");
    expect(selStyle.animationName).toBe("none");

    const handleDuration = await sel
      .locator("span")
      .first()
      .evaluate((node) => getComputedStyle(node).transitionDuration);
    expect(handleDuration).toBe("0s");

    const chip = await animationOf(page, ".dim-chip");
    expect(chip.transitionDuration).toBe("0s");
    expect(chip.animationName).toBe("none");

    await shot(page, "overlay-dim-selection");
  });

  test("the selection rect never transitions while dragging", async ({ page }) => {
    await page.goto("/?view=overlay");

    const root = page.locator(".capture-overlay").first();
    const box = await root.boundingBox();
    expect(box).not.toBeNull();
    if (!box) return;

    await page.mouse.move(box.x + 60, box.y + 60);
    await page.mouse.down();
    await page.mouse.move(box.x + 240, box.y + 200, { steps: 10 });

    const sel = page.locator(".capture-sel").first();
    await expect(sel).toBeVisible();
    const during = await sel.evaluate((node) => {
      const style = getComputedStyle(node);
      return { transitionDuration: style.transitionDuration, animationName: style.animationName };
    });
    expect(during.transitionDuration).toBe("0s");
    expect(during.animationName).toBe("none");

    await page.mouse.up();
    await shot(page, "overlay-dim-dragging");
  });

  test("Esc shows the leaving state, then invokes overlay_cancel ~120ms later", async ({ page }) => {
    await page.addInitScript(() => {
      const calls: InvokeCall[] = [];
      const w = window as unknown as {
        __TAURI_INTERNALS__?: {
          invoke: (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;
          transformCallback?: () => number;
        };
        __overlayInvokes?: InvokeCall[];
        __overlayLeavingAt?: number;
      };
      w.__overlayInvokes = calls;
      w.__overlayLeavingAt = 0;
      w.__TAURI_INTERNALS__ = {
        invoke: (cmd) => {
          calls.push({ cmd, at: Date.now() });
          return Promise.resolve(null);
        },
        transformCallback: () => 1,
      };
      const observer = new MutationObserver(() => {
        if (!w.__overlayLeavingAt && document.querySelector(".capture-overlay.is-leaving")) {
          w.__overlayLeavingAt = Date.now();
        }
      });
      observer.observe(document, {
        subtree: true,
        attributes: true,
        attributeFilter: ["class"],
      });
    });

    await page.goto("/?view=overlay");
    const root = page.locator(".capture-overlay").first();
    await expect(root).toBeVisible();

    await page.keyboard.press("Escape");
    await expect(root).toHaveClass(/is-leaving/);

    const leavingDim = await page.locator(".capture-dim").first().evaluate((node) => {
      const style = getComputedStyle(node);
      return { transitionDuration: style.transitionDuration, opacity: style.opacity };
    });
    expect(leavingDim.transitionDuration).toBe("0.12s");

    await page.waitForFunction(() => (window.__overlayInvokes ?? []).some((c) => c.cmd === "overlay_cancel"));

    const calls = await page.evaluate(() => (window.__overlayInvokes ?? []).filter((c) => c.cmd === "overlay_cancel"));
    expect(calls).toHaveLength(1);

    // The leaving class is applied *before* the invoke, and the invoke waits out
    // the 120ms fade — not the other way round.
    const leavingAt = await page.evaluate(() => window.__overlayLeavingAt ?? 0);
    expect(leavingAt).toBeGreaterThan(0);
    expect(calls[0].at).toBeGreaterThan(leavingAt);
    expect(calls[0].at - leavingAt).toBeGreaterThanOrEqual(100);
    expect(calls[0].at - leavingAt).toBeLessThan(400);

    await shot(page, "overlay-dim-leaving");
  });

  test("reduced motion — the dim appears instantly with no transform", async ({ page }) => {
    await withReducedMotion(page);
    await page.goto("/?view=overlay&sel=120,140,260,180");

    const dim = await animationOf(page, ".capture-dim");
    expect(dim.animationName).toBe("none");
    expect(dim.animationDuration).toBe("0s");
    expect(dim.transitionDuration).toBe("0s");

    await shot(page, "overlay-dim-reduced-motion");
  });
});

/* ------------------------------------------------------------------ */
/* T11 — the reduced-motion correctness hinge.                          */
/*                                                                      */
/* The app has TWO reduced-motion switch paths: the OS media query      */
/* (`prefers-reduced-motion: reduce`) and the manual `.reduced-motion`  */
/* root class (the dev-board toggle + prop gating). BOTH must behave    */
/* identically — EVERY loop stops and EVERY transition collapses —      */
/* while the state stays legible from icon + colour + word without       */
/* motion.                                                              */
/* ------------------------------------------------------------------ */

const REDUCED_PANEL_STATES: PanelState[] = [
  "idle",
  "ask",
  "listening",
  "transcribing",
  "thinking",
  "speaking",
  "streaming",
  "complete",
  "error",
  "degraded",
];

const REDUCED_MINI_STATES: PanelState[] = ["idle", "listening", "thinking", "complete"];

/**
 * Walk `root` + every descendant and return every element still running motion
 * under reduced motion: an animation with a name whose duration is > 0.01s or
 * that loops forever, or a transition whose longest duration is > 0.01s. The
 * contract is an empty array.
 */
async function motionViolations(root: Locator): Promise<string[]> {
  return root.evaluate((host) => {
    const offenders: string[] = [];
    const visit = (el: Element) => {
      const style = getComputedStyle(el);
      const names = style.animationName.split(",").map((v) => v.trim());
      const durations = style.animationDuration.split(",").map((v) => v.trim());
      const iterations = style.animationIterationCount.split(",").map((v) => v.trim());
      names.forEach((name, i) => {
        if (name === "" || name === "none") return;
        const duration = Number.parseFloat(durations[i % durations.length] ?? "0") || 0;
        const iteration = iterations[i % iterations.length] ?? "1";
        if (duration > 0.01 || iteration === "infinite") {
          offenders.push(`animation ${el.tagName}.${el.className} → ${name} ${duration}s ${iteration}`);
        }
      });
      const maxTransition = Math.max(
        0,
        ...style.transitionDuration.split(",").map((v) => Number.parseFloat(v) || 0),
      );
      if (maxTransition > 0.01) {
        offenders.push(`transition ${el.tagName}.${el.className} → ${maxTransition}s`);
      }
    };
    visit(host);
    host.querySelectorAll("*").forEach(visit);
    return offenders;
  });
}

async function pseudoAnimation(
  page: Page,
  selector: string,
  pseudo: "::before" | "::after",
): Promise<{ animationName: string; animationDuration: string }> {
  return page.locator(selector).first().evaluate((node, which) => {
    const style = getComputedStyle(node, which as string);
    return { animationName: style.animationName, animationDuration: style.animationDuration };
  }, pseudo);
}

async function enableManualReducedMotion(page: Page): Promise<void> {
  await page.evaluate(() => document.documentElement.classList.add("reduced-motion"));
}

test.describe("reduced-motion — the blanket reset behaves identically on both switch paths", () => {
  test("OS query: every panel state stops every loop and transition", async ({ page }) => {
    await withReducedMotion(page);
    for (const state of REDUCED_PANEL_STATES) {
      await openPanel(page, { state });
      const surface = panelSurface(page);
      await expect(surface).toHaveClass(/reduced-motion/);
      expect(await motionViolations(surface), `OS reduce · panel ${state}`).toEqual([]);
    }
    await openPanel(page, { state: "thinking" });
    await shot(page, "reduced-motion-panel-thinking");
  });

  test("manual .reduced-motion root class: every panel state stops every loop and transition", async ({ page }) => {
    for (const state of REDUCED_PANEL_STATES) {
      await openPanel(page, { state });
      await enableManualReducedMotion(page);
      expect(await motionViolations(panelSurface(page)), `manual reduce · panel ${state}`).toEqual([]);
    }
    await enableManualReducedMotion(page);
    await shot(page, "reduced-motion-manual-panel");
  });

  test("OS query: every mini state stops every loop and transition", async ({ page }) => {
    await withReducedMotion(page);
    for (const state of REDUCED_MINI_STATES) {
      await openPanel(page, { mode: "mini", state });
      const mark = panelSurface(page, "mini");
      await expect(mark).toHaveClass(/reduced-motion/);
      expect(await motionViolations(mark), `OS reduce · mini ${state}`).toEqual([]);
    }
    await openPanel(page, { mode: "mini", state: "listening" });
    await shot(page, "reduced-motion-mini-listening");
  });

  test("manual .reduced-motion root class: every mini state stops every loop and transition", async ({ page }) => {
    for (const state of REDUCED_MINI_STATES) {
      await openPanel(page, { mode: "mini", state });
      await enableManualReducedMotion(page);
      expect(await motionViolations(panelSurface(page, "mini")), `manual reduce · mini ${state}`).toEqual([]);
    }
  });

  test("both paths: the hand-listed loop carriers report animationName:none", async ({ page }) => {
    const carriers: { state: PanelState; selector: string }[] = [
      { state: "idle", selector: ".orb" },
      { state: "listening", selector: ".orb-ring" },
      { state: "listening", selector: ".ptt-wave i" },
      { state: "thinking", selector: ".orb-orbit" },
      { state: "thinking", selector: ".step-dot" },
      { state: "streaming", selector: ".panel-caret" },
      { state: "complete", selector: ".panel-body > *" },
    ];

    await withReducedMotion(page);
    for (const { state, selector } of carriers) {
      await openPanel(page, { state });
      const snapshot = await animationOf(page, selector);
      expect(snapshot.animationName, `OS reduce · ${state} ${selector}`).toBe("none");
    }

    await page.emulateMedia({ reducedMotion: "no-preference" });
    for (const { state, selector } of carriers) {
      await openPanel(page, { state });
      await enableManualReducedMotion(page);
      const snapshot = await animationOf(page, selector);
      expect(snapshot.animationName, `manual reduce · ${state} ${selector}`).toBe("none");
    }
  });

  test("OS query + manual toggle freeze the tray listening loop and keep the mark visible", async ({ page }) => {
    await withReducedMotion(page);
    await page.goto("/");
    const mark = page.locator('[data-tray-state="listening"] .tray-mark').first();
    expect(await motionViolations(mark), "OS reduce · tray").toEqual([]);
    const osBox = await mark.locator("svg").first().boundingBox();
    expect(osBox?.width ?? 0).toBeGreaterThan(0);
    await shot(page, "reduced-motion-tray-os");

    await page.emulateMedia({ reducedMotion: "no-preference" });
    await page.goto("/");
    await page.getByRole("switch").first().click();
    await expect(page.locator('[data-tray-state="listening"]').first()).toContainText("frozen · pulse-ring");
    expect(await motionViolations(mark), "manual reduce · tray").toEqual([]);
    const manualBox = await mark.locator("svg").first().boundingBox();
    expect(manualBox?.width ?? 0).toBeGreaterThan(0);
    await shot(page, "reduced-motion-tray-manual");
  });

  test("OS query: the overlay honours reduced motion through the prop path", async ({ page }) => {
    await withReducedMotion(page);
    await page.goto("/?view=overlay&sel=120,140,260,180");

    const overlay = page.locator(".capture-overlay").first();
    await expect(overlay).toHaveClass(/reduced-motion/);
    expect(await motionViolations(overlay), "OS reduce · overlay").toEqual([]);

    const dim = await animationOf(page, ".capture-dim");
    expect(dim.animationName).toBe("none");
    expect(dim.animationDuration).toBe("0s");
    expect(dim.transitionDuration).toBe("0s");
    await shot(page, "reduced-motion-overlay");
  });

  test("manual .reduced-motion root class also stills the overlay", async ({ page }) => {
    await page.goto("/?view=overlay&sel=120,140,260,180");
    await enableManualReducedMotion(page);

    const overlay = page.locator(".capture-overlay").first();
    expect(await motionViolations(overlay), "manual reduce · overlay").toEqual([]);

    const dim = await animationOf(page, ".capture-dim");
    expect(dim.animationName).toBe("none");
    expect(dim.transitionDuration).toBe("0s");
  });

  test("legibility: the active step differs from done by a non-animation property", async ({ page }) => {
    await withReducedMotion(page);
    await openPanel(page, { state: "thinking" });

    const active = page.locator('.step-dot[data-step="active"]').first();
    const done = page.locator('.step-dot[data-step="done"]').first();
    await expect(active).toBeVisible();
    await expect(done).toBeVisible();

    const read = (locator: Locator) =>
      locator.evaluate((node) => {
        const style = getComputedStyle(node);
        return {
          animationName: style.animationName,
          boxShadow: style.boxShadow,
          backgroundColor: style.backgroundColor,
          outlineWidth: style.outlineWidth,
        };
      });

    const [activeStyle, doneStyle] = await Promise.all([read(active), read(done)]);
    expect(activeStyle.animationName).toBe("none");
    expect(await active.getAttribute("aria-current")).toBe("step");
    const differs =
      activeStyle.boxShadow !== doneStyle.boxShadow ||
      activeStyle.backgroundColor !== doneStyle.backgroundColor ||
      activeStyle.outlineWidth !== doneStyle.outlineWidth;
    expect(differs, `active ${JSON.stringify(activeStyle)} vs done ${JSON.stringify(doneStyle)}`).toBe(true);

    await shot(page, "reduced-motion-active-step");
  });

  test("legibility: listening keeps a static ring with motion off", async ({ page }) => {
    await withReducedMotion(page);
    await openPanel(page, { state: "listening" });

    const ring = await page.locator(".panel-surface .orb-ring").first().evaluate((node) => {
      const style = getComputedStyle(node);
      return {
        display: style.display,
        opacity: style.opacity,
        borderTopWidth: style.borderTopWidth,
        animationName: style.animationName,
      };
    });
    expect(ring.display).not.toBe("none");
    expect(Number.parseFloat(ring.opacity)).toBeGreaterThan(0);
    expect(Number.parseFloat(ring.borderTopWidth)).toBeGreaterThan(0);
    expect(ring.animationName).toBe("none");

    const pttRing = await pseudoAnimation(page, ".ptt-dot", "::after");
    expect(pttRing.animationName).toBe("none");
    await shot(page, "reduced-motion-listening-ring");
  });

  test("legibility: the thinking skeleton keeps a static muted fill (no sweep)", async ({ page }) => {
    await withReducedMotion(page);
    await openPanel(page, { state: "thinking" });

    const skeleton = page.locator(".panel-skeleton").first();
    await expect(skeleton).toBeVisible();
    const sweep = await skeleton.evaluate((node) => {
      const style = getComputedStyle(node, "::after");
      return { display: style.display, animationName: style.animationName };
    });
    expect(sweep.animationName).toBe("none");
    expect(sweep.display).toBe("none");

    const fill = await skeleton.evaluate((node) => getComputedStyle(node).backgroundColor);
    expect(fill).not.toBe("rgba(0, 0, 0, 0)");
    await shot(page, "reduced-motion-skeleton");
  });

  test("legibility: the streaming caret stays a static bar", async ({ page }) => {
    await withReducedMotion(page);
    await openPanel(page, { state: "streaming" });

    const caret = await page.locator(".panel-caret").first().evaluate((node) => {
      const style = getComputedStyle(node);
      return {
        animationName: style.animationName,
        opacity: style.opacity,
        width: style.width,
        backgroundColor: style.backgroundColor,
      };
    });
    expect(caret.animationName).toBe("none");
    expect(Number.parseFloat(caret.opacity)).toBeGreaterThan(0);
    expect(Number.parseFloat(caret.width)).toBeGreaterThan(0);
    expect(caret.backgroundColor).not.toBe("rgba(0, 0, 0, 0)");
  });

  test("legibility: thinking leaves a static comet/dots marker", async ({ page }) => {
    await withReducedMotion(page);
    await openPanel(page, { mode: "mini", state: "thinking" });

    const spin = await page.locator(".mini-spin").first().evaluate((node) => {
      const style = getComputedStyle(node);
      return {
        animationName: style.animationName,
        borderTopWidth: style.borderTopWidth,
        borderTopColor: style.borderTopColor,
      };
    });
    expect(spin.animationName).toBe("none");
    expect(Number.parseFloat(spin.borderTopWidth)).toBeGreaterThan(0);
    expect(spin.borderTopColor).not.toBe("rgba(0, 0, 0, 0)");

    const dotOpacity = await page.locator(".mini-dots i").first().evaluate((node) => getComputedStyle(node).opacity);
    expect(Number.parseFloat(dotOpacity)).toBeGreaterThan(0);
    await shot(page, "reduced-motion-mini-thinking");
  });

  test("legibility: the answered bulb keeps its final static brightness", async ({ page }) => {
    await withReducedMotion(page);
    await openPanel(page, { mode: "mini", state: "complete" });

    const badge = page.locator(".mini-badge").first();
    await expect(badge).toBeVisible();
    const box = await badge.boundingBox();
    expect(box?.width ?? 0).toBeGreaterThan(0);

    const bulb = await badge.locator("svg").first().evaluate((node) => {
      const style = getComputedStyle(node);
      return { animationName: style.animationName, opacity: style.opacity };
    });
    expect(bulb.animationName).toBe("none");
    expect(Number.parseFloat(bulb.opacity)).toBeGreaterThan(0.5);

    const bloom = await page.locator(".mini-badge").first().evaluate((node) => getComputedStyle(node, "::after").display);
    expect(bloom).toBe("none");
    await shot(page, "reduced-motion-mini-answered");
  });

  test("restoration: the onboarding ritual step keeps its 240ms reduced-motion fade", async ({ page }) => {
    await withReducedMotion(page);
    await page.goto("/?view=onboarding");

    const step = page.locator(".ritual-step").first();
    await expect(step).toBeVisible();
    const snapshot = await animationOf(page, ".ritual-step");
    expect(snapshot.animationName).toContain("ritual-step-fade");
    expect(snapshot.animationDuration).toBe("0.24s");
  });
});

/* ------------------------------------------------------------------ */
/* T11 — the reset must FAIL CLOSED: an element that animates today    */
/* but sits OUTSIDE every previously hand-listed scope (`.tray-mark`,  */
/* `.mini`, `.panel-surface`, `.orb`) must still report `animationName: none` */
/* or a ≤0.01s single iteration, and ≤0.01s transitions, on BOTH the   */
/* OS query and the manual `.reduced-motion` class — while the         */
/* restoration signals stay visible.                                   */
/* ------------------------------------------------------------------ */
test.describe("reduced-motion — fail-closed outside the previously-listed scopes", () => {
  type MotionState = { name: string; duration: number; iteration: string; transition: number };

  async function motionState(page: Page, selector: string): Promise<MotionState> {
    return page.locator(selector).first().evaluate((node) => {
      const style = getComputedStyle(node);
      const duration = Number.parseFloat(style.animationDuration.split(",")[0] ?? "0") || 0;
      const iteration = style.animationIterationCount.split(",")[0]?.trim() ?? "1";
      const transition = Math.max(
        0,
        ...style.transitionDuration.split(",").map((v) => Number.parseFloat(v) || 0),
      );
      return { name: style.animationName, duration, iteration, transition };
    });
  }

  function expectStilled(state: MotionState, label: string): void {
    const stillAnimating =
      state.name !== "none" && (state.duration > 0.01 || state.iteration === "infinite");
    expect(stillAnimating, `${label} → ${JSON.stringify(state)}`).toBe(false);
    expect(state.transition, `${label} transition → ${state.transition}s`).toBeLessThanOrEqual(0.01);
  }

  test("manual path: an unlisted Tailwind animate-* consumer (calling-cloud dot) stills", async ({ page }) => {
    await openPanel(page, { state: "streaming", net: "calling_cloud" });
    await expect(page.locator(".animate-pulse-ring").first()).toBeVisible();
    await enableManualReducedMotion(page);
    expectStilled(await motionState(page, ".animate-pulse-ring"), "manual · cloud dot");
  });

  test("OS path: settings controls outside the listed scopes still", async ({ page }) => {
    await withReducedMotion(page);
    await page.goto("/?view=settings");
    await expect(page.getByRole("button", { name: "Delete all…" })).toBeVisible();
    expectStilled(await motionState(page, '[role="switch"][aria-label="Offline mode"]'), "OS · settings toggle");
    expectStilled(await motionState(page, 'button:has-text("Delete all…")'), "OS · settings button");
  });

  test("manual path: settings controls outside the listed scopes still", async ({ page }) => {
    await page.goto("/?view=settings");
    await expect(page.getByRole("button", { name: "Delete all…" })).toBeVisible();
    await enableManualReducedMotion(page);
    expectStilled(await motionState(page, '[role="switch"][aria-label="Offline mode"]'), "manual · settings toggle");
    expectStilled(await motionState(page, 'button:has-text("Delete all…")'), "manual · settings button");
  });

  test("the restoration signals survive on both paths", async ({ page }) => {
    await withReducedMotion(page);
    await openPanel(page, { state: "thinking" });
    const skeleton = page.locator(".panel-skeleton").first();
    await expect(skeleton).toBeVisible();
    expect(await skeleton.evaluate((node) => getComputedStyle(node, "::after").display)).toBe("none");
    expect(await skeleton.evaluate((node) => getComputedStyle(node).backgroundColor)).not.toBe("rgba(0, 0, 0, 0)");

    await openPanel(page, { state: "streaming" });
    const caret = await page.locator(".panel-caret").first().evaluate((node) => getComputedStyle(node).opacity);
    expect(Number.parseFloat(caret)).toBeGreaterThan(0);

    await openPanel(page, { state: "listening" });
    const ring = await page.locator(".panel-surface .orb-ring").first().evaluate((node) => {
      const style = getComputedStyle(node);
      return { display: style.display, opacity: style.opacity };
    });
    expect(ring.display).not.toBe("none");
    expect(Number.parseFloat(ring.opacity)).toBeGreaterThan(0);

    await openPanel(page, { mode: "mini", state: "complete" });
    const bulb = await page.locator(".mini-badge svg").first().evaluate((node) => getComputedStyle(node).opacity);
    expect(Number.parseFloat(bulb)).toBeGreaterThan(0.5);
  });
});

/* ------------------------------------------------------------------ */
/* T12 — the exhaustive state × mode × reduced-motion matrix.          */
/*                                                                     */
/* Every cell of `mode ∈ {panel, mini}` × the ten `PanelState`s        */
/* asserts the OD's exact computed motion for that cell: the canonical */
/* four-state collapse (capture-and-ask.frag:440), the mini rules      */
/* (components.css:63-154) and the fixed per-state block               */
/* (components.css:539-563). The reduced-motion column then re-runs    */
/* both switch paths (the OS `prefers-reduced-motion` query and the    */
/* manual `.reduced-motion` root class) over every cell: every         */
/* animation must be `none` or a ≤0.01s single shot, every transition  */
/* ≤0.01s, and the state stays legible from static icon/colour/word.   */
/* One screenshot per state/mode plus the overlay is the artifact.     */
/* ------------------------------------------------------------------ */

/** The ten dev-harness phases, in canonical order. */
const MATRIX_STATES: readonly PanelState[] = REDUCED_PANEL_STATES;

/** The OD canonical collapse (capture-and-ask.frag:440) — asserted, never derived. */
const EXPECTED_CANONICAL: Record<PanelState, CanonicalState> = {
  idle: "idle",
  ask: "idle",
  listening: "listening",
  transcribing: "thinking",
  thinking: "thinking",
  speaking: "answered",
  streaming: "thinking",
  complete: "answered",
  error: "idle",
  degraded: "idle",
};

/** The mini mark's legible word per phase (design `LABEL`, lower-cased). */
const MINI_WORD: Record<PanelState, string> = {
  idle: "ready",
  ask: "ready",
  listening: "listening",
  transcribing: "transcribing",
  thinking: "thinking",
  speaking: "speaking",
  streaming: "writing",
  complete: "complete",
  error: "error",
  degraded: "resting",
};

type MotionCheck = {
  /** Human label used in the failure message. */
  label: string;
  selector: string;
  pseudo?: "::before" | "::after";
  /** `"none"` = exactly none; a string = contained; an array = every entry contained. */
  name: "none" | string | readonly string[];
  /** Exact first `animationDuration` (e.g. `"0.9s"`). */
  duration?: string;
  /** Exact first `animationDelay`. */
  delay?: string;
  /** Exact first `animationTimingFunction`. */
  timing?: string;
  /** The first animation must (true) / must not (false) loop forever. */
  infinite?: boolean;
};

type MatrixCell = {
  checks: readonly MotionCheck[];
  /** Selectors that must NOT match in this cell — an absence is part of the spec. */
  absent?: readonly string[];
  /** When true the whole surface must contain no infinite animation. */
  loopFree?: boolean;
};

/** Split a computed CSS list on commas that are NOT inside `cubic-bezier(...)`. */
function splitCommaList(value: string): string[] {
  const parts: string[] = [];
  let depth = 0;
  let current = "";
  for (const char of value) {
    if (char === "(") depth += 1;
    if (char === ")") depth -= 1;
    if (char === "," && depth === 0) {
      parts.push(current.trim());
      current = "";
      continue;
    }
    current += char;
  }
  parts.push(current.trim());
  return parts;
}

/** Read one computed motion check off the live DOM; failures carry the exact values. */
async function expectMotion(page: Page, check: MotionCheck): Promise<void> {
  const snapshot = await page.locator(check.selector).first().evaluate((node, pseudo) => {
    const style = getComputedStyle(node, pseudo);
    return {
      name: style.animationName,
      durations: style.animationDuration,
      delays: style.animationDelay,
      iterations: style.animationIterationCount,
      timings: style.animationTimingFunction,
    };
  }, check.pseudo);
  const durations = splitCommaList(snapshot.durations);
  const delays = splitCommaList(snapshot.delays);
  const iterations = splitCommaList(snapshot.iterations);
  const timings = splitCommaList(snapshot.timings);
  const where = `${check.label} — ${check.selector}${check.pseudo ?? ""}`;
  if (check.name === "none") {
    expect(snapshot.name, `${where} animation-name`).toBe("none");
  } else if (Array.isArray(check.name)) {
    for (const part of check.name) {
      expect(snapshot.name, `${where} animation-name`).toContain(part);
    }
  } else {
    expect(snapshot.name, `${where} animation-name`).toContain(check.name);
  }
  if (check.duration !== undefined) {
    expect(durations[0], `${where} animation-duration`).toBe(check.duration);
  }
  if (check.delay !== undefined) {
    expect(delays[0], `${where} animation-delay`).toBe(check.delay);
  }
  if (check.timing !== undefined) {
    expect(timings[0], `${where} animation-timing-function`).toBe(check.timing);
  }
  if (check.infinite !== undefined) {
    expect(iterations.includes("infinite"), `${where} iteration-count ${iterations.join(", ")}`).toBe(check.infinite);
  }
}

/* The mini mark's four canonical gestures, shared across the phases that
   collapse onto them (CANONICAL_STATE). `error`/`degraded` collapse onto
   idle, so the mini mark hops there: the OD four-state model has no error
   mark, and the panel orb is the surface that holds its static colour. */
const MINI_HOP: MotionCheck = {
  label: "the mark hops",
  selector: ".mini .mini-mark",
  name: "mini-hop",
  duration: "2.8s",
  timing: "cubic-bezier(0.2, 1.05, 0.3, 1)",
  infinite: true,
};
const MINI_COMET: MotionCheck = {
  label: "the comet orbits",
  selector: ".mini .mini-spin",
  name: ["orbit", "fade-in"],
  duration: "0.9s",
  timing: "linear",
  infinite: true,
};
const MINI_DOTS: MotionCheck = {
  label: "the dots pulse",
  selector: ".mini .mini-dots i",
  name: "dot-pulse",
  duration: "1.35s",
  infinite: true,
};
const MINI_DOT_STAGGER: MotionCheck = {
  label: "the second dot is 180ms behind",
  selector: ".mini .mini-dots i:nth-child(2)",
  name: "dot-pulse",
  duration: "1.35s",
  delay: "0.18s",
  infinite: true,
};
const MINI_BADGE: MotionCheck = {
  label: "the badge pops",
  selector: ".mini .mini-badge",
  name: "badge-pop",
  duration: "0.28s",
  timing: "cubic-bezier(0.2, 1.05, 0.3, 1)",
  infinite: false,
};
const MINI_BULB: MotionCheck = {
  label: "the bulb flickers on, then glows",
  selector: ".mini .mini-badge svg",
  name: ["bulb-flicker", "bulb-glow"],
  duration: "0.52s",
  delay: "0s",
  infinite: true,
};
const MINI_BLOOM: MotionCheck = {
  label: "the answered bloom widens",
  selector: ".mini .mini-badge",
  pseudo: "::after",
  name: "ring-widen",
  duration: "2.8s",
  delay: "0.62s",
  infinite: true,
};

const PANEL_MATRIX: Record<PanelState, MatrixCell> = {
  idle: {
    checks: [
      { label: "the orb breathes", selector: ".panel-surface .orb", name: "breathe", duration: "3s", infinite: true },
    ],
  },
  ask: {
    checks: [
      { label: "the orb breathes", selector: ".panel-surface .orb", name: "breathe", duration: "3s", infinite: true },
    ],
  },
  listening: {
    checks: [
      { label: "the orb body holds still", selector: ".panel-surface .orb", name: "none" },
      { label: "the orb ring ripples", selector: ".panel-surface .orb-ring", name: "pulse-ring", duration: "1.2s", infinite: true },
      {
        label: "the live pill ring ripples",
        selector: ".ptt.is-live .ptt-dot",
        pseudo: "::after",
        name: "pulse-ring",
        duration: "1.2s",
        infinite: true,
      },
      {
        label: "the live pill wave waves",
        selector: ".ptt.is-live .ptt-wave i",
        name: "wave",
        duration: "0.9s",
        infinite: true,
      },
    ],
  },
  transcribing: {
    checks: [
      { label: "the orb body holds still", selector: ".panel-surface .orb", name: "none" },
      { label: "the comet orbits", selector: ".panel-surface .orb-orbit", name: "orbit", duration: "1.6s", infinite: true },
    ],
    // Transcribing shows the transcript; the step dots belong to thinking/streaming.
    absent: [".panel-surface .step-dot"],
  },
  thinking: {
    checks: [
      { label: "the orb body holds still", selector: ".panel-surface .orb", name: "none" },
      { label: "the comet orbits", selector: ".panel-surface .orb-orbit", name: "orbit", duration: "1.6s", infinite: true },
      { label: "the step dots run", selector: ".panel-surface .step-dot", name: "step-run", duration: "1.2s", infinite: true },
      {
        label: "the second step dot is 150ms behind",
        selector: ".panel-surface .step-dot:nth-child(2)",
        name: "step-run",
        duration: "1.2s",
        delay: "0.15s",
        infinite: true,
      },
      {
        label: "the skeleton shimmers",
        selector: ".panel-surface .panel-skeleton",
        pseudo: "::after",
        name: "panel-shimmer",
        duration: "1.4s",
        infinite: true,
      },
    ],
  },
  speaking: {
    checks: [{ label: "the orb waves", selector: ".panel-surface .orb", name: "wave", duration: "0.9s", infinite: true }],
  },
  streaming: {
    checks: [
      { label: "the orb body holds still", selector: ".panel-surface .orb", name: "none" },
      { label: "the caret blinks", selector: ".panel-surface .panel-caret", name: "token-caret", duration: "1s", infinite: true },
      { label: "the step dots run", selector: ".panel-surface .step-dot", name: "step-run", duration: "1.2s", infinite: true },
      {
        label: "the second step dot is 150ms behind",
        selector: ".panel-surface .step-dot:nth-child(2)",
        name: "step-run",
        duration: "1.2s",
        delay: "0.15s",
        infinite: true,
      },
    ],
  },
  complete: {
    checks: [
      { label: "the orb breathes", selector: ".panel-surface .orb", name: "breathe", duration: "3s", infinite: true },
      {
        label: "the first answer block rises",
        selector: ".panel-surface .panel-body > *:nth-child(1)",
        name: "block-rise",
        duration: "0.26s",
        infinite: false,
      },
      {
        label: "the second answer block rises 80ms later",
        selector: ".panel-surface .panel-body > *:nth-child(2)",
        name: "block-rise",
        duration: "0.26s",
        delay: "0.08s",
        infinite: false,
      },
      {
        label: "the third answer block rises 160ms later",
        selector: ".panel-surface .panel-body > *:nth-child(3)",
        name: "block-rise",
        duration: "0.26s",
        delay: "0.16s",
        infinite: false,
      },
    ],
  },
  error: {
    checks: [
      { label: "the orb holds a static colour", selector: ".panel-surface .orb", name: "none" },
      {
        label: "the error banner rises once",
        selector: ".panel-surface .panel-error-banner",
        name: "block-rise",
        duration: "0.2s",
        infinite: false,
      },
    ],
    loopFree: true,
  },
  degraded: {
    checks: [{ label: "the orb holds a static colour", selector: ".panel-surface .orb", name: "none" }],
    loopFree: true,
  },
};

const MINI_MATRIX: Record<PanelState, MatrixCell> = {
  // idle / ask / error / degraded all collapse onto canonical idle (mini-hop).
  idle: { checks: [MINI_HOP] },
  ask: { checks: [MINI_HOP] },
  listening: {
    checks: [
      { label: "the disc breathes", selector: ".mini .mini-mark", name: "breathe", duration: "1.2s", infinite: true },
      {
        label: "the first half-beat ripple",
        selector: ".mini .mini-mark",
        pseudo: "::before",
        name: "pulse-ring",
        duration: "1.2s",
        infinite: true,
      },
      {
        label: "the second half-beat ripple",
        selector: ".mini .mini-mark",
        pseudo: "::after",
        name: "pulse-ring",
        duration: "1.2s",
        infinite: true,
      },
      { label: "the bars wave", selector: ".mini .mini-bars i", name: "wave", duration: "0.9s", infinite: true },
    ],
  },
  // transcribing / thinking / streaming all collapse onto canonical thinking.
  transcribing: { checks: [MINI_COMET, MINI_DOTS, MINI_DOT_STAGGER] },
  thinking: { checks: [MINI_COMET, MINI_DOTS, MINI_DOT_STAGGER] },
  streaming: { checks: [MINI_COMET, MINI_DOTS, MINI_DOT_STAGGER] },
  // speaking / complete collapse onto canonical answered.
  speaking: { checks: [MINI_BADGE, MINI_BULB, MINI_BLOOM] },
  complete: { checks: [MINI_BADGE, MINI_BULB, MINI_BLOOM] },
  error: { checks: [MINI_HOP] },
  degraded: { checks: [MINI_HOP] },
};

/** Navigate a cell, assert `data-state`/`data-phase`, every motion check and absences. */
async function assertMatrixCell(page: Page, mode: PanelMode, state: PanelState, cell: MatrixCell): Promise<void> {
  await openPanel(page, { mode, state });
  const surface = panelSurface(page, mode);
  await expect(surface).toBeVisible();
  await expect(surface).toHaveAttribute("data-state", EXPECTED_CANONICAL[state]);
  await expect(surface).toHaveAttribute("data-phase", state);
  for (const check of cell.checks) await expectMotion(page, check);
  for (const selector of cell.absent ?? []) {
    expect(await page.locator(selector).count(), `${mode}/${state} must not render ${selector}`).toBe(0);
  }
  if (cell.loopFree) {
    const loops = await surface.evaluate((host) => {
      const found: string[] = [];
      const visit = (el: Element) => {
        const style = getComputedStyle(el);
        if (
          style.animationName !== "none" &&
          style.animationIterationCount.split(",").some((value) => value.trim() === "infinite")
        ) {
          found.push(`${el.tagName}.${String(el.className)} → ${style.animationName}`);
        }
      };
      visit(host);
      host.querySelectorAll("*").forEach(visit);
      return found;
    });
    expect(loops, `${mode}/${state} looping elements: ${loops.join(", ")}`).toEqual([]);
  }
}

