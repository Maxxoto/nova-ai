export default function ToolStepIndicator({ active = 1, total = 3 }: { active?: number; total?: number }) {
  return (
    <span
      role="img"
      aria-label={`step ${Math.min(active + 1, total)} of ${total}`}
      className="inline-flex flex-none items-center gap-1.5"
    >
      {Array.from({ length: total }, (_, i) => (
        <i
          key={i}
          className={`h-1.5 w-1.5 rounded-full ${
            i < active ? "bg-primary" : i === active ? "bg-primary animate-pulse-ring" : "bg-border-strong"
          }`}
        />
      ))}
    </span>
  );
}
