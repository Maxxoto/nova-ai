/** Shared Tailwind class strings for the onboarding ritual — tokens only. */

export const FOCUS_RING =
  "focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring";

const BUTTON_BASE = `inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md border border-transparent font-ui font-semibold transition-colors duration-200 disabled:cursor-default aria-disabled:cursor-not-allowed aria-disabled:opacity-45 ${FOCUS_RING}`;

/** Design `.btn btn-primary` — 32px tall, 13px label. */
export const PRIMARY_BUTTON = `${BUTTON_BASE} h-8 bg-primary px-3.5 text-[13px] text-primary-foreground hover:bg-primary-active disabled:opacity-60`;

/** Design `.btn btn-secondary` — border-strong undo of primary. */
export const SECONDARY_BUTTON = `${BUTTON_BASE} h-8 border-border-strong bg-card px-3.5 text-[13px] text-foreground hover:bg-muted disabled:opacity-50`;

/** Design `.btn btn-ghost`. */
export const GHOST_BUTTON = `${BUTTON_BASE} h-8 px-3.5 text-[13px] text-body hover:bg-muted hover:text-foreground disabled:opacity-40`;

/** Design `.btn btn-primary btn-sm` — 26px tall, 11px label. */
export const PRIMARY_BUTTON_SM = `${BUTTON_BASE} h-[26px] bg-primary px-2.5 text-[11px] text-primary-foreground hover:bg-primary-active disabled:opacity-60`;

/** Design `.btn btn-secondary btn-sm`. */
export const SECONDARY_BUTTON_SM = `${BUTTON_BASE} h-[26px] border-border-strong bg-card px-2.5 text-[11px] text-foreground hover:bg-muted disabled:opacity-50`;

/** Design `.btn btn-ghost btn-sm`. */
export const GHOST_BUTTON_SM = `${BUTTON_BASE} h-[26px] px-2.5 text-[11px] text-body hover:bg-muted hover:text-foreground disabled:opacity-40`;

/** Small mono tag — design `.tag` (pill, uppercase 11px). */
export const TAG =
  "inline-flex w-fit flex-none items-center gap-1 whitespace-nowrap rounded-pill border border-border px-2 py-0.5 font-mono text-[11px] uppercase tracking-[0.04em] text-muted-foreground";

/** Design `.tag-ok`. */
export const TAG_OK = `${TAG} border-success text-success`;

/** Design `.tag-accent`. */
export const TAG_ACCENT = `${TAG} border-primary bg-primary-soft text-primary`;

/** Muted status chip — same convention as the panel's CloudIndicator. */
export const CHIP =
  "inline-flex w-fit items-center gap-1.5 whitespace-nowrap rounded-pill border border-border bg-muted px-2.5 py-1 font-ui text-[11px] font-medium";

/** Keycap — design `.kbd`. */
export const KBD =
  "rounded-[5px] border border-border-strong border-b-2 bg-card px-1.5 py-[2px] font-mono text-[11px] leading-[1.45] whitespace-nowrap text-muted-foreground";

/** Mono eyebrow above a step heading — design `.eyebrow` (11px, .09em). */
export const EYEBROW =
  "font-mono text-[11px] font-normal uppercase tracking-[0.09em] text-muted-foreground";

/** The single display-scale headline per step — design `.t-display` (28px/700). */
export const STEP_HEADING =
  "font-ui text-[28px] font-bold leading-[1.2] tracking-[-0.02em] text-foreground focus:outline-none";

/** Body copy — Inter 14px, `--body` colour (design `.t-body-muted`). */
export const BODY = "font-ui text-[14px] font-normal leading-[1.45] text-body";

/** Section label for a permission name / set row — Inter 14/600. */
export const STRONG = "font-ui text-[14px] font-semibold leading-[1.4] text-foreground";

/** Caption / helper copy — Inter 11/500 (design `.t-cap`). */
export const CAPTION = "font-ui text-[11px] font-medium leading-[1.4] text-muted-foreground";

/** Technical caption — mono 11px (design `.perm-tech` / `.t-mono`). */
export const TECH = "font-mono text-[11px] leading-[1.4] text-muted-foreground";

/** The boxed why-line — design `.why-line`. */
export const WHY_LINE =
  "font-ui text-[14px] font-semibold leading-[1.4] text-foreground bg-muted border border-border rounded-md px-3.5 py-3";
