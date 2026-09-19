/** Shared Tailwind class strings for the onboarding ritual — tokens only. */

export const FOCUS_RING =
  "focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring";

export const PRIMARY_BUTTON =
  `bg-primary text-primary-foreground hover:bg-primary-active flex h-9 items-center justify-center rounded px-4 font-ui text-[14px] font-semibold transition-colors duration-200 disabled:cursor-default disabled:opacity-60 ${FOCUS_RING}`;

export const GHOST_BUTTON =
  `text-muted-foreground hover:bg-muted hover:text-foreground flex h-9 items-center justify-center rounded px-4 font-ui text-[14px] font-semibold transition-colors duration-200 disabled:cursor-default disabled:opacity-40 ${FOCUS_RING}`;

export const SECONDARY_BUTTON =
  `border border-border bg-card text-foreground hover:bg-muted flex h-8 items-center justify-center whitespace-nowrap rounded px-3 font-ui text-[13px] font-semibold transition-colors duration-200 disabled:cursor-default disabled:opacity-50 ${FOCUS_RING}`;

export const PRIMARY_BUTTON_SM =
  `bg-primary text-primary-foreground hover:bg-primary-active flex h-8 items-center justify-center whitespace-nowrap rounded px-3 font-ui text-[13px] font-semibold transition-colors duration-200 disabled:cursor-default disabled:opacity-50 ${FOCUS_RING}`;

/** Muted status chip — same convention as the panel's CloudIndicator. */
export const CHIP =
  "inline-flex w-fit items-center gap-1.5 whitespace-nowrap rounded-pill border border-border bg-muted px-2.5 py-1 font-ui text-[11px] font-medium";

/** Keycap — same convention as PanelHeader's esc hint. */
export const KBD =
  "rounded-sm border border-border bg-card px-1.5 py-px font-mono text-[11px] leading-none text-foreground";

/** Small mono eyebrow above a step heading. */
export const EYEBROW =
  "font-mono text-[11px] font-medium uppercase tracking-[0.08em] text-muted-foreground";

/** The single display-scale headline per step. */
export const STEP_HEADING =
  "font-ui text-[22px] font-bold leading-[1.2] tracking-[-0.02em] text-foreground focus:outline-none";

/** Body copy — Inter. */
export const BODY = "font-ui text-[14px] font-normal leading-[1.5] text-muted-foreground";

/** Caption / helper copy — mono, muted. */
export const CAPTION = "font-mono text-[11px] leading-[1.45] text-muted-foreground";

/** Her voice — the why-line / promises. Nunito. */
export const WHY_LINE = "font-companion text-[15px] font-semibold leading-[1.35] text-foreground";
