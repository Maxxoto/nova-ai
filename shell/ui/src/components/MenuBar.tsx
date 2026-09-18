import type { ReactNode } from "react";

function WifiGlyph() {
  return (
    <svg className="menubar-glyph" width="17" height="13" viewBox="0 0 17 13" aria-hidden="true">
      <path
        d="M1.5 4.4a10.5 10.5 0 0 1 14 0M4 7.2a7 7 0 0 1 9 0M6.5 9.9a3.4 3.4 0 0 1 4 0"
        fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round"
      />
      <circle cx="8.5" cy="11.6" r="1.15" fill="currentColor" />
    </svg>
  );
}

function BatteryGlyph() {
  return (
    <svg className="menubar-glyph" width="25" height="13" viewBox="0 0 25 13" aria-hidden="true">
      <rect x="0.75" y="1.75" width="20.5" height="9.5" rx="3.25" fill="none" stroke="currentColor" strokeWidth="1.3" opacity="0.5" />
      <rect x="2.5" y="3.5" width="13" height="6" rx="1.75" fill="currentColor" />
      <path d="M23 4.8v3.4a2.1 2.1 0 0 0 0-3.4Z" fill="currentColor" opacity="0.5" />
    </svg>
  );
}

function ControlCenterGlyph() {
  return (
    <svg className="menubar-glyph" width="15" height="15" viewBox="0 0 15 15" aria-hidden="true">
      <rect x="1" y="1.5" width="5.4" height="12" rx="2.7" fill="none" stroke="currentColor" strokeWidth="1.4" />
      <rect x="8.6" y="1.5" width="5.4" height="12" rx="2.7" fill="none" stroke="currentColor" strokeWidth="1.4" />
      <circle cx="3.7" cy="4.6" r="1.2" fill="currentColor" />
      <circle cx="11.3" cy="10.4" r="1.2" fill="currentColor" />
    </svg>
  );
}

interface MenuBarProps {
  theme: "light" | "dark";
  /** Status items under review, rendered left of the system glyphs. */
  children: ReactNode;
  /** 28px strip (real menu bar) vs 56px loupe for 32/48px marks. */
  tall?: boolean;
}

export default function MenuBar({ theme, children, tall = false }: MenuBarProps) {
  return (
    <div
      className={`menubar-bar menubar-bar--${theme} ${tall ? "h-14 px-4" : "h-7 px-3"}`}
      role="img"
      aria-label={`${theme} menu bar mockup`}
    >
      <div className={`flex items-center gap-3 ${tall ? "text-[15px]" : ""}`}>
        <span className="font-semibold">Finder</span>
        <span className="opacity-70">File</span>
        <span className="opacity-70">Edit</span>
        <span className="opacity-70">View</span>
      </div>
      <div className="flex items-center gap-2.5">
        {children}
        <WifiGlyph />
        <BatteryGlyph />
        <ControlCenterGlyph />
        <span className={`menubar-glyph tabular-nums ${tall ? "text-[15px]" : ""}`}>
          Fri Sep 19&nbsp;&nbsp;9:41 AM
        </span>
      </div>
    </div>
  );
}
