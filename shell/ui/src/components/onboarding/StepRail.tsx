import { Fragment } from "react";
import { RITUAL_STEPS } from "./types";
import type { RitualStep } from "./types";

const LABELS: Record<RitualStep, string> = {
  welcome: "Welcome",
  permissions: "Access",
  "first-capture": "Capture",
  preferences: "Defaults",
  models: "Models",
  ready: "Ready",
};

/**
 * `01 Welcome … 06 Ready` progress rail — design `.wiz-rail`.
 * MONO 11px uppercase, tracking .06em; 6px dots, current = primary,
 * done = success, separators flex to fill.
 */
export default function StepRail({ current }: { current: RitualStep }) {
  const currentIndex = RITUAL_STEPS.indexOf(current);
  return (
    <div
      role="list"
      aria-label="Setup progress"
      className="flex flex-wrap items-center gap-x-2 gap-y-1.5 border-b border-border bg-muted px-[18px] py-3"
    >
      {RITUAL_STEPS.map((step, i) => {
        const isCurrent = i === currentIndex;
        const isDone = i < currentIndex;
        const dotClass = isCurrent ? "bg-primary" : isDone ? "bg-success" : "bg-border-strong";
        const textClass = isCurrent ? "text-foreground" : isDone ? "text-body" : "text-muted-foreground";
        return (
          <Fragment key={step}>
            <span
              role="listitem"
              aria-current={isCurrent ? "step" : undefined}
              className={`inline-flex items-center gap-[7px] whitespace-nowrap font-mono text-[11px] uppercase tracking-[0.06em] transition-colors duration-[120ms] ease-[cubic-bezier(.4,0,.2,1)] ${textClass}`}
            >
              <i
                aria-hidden
                className={`h-1.5 w-1.5 rounded-pill transition-colors duration-[120ms] ease-[cubic-bezier(.4,0,.2,1)] ${dotClass}`}
              />
              {String(i + 1).padStart(2, "0")} {LABELS[step]}
            </span>
            {i < RITUAL_STEPS.length - 1 && <span aria-hidden className="h-px flex-1 bg-border" />}
          </Fragment>
        );
      })}
    </div>
  );
}
