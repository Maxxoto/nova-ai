import type { ReactElement } from "react";
import { useState } from "react";

function BookmarkGlyph(): ReactElement {
  return (
    <svg
      viewBox="0 0 24 24"
      className="h-3.5 w-3.5"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.7"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d="M6 4h12v16l-6-4-6 4Z" />
    </svg>
  );
}

export default function SaveMemoryButton({ onSaved }: { onSaved?: () => void }) {
  const [saved, setSaved] = useState(false);
  return (
    <button
      type="button"
      aria-pressed={saved}
      disabled={saved}
      onClick={() => {
        setSaved(true);
        onSaved?.();
      }}
      className={`inline-flex h-[26px] items-center gap-1.5 rounded-[10px] border px-2.5 font-ui text-[13px] font-semibold transition-colors duration-200 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring disabled:cursor-default ${
        saved ? "border-success text-success" : "border-border-strong bg-card text-foreground hover:bg-muted"
      }`}
    >
      <BookmarkGlyph />
      <span>{saved ? "Saved to memory" : "Save to memory"}</span>
    </button>
  );
}
