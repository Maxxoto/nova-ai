export default function ToolStepIndicator({ active = 1, total = 3 }: { active?: number; total?: number }) {
  return (
    <div
      role="progressbar"
      aria-label="tool step"
      aria-valuemin={0}
      aria-valuemax={total}
      aria-valuenow={active}
      className="panel-steps"
    >
      {Array.from({ length: total }, (_, i) => (
        <i key={i} data-step={i < active ? "done" : i === active ? "active" : "pending"} />
      ))}
    </div>
  );
}
