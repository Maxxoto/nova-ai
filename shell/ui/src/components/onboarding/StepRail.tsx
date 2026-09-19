import { Fragment } from "react";
import { RITUAL_STEPS } from "./types";
import type { RitualStep } from "./types";

const LABELS: Record<RitualStep, string> = {
  welcome: "Welcome",
  permissions: "Permissions",
  "first-capture": "First capture",
  preferences: "Preferences",
  ready: "Ready",
};

/** `01 Welcome … 05 Ready` progress rail — current step highlighted. */
export default function StepRail({ current }: { current: RitualStep }) {
  const currentIndex = RITUAL_STEPS.indexOf(current);
  return (
    <div
      role="list"
      aria-label="Setup progress"
      className="flex flex-wrap items-center gap-x-2 gap-y-2 border-b border-border bg-muted px-5 py-3"
    >
      {RITUAL_STEPS.map((step, i) => {
        const isCurrent = i === currentIndex;
        const isDone = i < currentIndex;
        const dotClass = isCurrent ? "bg-primary" : isDone ? "bg-success" : "bg-border";
        const textClass = isCurrent ? "text-foreground" : "text-muted-foreground";
        return (
          <Fragment key={step}>
            <span
              role="listitem"
              aria-current={isCurrent ? "step" : undefined}
              className={`flex items-center gap-1.5 whitespace-nowrap font-ui text-[11px] font-semibold ${textClass}`}
            >
              <span aria-hidden className={`h-1.5 w-1.5 rounded-pill ${dotClass}`} />
              {String(i + 1).padStart(2, "0")} {LABELS[step]}
            </span>
            {i < RITUAL_STEPS.length - 1 && <span aria-hidden className="h-px min-w-3 flex-1 bg-border" />}
          </Fragment>
        );
      })}
    </div>
  );
}
