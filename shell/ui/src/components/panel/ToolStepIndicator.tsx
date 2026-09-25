type StepStatus = "done" | "active" | "upcoming";

export default function ToolStepIndicator({ active = 1, total = 3 }: { active?: number; total?: number }) {
  return (
    <span
      role="img"
      aria-label={`step ${Math.min(active + 1, total)} of ${total}`}
      className="inline-flex flex-none items-center gap-1.5"
    >
      {Array.from({ length: total }, (_, i) => {
        const status: StepStatus = i < active ? "done" : i === active ? "active" : "upcoming";
        return (
          <i
            key={i}
            data-step={status}
            aria-current={status === "active" ? "step" : undefined}
            className={`step-dot step-dot--${status}`}
          />
        );
      })}
    </span>
  );
}
