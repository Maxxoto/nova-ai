import type { ReactNode } from "react";

/** Focus-visible ring per DESIGN.md — 2px primary, offset 2. */
export const FOCUS_RING =
  "focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring";

export const DANGER_BUTTON = `inline-flex h-7 items-center justify-center rounded border border-destructive bg-card px-2.5 font-ui text-[11px] font-semibold text-destructive transition-colors duration-200 hover:bg-destructive/10 disabled:cursor-default disabled:opacity-45 ${FOCUS_RING}`;
export const PERMISSION_ACTION_BUTTON = `inline-flex h-7 items-center justify-center rounded px-2.5 font-ui text-[11px] font-semibold text-muted-foreground transition-colors duration-200 hover:bg-muted hover:text-foreground ${FOCUS_RING}`;
export const SECONDARY_BUTTON = `inline-flex h-8 items-center justify-center whitespace-nowrap rounded border border-border-strong bg-card px-3 font-ui text-[13px] font-semibold text-foreground transition-colors duration-200 hover:bg-muted ${FOCUS_RING}`;
export const GHOST_BUTTON_SM = `inline-flex h-8 items-center justify-center rounded px-3 font-ui text-[13px] font-semibold text-muted-foreground transition-colors duration-200 hover:bg-muted hover:text-foreground ${FOCUS_RING}`;
export const CHANGE_BUTTON = `inline-flex h-7 items-center justify-center rounded border border-border-strong bg-card px-2.5 font-ui text-[11px] font-semibold text-foreground transition-colors duration-200 hover:bg-muted ${FOCUS_RING}`;
export const PRIMARY_BUTTON_SM = `inline-flex h-8 items-center justify-center whitespace-nowrap rounded bg-primary px-3 font-ui text-[13px] font-semibold text-primary-foreground transition-colors duration-200 hover:bg-primary-active disabled:cursor-default disabled:opacity-50 ${FOCUS_RING}`;
export const PICK_BUTTON = `inline-flex h-7 items-center justify-center whitespace-nowrap rounded border border-border-strong bg-card px-2.5 font-ui text-[11px] font-semibold text-foreground transition-colors duration-200 hover:bg-muted aria-pressed:border-primary aria-pressed:bg-primary-soft ${FOCUS_RING}`;

export function Toggle({ label, on, onChange }: { label: string; on: boolean; onChange: (next: boolean) => void }) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={on}
      aria-label={label}
      onClick={() => onChange(!on)}
      className={`flex flex-none items-center rounded-pill border border-border bg-card p-[3px] shadow-e1 transition-colors duration-200 hover:border-border-strong ${FOCUS_RING}`}
    >
      <span
        aria-hidden="true"
        className={`relative inline-block rounded-pill transition-colors duration-200 ${
          on ? "bg-primary" : "bg-muted"
        }`}
        style={{ height: 20, width: 36 }}
      >
        <span
          className="absolute top-0.5 h-4 w-4 rounded-pill bg-card shadow-e1 transition-all duration-200"
          style={{ left: on ? 18 : 2 }}
        />
      </span>
    </button>
  );
}

export type TagTone = "neutral" | "ok" | "warn";

export function Tag({ children, tone = "neutral" }: { children: ReactNode; tone?: TagTone }) {
  const toneClass =
    tone === "ok"
      ? "border-success text-success"
      : tone === "warn"
        ? "border-warning text-warning"
        : "border-border text-muted-foreground";
  return (
    <span
      className={`inline-flex w-fit flex-none items-center rounded-pill border bg-muted px-2.5 py-1 font-ui text-[11px] font-medium ${toneClass}`}
    >
      {children}
    </span>
  );
}

export function Section({
  id,
  title,
  description,
  headerExtra,
  children,
}: {
  id: string;
  title: string;
  description?: ReactNode;
  headerExtra?: ReactNode;
  children: ReactNode;
}) {
  return (
    <section id={id} className="scroll-mt-24 flex flex-col gap-4 rounded-xl border border-border bg-card p-5">
      <div className="flex flex-col gap-1">
        <div className="flex flex-wrap items-center gap-2">
          <h2 className="font-ui text-[17px] font-semibold leading-[1.3] tracking-[-0.01em] text-foreground">
            {title}
          </h2>
          {headerExtra}
        </div>
        {description ? (
          <p className="max-w-[70ch] font-ui text-[14px] font-normal leading-[1.45] text-muted-foreground">
            {description}
          </p>
        ) : null}
      </div>
      {children}
    </section>
  );
}

export function Row({
  label,
  help,
  side,
  danger = false,
}: {
  label: string;
  help: ReactNode;
  side: ReactNode;
  danger?: boolean;
}) {
  return (
    <div className="flex items-start justify-between gap-6 border-t border-border py-3.5 first:border-t-0">
      <div className="flex min-w-0 flex-col gap-1">
        <span
          className={`font-ui text-[14px] font-semibold leading-[1.4] ${
            danger ? "text-destructive" : "text-foreground"
          }`}
        >
          {label}
        </span>
        <p className="max-w-[70ch] font-ui text-[13px] leading-[1.45] text-muted-foreground">{help}</p>
      </div>
      <div className="flex flex-none items-center gap-2">{side}</div>
    </div>
  );
}

export function AppRow({
  mark,
  name,
  caption,
  side,
}: {
  mark: string;
  name: string;
  caption: ReactNode;
  side?: ReactNode;
}) {
  return (
    <div className="flex items-center gap-3 rounded border border-border bg-card px-3 py-2.5">
      <span
        aria-hidden="true"
        className="grid h-7 w-7 flex-none place-items-center rounded-[7px] border border-border bg-muted font-mono text-[11px] text-muted-foreground"
      >
        {mark}
      </span>
      <div className="flex min-w-0 flex-1 flex-col gap-0.5">
        <span className="font-ui text-[13px] font-semibold leading-[1.4] text-foreground">{name}</span>
        <p className="font-ui text-[11px] leading-[1.4] text-muted-foreground">{caption}</p>
      </div>
      {side ? <div className="flex flex-none items-center gap-2">{side}</div> : null}
    </div>
  );
}

export function Segmented<T extends string>({
  ariaLabel,
  value,
  options,
  onChange,
}: {
  ariaLabel: string;
  value: T;
  options: readonly { value: T; label: string }[];
  onChange: (next: T) => void;
}) {
  return (
    <div
      role="group"
      aria-label={ariaLabel}
      className="inline-flex gap-0.5 rounded border border-border bg-muted p-0.5"
    >
      {options.map((option) => {
        const selected = option.value === value;
        return (
          <button
            key={option.value}
            type="button"
            aria-pressed={selected}
            onClick={() => onChange(option.value)}
            className={`rounded-sm px-2.5 py-1 font-ui text-[11px] font-semibold transition-colors duration-200 ${FOCUS_RING} ${
              selected ? "bg-card text-foreground shadow-e1" : "text-muted-foreground hover:text-foreground"
            }`}
          >
            {option.label}
          </button>
        );
      })}
    </div>
  );
}
