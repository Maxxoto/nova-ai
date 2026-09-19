import type { ReactElement } from "react";
import { useState } from "react";

function BookmarkGlyph(): ReactElement {
  return (
    <svg
      viewBox="0 0 24 24"
      className="h-3.5 w-3.5"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z" />
    </svg>
  );
}

function CheckGlyph(): ReactElement {
  return (
    <svg
      viewBox="0 0 24 24"
      className="h-3.5 w-3.5"
      fill="none"
      stroke="currentColor"
      strokeWidth="2.4"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <polyline points="20 6 9 17 4 12" />
    </svg>
  );
}

export default function SaveMemoryButton({ onSaved }: { onSaved?: () => void }) {
  const [saved, setSaved] = useState(false);
  return (
    <button
      type="button"
      disabled={saved}
      onClick={() => {
        setSaved(true);
        onSaved?.();
      }}
      className={`flex h-9 items-center gap-2 rounded-[10px] px-4 font-ui text-[14px] font-semibold transition-colors duration-200 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring disabled:cursor-default ${
        saved
          ? "border border-success text-success"
          : "bg-primary text-primary-foreground hover:bg-primary-active"
      }`}
    >
      {saved ? <CheckGlyph /> : <BookmarkGlyph />}
      <span>{saved ? "Saved" : "Save to memory"}</span>
    </button>
  );
}
