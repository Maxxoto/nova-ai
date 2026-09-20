import type { ReactElement } from "react";

interface GlyphProps {
  className?: string;
}

const BASE = "h-3.5 w-3.5 flex-none";

export function CheckGlyph({ className }: GlyphProps): ReactElement {
  return (
    <svg
      viewBox="0 0 24 24"
      className={className ?? BASE}
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

export function MinusGlyph({ className }: GlyphProps): ReactElement {
  return (
    <svg
      viewBox="0 0 24 24"
      className={className ?? BASE}
      fill="none"
      stroke="currentColor"
      strokeWidth="2.4"
      strokeLinecap="round"
      aria-hidden="true"
    >
      <line x1="5" y1="12" x2="19" y2="12" />
    </svg>
  );
}

export function DenyGlyph({ className }: GlyphProps): ReactElement {
  return (
    <svg
      viewBox="0 0 24 24"
      className={className ?? BASE}
      fill="none"
      stroke="currentColor"
      strokeWidth="2.2"
      strokeLinecap="round"
      aria-hidden="true"
    >
      <line x1="18" y1="6" x2="6" y2="18" />
      <line x1="6" y1="6" x2="18" y2="18" />
    </svg>
  );
}

export function DotGlyph({ className }: GlyphProps): ReactElement {
  return (
    <svg viewBox="0 0 24 24" className={className ?? BASE} aria-hidden="true">
      <circle cx="12" cy="12" r="4" fill="currentColor" />
    </svg>
  );
}

export function HelpGlyph({ className }: GlyphProps): ReactElement {
  return (
    <svg
      viewBox="0 0 24 24"
      className={className ?? BASE}
      fill="none"
      stroke="currentColor"
      strokeWidth="2.2"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <circle cx="12" cy="12" r="9" />
      <path d="M9.5 9.5a2.5 2.5 0 1 1 3.5 2.3c-.7.4-1 .9-1 1.7" />
      <line x1="12" y1="17" x2="12.01" y2="17" />
    </svg>
  );
}

export function ScreenGlyph({ className }: GlyphProps): ReactElement {
  return (
    <svg
      viewBox="0 0 24 24"
      className={className ?? "h-4 w-4 flex-none"}
      fill="none"
      stroke="currentColor"
      strokeWidth="1.7"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <rect x="3" y="4" width="18" height="13" rx="2" />
      <path d="M8 21h8" />
    </svg>
  );
}

export function MicGlyph({ className }: GlyphProps): ReactElement {
  return (
    <svg
      viewBox="0 0 24 24"
      className={className ?? "h-4 w-4 flex-none"}
      fill="none"
      stroke="currentColor"
      strokeWidth="1.7"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <rect x="9" y="3" width="6" height="11" rx="3" />
      <path d="M5 11a7 7 0 0 0 14 0M12 18v3" />
    </svg>
  );
}

export function AccessibilityGlyph({ className }: GlyphProps): ReactElement {
  return (
    <svg
      viewBox="0 0 24 24"
      className={className ?? "h-4 w-4 flex-none"}
      fill="none"
      stroke="currentColor"
      strokeWidth="1.7"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d="M6 9V6h3M18 9V6h-3M6 15v3h3M18 15v3h-3" />
    </svg>
  );
}
